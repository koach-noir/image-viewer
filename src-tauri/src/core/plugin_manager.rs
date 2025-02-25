// plugin_manager.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::any::Any;
use serde_json::Value as JsonValue;

use crate::core::event_bus::EventBus;
use crate::plugins::plugin_trait::Plugin;

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

/// プラグインコンテキスト - プラグインに提供される機能
pub struct PluginContext {
    /// イベントバス
    pub event_bus: Arc<EventBus>,
    /// 共有データストア
    pub shared_data: Arc<Mutex<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl PluginContext {
    /// 新しいPluginContextインスタンスを作成
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            shared_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 共有データを設定
    pub fn set_shared_data<T: 'static + Send + Sync>(&self, key: &str, value: T) -> Result<(), String> {
        match self.shared_data.lock() {
            Ok(mut data) => {
                data.insert(key.to_string(), Box::new(value));
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock shared data: {}", e)),
        }
    }

    /// 共有データを取得
    pub fn get_shared_data<T: 'static + Clone>(&self, key: &str) -> Result<Option<T>, String> {
        match self.shared_data.lock() {
            Ok(data) => {
                if let Some(value) = data.get(key) {
                    if let Some(typed_value) = value.downcast_ref::<T>() {
                        Ok(Some(typed_value.clone()))
                    } else {
                        Err(format!("Type mismatch for key: {}", key))
                    }
                } else {
                    Ok(None)
                }
            },
            Err(e) => Err(format!("Failed to lock shared data: {}", e)),
        }
    }
}

/// プラグイン登録情報
struct PluginRegistration {
    /// プラグインインスタンス
    plugin: Box<dyn Plugin>,
    /// プラグインの状態
    state: PluginState,
    /// エラーメッセージ（あれば）
    error: Option<String>,
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
                
