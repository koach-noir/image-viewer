// plugin_manager.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value as JsonValue;

use crate::core::event_bus::EventBus;
use crate::plugins::plugin_trait::Plugin;
use crate::core::plugin_context::PluginContext; // 共通のPluginContextをインポート

/// プラグインの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// 登録済み、まだ初期化されていない
    Registered,
    /// 初期化済み
    Initialized,
    /// 有効化済み
    Active,
    /// 無効化済み
    Inactive,
    /// エラー発生
    Error,
}

/// プラグイン登録情報
struct PluginRegistration {
    /// プラグインインスタンス
    plugin: Box<dyn Plugin>,
    /// プラグインの状態
    state: PluginState,
    /// エラーメッセージ（あれば）
    error: Option<String>,
    /// プラグインの依存関係
    dependencies: Vec<String>,
}

/// プラグインマネージャークラス
pub struct PluginManager {
    /// 登録されたプラグイン
    plugins: Arc<Mutex<HashMap<String, PluginRegistration>>>,
    /// プラグインコンテキスト
    context: Arc<PluginContext>,
    /// イベントバス
    event_bus: Arc<EventBus>,
}

impl PluginManager {
    /// 新しいPluginManagerインスタンスを作成
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let context = Arc::new(PluginContext::new(Arc::clone(&event_bus)));
        
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            context,
            event_bus,
        }
    }

    /// プラグインを登録
    pub fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let plugin_id = plugin.get_id();
        
        match self.plugins.lock() {
            Ok(mut plugins) => {
                // 既に登録されているかチェック
                if plugins.contains_key(&plugin_id) {
                    return Err(format!("Plugin with ID '{}' is already registered", plugin_id));
                }
                
                // 依存関係の抽出
                let dependencies = Vec::new(); // 将来的に拡張
                
                // 登録
                plugins.insert(plugin_id.clone(), PluginRegistration {
                    plugin,
                    state: PluginState::Registered,
                    error: None,
                    dependencies,
                });
                
                // イベント発行
                self.event_bus.publish("plugin:registered", serde_json::json!({
                    "plugin_id": plugin_id
                }))?;
                
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock plugins: {}", e)),
        }
    }

    /// プラグインを初期化
    pub fn initialize_plugin(&self, plugin_id: &str) -> Result<(), String> {
        // 依存関係の確認と初期化
        let dependencies = {
            let plugins = self.plugins.lock().map_err(|e| {
                format!("Failed to lock plugins registry: {}", e)
            })?;
            
            let entry = plugins.get(plugin_id).ok_or_else(|| {
                format!("Plugin with ID '{}' not found", plugin_id)
            })?;
            
            entry.dependencies.clone()
        };
        
        // 依存関係を先に初期化
        for dep_id in &dependencies {
            if let Err(e) = self.initialize_plugin(dep_id) {
                return Err(format!("Failed to initialize dependency {}: {}", dep_id, e));
            }
        }
        
        // プラグインの状態チェック
        let plugin_state = self.get_plugin_state(plugin_id)?;
        if plugin_state != PluginState::Registered {
            // 既に初期化済みの場合は成功
            if plugin_state == PluginState::Initialized || 
               plugin_state == PluginState::Active {
                return Ok(());
            }
            // エラー状態の場合
            if plugin_state == PluginState::Error {
                return Err(format!("Plugin '{}' is in error state", plugin_id));
            }
        }
        
        // プラグインの初期化
        let result = {
            let mut plugins = self.plugins.lock().map_err(|e| {
                format!("Failed to lock plugins registry for writing: {}", e)
            })?;
            
            let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
                format!("Plugin with ID '{}' not found", plugin_id)
            })?;
            
            let plugin_instance = &mut entry.plugin;
            
            // 初期化処理
            match plugin_instance.initialize(Arc::clone(&self.context)) {
                Ok(()) => {
                    entry.state = PluginState::Initialized;
                    entry.error = None;
                    Ok(())
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    entry.state = PluginState::Error;
                    entry.error = Some(error_msg.clone());
                    Err(format!("Failed to initialize plugin {}: {}", plugin_id, error_msg))
                }
            }
        };
        
        // 結果に基づいてイベント発行
        if result.is_ok() {
            self.event_bus.publish("plugin:initialized", serde_json::json!({
                "plugin_id": plugin_id,
            }))?;
            log::info!("Plugin initialized: {}", plugin_id);
        } else {
            self.event_bus.publish("plugin:error", serde_json::json!({
                "plugin_id": plugin_id,
                "error": result.as_ref().err().unwrap(),
                "operation": "initialize",
            }))?;
            log::error!("Failed to initialize plugin {}: {}", plugin_id, result.as_ref().err().unwrap());
        }
        
        result
    }

    /// プラグインを有効化
    pub fn activate_plugin(&self, plugin_id: &str) -> Result<(), String> {
        // プラグインの状態をチェック
        let plugin_state = self.get_plugin_state(plugin_id)?;
        
        // 既に有効化されている場合は何もしない
        if plugin_state == PluginState::Active {
            return Ok(());
        }
        
        // エラー状態の場合は有効化できない
        if plugin_state == PluginState::Error {
            let error = match self.get_plugin_error(plugin_id)? {
                Some(err) => err,
                None => "Unknown error".to_string(),
            };
            return Err(format!("Cannot activate plugin {} due to error: {}", plugin_id, error));
        }
        
        // 初期化されていない場合は初期化
        if plugin_state == PluginState::Registered {
            self.initialize_plugin(plugin_id)?;
        }
        
        // プラグインを有効化
        let result = {
            let mut plugins = self.plugins.lock().map_err(|e| {
                format!("Failed to lock plugins registry for writing: {}", e)
            })?;
            
            let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
                format!("Plugin with ID '{}' not found", plugin_id)
            })?;
            
            let plugin_instance = &mut entry.plugin;
            
            // 有効化処理
            match plugin_instance.activate() {
                Ok(()) => {
                    entry.state = PluginState::Active;
                    entry.error = None;
                    Ok(())
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    entry.state = PluginState::Error;
                    entry.error = Some(error_msg.clone());
                    Err(format!("Failed to activate plugin {}: {}", plugin_id, error_msg))
                }
            }
        };
        
        // 結果に基づいてイベント発行
        if result.is_ok() {
            self.event_bus.publish("plugin:activated", serde_json::json!({
                "plugin_id": plugin_id,
            }))?;
            log::info!("Plugin activated: {}", plugin_id);
        } else {
            self.event_bus.publish("plugin:error", serde_json::json!({
                "plugin_id": plugin_id,
                "error": result.as_ref().err().unwrap(),
                "operation": "activate",
            }))?;
            log::error!("Failed to activate plugin {}: {}", plugin_id, result.as_ref().err().unwrap());
        }
        
        result
    }

    /// プラグインを無効化
    pub fn deactivate_plugin(&self, plugin_id: &str) -> Result<(), String> {
        // プラグインの状態をチェック
        let plugin_state = self.get_plugin_state(plugin_id)?;
        
        // 有効化されていない場合は何もしない
        if plugin_state != PluginState::Active {
            return Ok(());
        }
        
        // プラグインを無効化
        let result = {
            let mut plugins = self.plugins.lock().map_err(|e| {
                format!("Failed to lock plugins registry for writing: {}", e)
            })?;
            
            let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
                format!("Plugin with ID '{}' not found", plugin_id)
            })?;
            
            let plugin_instance = &mut entry.plugin;
            
            // 無効化処理
            match plugin_instance.deactivate() {
                Ok(()) => {
                    entry.state = PluginState::Inactive;
                    entry.error = None;
                    Ok(())
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    entry.state = PluginState::Error;
                    entry.error = Some(error_msg.clone());
                    Err(format!("Failed to deactivate plugin {}: {}", plugin_id, error_msg))
                }
            }
        };
        
        // 結果に基づいてイベント発行
        if result.is_ok() {
            self.event_bus.publish("plugin:deactivated", serde_json::json!({
                "plugin_id": plugin_id,
            }))?;
            log::info!("Plugin deactivated: {}", plugin_id);
        } else {
            self.event_bus.publish("plugin:error", serde_json::json!({
                "plugin_id": plugin_id,
                "error": result.as_ref().err().unwrap(),
                "operation": "deactivate",
            }))?;
            log::error!("Failed to deactivate plugin {}: {}", plugin_id, result.as_ref().err().unwrap());
        }
        
        result
    }

    /// プラグインを登録解除
    pub fn unregister_plugin(&self, plugin_id: &str) -> Result<(), String> {
        // アクティブなら先に無効化
        let plugin_state = self.get_plugin_state(plugin_id)?;
        if plugin_state == PluginState::Active {
            self.deactivate_plugin(plugin_id)?;
        }
        
        // 登録解除
        let mut plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry for writing: {}", e)
        })?;
        
        if plugins.remove(plugin_id).is_some() {
            // イベント発行
            drop(plugins); // ロックを解放
            self.event_bus.publish("plugin:unregistered", serde_json::json!({
                "plugin_id": plugin_id,
            }))?;
            
            log::info!("Plugin unregistered: {}", plugin_id);
            Ok(())
        } else {
            Err(format!("Plugin with ID '{}' not found", plugin_id))
        }
    }

    /// プラグインの状態を取得
    pub fn get_plugin_state(&self, plugin_id: &str) -> Result<PluginState, String> {
        let plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry: {}", e)
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            format!("Plugin with ID '{}' not found", plugin_id)
        })?;
        
        Ok(entry.state)
    }

    /// プラグインのエラーメッセージを取得
    pub fn get_plugin_error(&self, plugin_id: &str) -> Result<Option<String>, String> {
        let plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry: {}", e)
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            format!("Plugin with ID '{}' not found", plugin_id)
        })?;
        
        Ok(entry.error.clone())
    }

    /// 登録済みの全プラグインIDを取得
    pub fn get_all_plugin_ids(&self) -> Result<Vec<String>, String> {
        let plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry: {}", e)
        })?;
        
        Ok(plugins.keys().cloned().collect())
    }
    
    /// プラグイン数を取得
    pub fn get_plugin_count(&self) -> Result<usize, String> {
        let plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry: {}", e)
        })?;
        
        Ok(plugins.len())
    }

    /// アクティブな全プラグインIDを取得
    pub fn get_active_plugin_ids(&self) -> Result<Vec<String>, String> {
        let plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry: {}", e)
        })?;
        
        Ok(plugins.iter()
            .filter(|(_, reg)| reg.state == PluginState::Active)
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// プラグインの設定を取得
    pub fn get_plugin_config(&self, plugin_id: &str) -> Result<JsonValue, String> {
        let plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry: {}", e)
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            format!("Plugin with ID '{}' not found", plugin_id)
        })?;
        
        entry.plugin.get_config()
    }

    /// プラグインの設定を更新
    pub fn update_plugin_config(&self, plugin_id: &str, config: JsonValue) -> Result<(), String> {
        let mut plugins = self.plugins.lock().map_err(|e| {
            format!("Failed to lock plugins registry for writing: {}", e)
        })?;
        
        let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
            format!("Plugin with ID '{}' not found", plugin_id)
        })?;
        
        entry.plugin.update_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::plugin_trait::MockPlugin;

    #[test]
    fn test_plugin_lifecycle() {
        let event_bus = Arc::new(EventBus::new());
        let plugin_manager = PluginManager::new(Arc::clone(&event_bus));
        
        // モックプラグインを作成
        let mock_plugin = Box::new(MockPlugin::new("test-plugin"));
        
        // 登録
        plugin_manager.register_plugin(mock_plugin).unwrap();
        
        // ステータス確認
        assert_eq!(plugin_manager.get_plugin_state("test-plugin").unwrap(), PluginState::Registered);
        
        // 初期化
        plugin_manager.initialize_plugin("test-plugin").unwrap();
        assert_eq!(plugin_manager.get_plugin_state("test-plugin").unwrap(), PluginState::Initialized);
        
        // 有効化
        plugin_manager.activate_plugin("test-plugin").unwrap();
        assert_eq!(plugin_manager.get_plugin_state("test-plugin").unwrap(), PluginState::Active);
        
        // 無効化
        plugin_manager.deactivate_plugin("test-plugin").unwrap();
        assert_eq!(plugin_manager.get_plugin_state("test-plugin").unwrap(), PluginState::Inactive);
        
        // 登録解除
        plugin_manager.unregister_plugin("test-plugin").unwrap();
        assert!(plugin_manager.get_plugin_state("test-plugin").is_err());
    }
}