                // 登録
                plugins.insert(plugin_id.clone(), PluginRegistration {
                    plugin,
                    state: PluginState::Registered,
                    error: None,
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
        let mut plugin_instance = {
            match self.plugins.lock() {
                Ok(mut plugins) => {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        if registration.state != PluginState::Registered {
                            return Err(format!("Plugin '{}' is not in a registerable state", plugin_id));
                        }
                        
                        &mut registration.plugin
                    } else {
                        return Err(format!("Plugin with ID '{}' not found", plugin_id));
                    }
                },
                Err(e) => return Err(format!("Failed to lock plugins: {}", e)),
            }
        };
        
        // プラグインを初期化
        match plugin_instance.initialize(Arc::clone(&self.context)) {
            Ok(_) => {
                // 状態を更新
                if let Ok(mut plugins) = self.plugins.lock() {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        registration.state = PluginState::Initialized;
                        registration.error = None;
                    }
                }
                
                // イベント発行
                self.event_bus.publish("plugin:initialized", serde_json::json!({
                    "plugin_id": plugin_id
                }))?;
                
                Ok(())
            },
            Err(e) => {
                // エラー状態を更新
                if let Ok(mut plugins) = self.plugins.lock() {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        registration.state = PluginState::Error;
                        registration.error = Some(e.clone());
                    }
                }
                
                // エラーイベント発行
                self.event_bus.publish("plugin:error", serde_json::json!({
                    "plugin_id": plugin_id,
                    "error": e.clone(),
                    "operation": "initialize"
                }))?;
                
                Err(e)
            }
        }
    }

    /// プラグインを有効化
    pub fn activate_plugin(&self, plugin_id: &str) -> Result<(), String> {
        let mut plugin_instance = {
            match self.plugins.lock() {
                Ok(mut plugins) => {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        // 初期化されていなければ初期化
                        if registration.state == PluginState::Registered {
                            drop(plugins); // ロックを解放
                            self.initialize_plugin(plugin_id)?;
                            if let Ok(plugins) = self.plugins.lock() {
                                &mut plugins.get_mut(plugin_id).unwrap().plugin
                            } else {
                                return Err("Failed to lock plugins after initialization".to_string());
                            }
                        } else if registration.state == PluginState::Active {
                            return Ok(()); // 既に有効化済み
                        } else if registration.state == PluginState::Error {
                            return Err(format!("Cannot activate plugin in error state: {}", 
                                              registration.error.as_ref().unwrap_or(&"Unknown error".to_string())));
                        } else {
                            &mut registration.plugin
                        }
                    } else {
                        return Err(format!("Plugin with ID '{}' not found", plugin_id));
                    }
                },
                Err(e) => return Err(format!("Failed to lock plugins: {}", e)),
            }
        };
        
        // プラグインを有効化
        match plugin_instance.activate() {
            Ok(_) => {
                // 状態を更新
                if let Ok(mut plugins) = self.plugins.lock() {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        registration.state = PluginState::Active;
                        registration.error = None;
                    }
                }
                
                // イベント発行
                self.event_bus.publish("plugin:activated", serde_json::json!({
                    "plugin_id": plugin_id
                }))?;
                
                Ok(())
            },
            Err(e) => {
                // エラー状態を更新
                if let Ok(mut plugins) = self.plugins.lock() {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        registration.state = PluginState::Error;
                        registration.error = Some(e.clone());
                    }
                }
                
                // エラーイベント発行
                self.event_bus.publish("plugin:error", serde_json::json!({
                    "plugin_id": plugin_id,
                    "error": e.clone(),
                    "operation": "activate"
                }))?;
                
                Err(e)
            }
        }
    }

    /// プラグインを無効化
    pub fn deactivate_plugin(&self, plugin_id: &str) -> Result<(), String> {
        let mut plugin_instance = {
            match self.plugins.lock() {
                Ok(mut plugins) => {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        if registration.state != PluginState::Active {
                            return Ok(()); // 既に非アクティブ
                        }
                        &mut registration.plugin
                    } else {
                        return Err(format!("Plugin with ID '{}' not found", plugin_id));
                    }
                },
                Err(e) => return Err(format!("Failed to lock plugins: {}", e)),
            }
        };
        
        // プラグインを無効化
        match plugin_instance.deactivate() {
            Ok(_) => {
                // 状態を更新
                if let Ok(mut plugins) = self.plugins.lock() {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        registration.state = PluginState::Inactive;
                        registration.error = None;
                    }
                }
                
                // イベント発行
                self.event_bus.publish("plugin:deactivated", serde_json::json!({
                    "plugin_id": plugin_id
                }))?;
                
                Ok(())
            },
            Err(e) => {
                // エラー状態を更新
                if let Ok(mut plugins) = self.plugins.lock() {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        registration.state = PluginState::Error;
                        registration.error = Some(e.clone());
                    }
                }
                
                // エラーイベント発行
                self.event_bus.publish("plugin:error", serde_json::json!({
                    "plugin_id": plugin_id,
                    "error": e.clone(),
                    "operation": "deactivate"
                }))?;
                
                Err(e)
            }
        }
    }

    /// プラグインを登録解除
    pub fn unregister_plugin(&self, plugin_id: &str) -> Result<(), String> {
        // アクティブなら先に無効化
        {
            let plugin_state = self.get_plugin_state(plugin_id)?;
            if plugin_state == PluginState::Active {
                self.deactivate_plugin(plugin_id)?;
            }
        }
        
        // 登録解除
        match self.plugins.lock() {
            Ok(mut plugins) => {
                if plugins.remove(plugin_id).is_some() {
                    // イベント発行
                    self.event_bus.publish("plugin:unregistered", serde_json::json!({
                        "plugin_id": plugin_id
                    }))?;
                    
                    Ok(())
                } else {
                    Err(format!("Plugin with ID '{}' not found", plugin_id))
                }
            },
            Err(e) => Err(format!("Failed to lock plugins: {}", e)),
        }
    }

    /// プラグインの状態を取得
    pub fn get_plugin_state(&self, plugin_id: &str) -> Result<PluginState, String> {
        match self.plugins.lock() {
            Ok(plugins) => {
                if let Some(registration) = plugins.get(plugin_id) {
                    Ok(registration.state)
                } else {
                    Err(format!("Plugin with ID '{}' not found", plugin_id))
                }
            },
            Err(e) => Err(format!("Failed to lock plugins: {}", e)),
        }
    }

    /// プラグインのエラーメッセージを取得
    pub fn get_plugin_error(&self, plugin_id: &str) -> Result<Option<String>, String> {
        match self.plugins.lock() {
            Ok(plugins) => {
                if let Some(registration) = plugins.get(plugin_id) {
                    Ok(registration.error.clone())
                } else {
                    Err(format!("Plugin with ID '{}' not found", plugin_id))
                }
            },
            Err(e) => Err(format!("Failed to lock plugins: {}", e)),
        }
    }

    /// 登録済みの全プラグインIDを取得
    pub fn get_all_plugin_ids(&self) -> Result<Vec<String>, String> {
        match self.plugins.lock() {
            Ok(plugins) => {
                Ok(plugins.keys().cloned().collect())
            },
            Err(e) => Err(format!("Failed to lock plugins: {}", e)),
        }
    }

    /// アクティブな全プラグインIDを取得
    pub fn get_active_plugin_ids(&self) -> Result<Vec<String>, String> {
        match self.plugins.lock() {
            Ok(plugins) => {
                Ok(plugins.iter()
                    .filter(|(_, reg)| reg.state == PluginState::Active)
                    .map(|(id, _)| id.clone())
                    .collect())
            },
            Err(e) => Err(format!("Failed to lock plugins: {}", e)),
        }
    }

    /// プラグインの設定を取得
    pub fn get_plugin_config(&self, plugin_id: &str) -> Result<JsonValue, String> {
        let plugin_instance = {
            match self.plugins.lock() {
                Ok(plugins) => {
                    if let Some(registration) = plugins.get(plugin_id) {
                        registration.plugin.as_ref()
                    } else {
                        return Err(format!("Plugin with ID '{}' not found", plugin_id));
                    }
                },
                Err(e) => return Err(format!("Failed to lock plugins: {}", e)),
            }
        };
        
        plugin_instance.get_config()
    }

    /// プラグインの設定を更新
    pub fn update_plugin_config(&self, plugin_id: &str, config: JsonValue) -> Result<(), String> {
        let mut plugin_instance = {
            match self.plugins.lock() {
                Ok(mut plugins) => {
                    if let Some(registration) = plugins.get_mut(plugin_id) {
                        &mut registration.plugin
                    } else {
                        return Err(format!("Plugin with ID '{}' not found", plugin_id));
                    }
                },
                Err(e) => return Err(format!("Failed to lock plugins: {}", e)),
            }
        };
        
        plugin_instance.update_config(config)
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
