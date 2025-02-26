// src-tauri/src/core/plugin_manager.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// use serde_json::Value as JsonValue;
use thiserror::Error;

use crate::core::plugin_context::PluginContext;
use crate::core::event_bus::EventBus;
// use crate::plugins::plugin_trait::{Plugin, PluginDescriptor};
use crate::plugins::plugin_trait::Plugin;

/// プラグインマネージャーのエラー型
#[derive(Error, Debug)]
pub enum PluginManagerError {
    #[error("Plugin with ID '{0}' already exists")]
    PluginAlreadyExists(String),
    
    #[error("Plugin with ID '{0}' not found")]
    PluginNotFound(String),
    
    #[error("Failed to initialize plugin '{0}': {1}")]
    InitializationFailed(String, String),
    
    #[error("Plugin feature '{0}' is not enabled")]
    FeatureNotEnabled(String),
}

/// プラグインの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// 登録済みだが初期化されていない
    Registered,
    /// 初期化済み
    Initialized,
    /// 有効化済み（アクティブ）
    Active,
    /// 無効化済み
    Inactive,
    /// エラー状態
    Error,
}

/// プラグインの登録情報
struct PluginRegistration {
    /// プラグインインスタンス
    plugin: Box<dyn Plugin>,
    /// プラグインの状態
    state: PluginState,
    /// エラーメッセージ（エラー状態の場合）
    error: Option<String>,
    /// プラグインの依存関係
    dependencies: Vec<String>,
    /// 関連するフィーチャーフラグ
    feature_flag: Option<String>,
}

/// プラグインマネージャー
pub struct PluginManager {
    /// 登録されたプラグイン
    plugins: Arc<Mutex<HashMap<String, PluginRegistration>>>,
    /// イベントバス
    event_bus: Arc<EventBus>,
    /// プラグインコンテキスト
    context: Arc<PluginContext>,
    /// 有効なフィーチャーフラグ
    enabled_features: Arc<Mutex<Vec<String>>>,
}

impl PluginManager {
    /// 新しいプラグインマネージャーを作成
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let context = Arc::new(PluginContext::new(Arc::clone(&event_bus)));
        
        // 有効なフィーチャーフラグを取得
        let enabled_features = get_enabled_features();
        
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            event_bus,
            context,
            enabled_features: Arc::new(Mutex::new(enabled_features)),
        }
    }
    
    /// プラグインを登録
    pub fn register_plugin(
        &self, 
        plugin: Box<dyn Plugin>, 
        feature_flag: Option<String>
    ) -> Result<(), PluginManagerError> {
        let plugin_id = plugin.get_id();
        
        // フィーチャーフラグのチェック
        if let Some(flag) = &feature_flag {
            if !self.is_feature_enabled(flag) {
                return Err(PluginManagerError::FeatureNotEnabled(flag.clone()));
            }
        }
        
        // 登録処理
        let mut plugins = self.plugins.lock().map_err(|e| {
            PluginManagerError::InitializationFailed(
                plugin_id.clone(), 
                format!("Failed to lock plugins: {}", e)
            )
        })?;
        
        if plugins.contains_key(&plugin_id) {
            return Err(PluginManagerError::PluginAlreadyExists(plugin_id));
        }
        
        let registration = PluginRegistration {
            plugin,
            state: PluginState::Registered,
            error: None,
            dependencies: Vec::new(), // 将来的に依存関係を追加
            feature_flag,
        };
        
        plugins.insert(plugin_id.clone(), registration);
        
        // イベント発行
        let _ = self.event_bus.publish("plugin:registered", serde_json::json!({
            "plugin_id": plugin_id,
        }));
        
        Ok(())
    }
    
    /// プラグインを初期化
    pub fn initialize_plugin(&self, plugin_id: &str) -> Result<(), PluginManagerError> {
        let mut plugins = self.plugins.lock().map_err(|e| {
            PluginManagerError::InitializationFailed(
                plugin_id.to_string(), 
                format!("Failed to lock plugins: {}", e)
            )
        })?;
        
        let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginManagerError::PluginNotFound(plugin_id.to_string())
        })?;
        
        // すでに初期化済みの場合は何もしない
        if entry.state != PluginState::Registered {
            return Ok(());
        }
        
        // フィーチャーフラグのチェック
        if let Some(flag) = &entry.feature_flag {
            if !self.is_feature_enabled(flag) {
                return Err(PluginManagerError::FeatureNotEnabled(flag.clone()));
            }
        }
        
        // 初期化処理
        match entry.plugin.initialize(Arc::clone(&self.context)) {
            Ok(_) => {
                entry.state = PluginState::Initialized;
                entry.error = None;
                
                // イベント発行
                let _ = self.event_bus.publish("plugin:initialized", serde_json::json!({
                    "plugin_id": plugin_id,
                }));
                
                Ok(())
            },
            Err(e) => {
                entry.state = PluginState::Error;
                entry.error = Some(e.clone());
                
                Err(PluginManagerError::InitializationFailed(
                    plugin_id.to_string(), 
                    e
                ))
            }
        }
    }
    
    /// プラグインを有効化
    pub fn activate_plugin(&self, plugin_id: &str) -> Result<(), PluginManagerError> {
        let mut plugins = self.plugins.lock().map_err(|e| {
            PluginManagerError::InitializationFailed(
                plugin_id.to_string(), 
                format!("Failed to lock plugins: {}", e)
            )
        })?;
        
        let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginManagerError::PluginNotFound(plugin_id.to_string())
        })?;
        
        // すでに有効化済みの場合は何もしない
        if entry.state == PluginState::Active {
            return Ok(());
        }
        
        // フィーチャーフラグのチェック
        if let Some(flag) = &entry.feature_flag {
            if !self.is_feature_enabled(flag) {
                return Err(PluginManagerError::FeatureNotEnabled(flag.clone()));
            }
        }
        
        // 初期化されていない場合は初期化
        if entry.state == PluginState::Registered {
            self.initialize_plugin(plugin_id)?;
        }
        
        // 有効化処理
        match entry.plugin.activate() {
            Ok(_) => {
                entry.state = PluginState::Active;
                entry.error = None;
                
                // イベント発行
                let _ = self.event_bus.publish("plugin:activated", serde_json::json!({
                    "plugin_id": plugin_id,
                }));
                
                Ok(())
            },
            Err(e) => {
                entry.state = PluginState::Error;
                entry.error = Some(e.clone());
                
                Err(PluginManagerError::InitializationFailed(
                    plugin_id.to_string(), 
                    e
                ))
            }
        }
    }
    
    /// プラグインの状態を取得
    pub fn get_plugin_state(&self, plugin_id: &str) -> Result<PluginState, PluginManagerError> {
        let plugins = self.plugins.lock().map_err(|e| {
            PluginManagerError::InitializationFailed(
                plugin_id.to_string(), 
                format!("Failed to lock plugins: {}", e)
            )
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            PluginManagerError::PluginNotFound(plugin_id.to_string())
        })?;
        
        Ok(entry.state)
    }
    
    /// 有効なプラグイン数を取得
    pub fn get_active_plugin_count(&self) -> usize {
        let plugins = self.plugins.lock().unwrap();
        plugins.values()
            .filter(|entry| entry.state == PluginState::Active)
            .count()
    }
    
    /// フィーチャーフラグが有効かチェック
    fn is_feature_enabled(&self, feature: &str) -> bool {
        let features = self.enabled_features.lock().unwrap();
        features.contains(&feature.to_string())
    }
    
    /// 現在の有効なプラグインを取得
    pub fn get_active_plugins(&self) -> Vec<Box<dyn Plugin>> {
        let plugins = self.plugins.lock().unwrap();
        plugins.values()
            .filter(|entry| entry.state == PluginState::Active)
            .map(|entry| entry.plugin.clone())
            .collect()
    }
}

/// 現在有効なフィーチャーフラグを取得
fn get_enabled_features() -> Vec<String> {
    let mut features = Vec::new();
    
    // コンパイル時のフィーチャーフラグをチェック
    #[cfg(feature = "plugin-allviewer")]
    features.push("plugin-allviewer".to_string());
    
    #[cfg(feature = "plugin-findme")]
    features.push("plugin-findme".to_string());
    
    features
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::plugin_trait::MockPlugin;
    
    // テスト用のヘルパー関数
    fn create_test_plugin(id: &str) -> Box<dyn Plugin> {
        Box::new(MockPlugin::new(id))
    }
    
    #[test]
    fn test_plugin_registration() {
        let event_bus = Arc::new(EventBus::new());
        let plugin_manager = PluginManager::new(Arc::clone(&event_bus));
        
        // モックプラグインを作成して登録
        let plugin = create_test_plugin("test-plugin");
        assert!(plugin_manager.register_plugin(plugin, None).is_ok());
        
        // 同じIDのプラグインを再度登録しようとするとエラー
        let duplicate_plugin = create_test_plugin("test-plugin");
        assert!(plugin_manager.register_plugin(duplicate_plugin, None).is_err());
    }
    
    #[test]
    fn test_plugin_lifecycle() {
        let event_bus = Arc::new(EventBus::new());
        let plugin_manager = PluginManager::new(Arc::clone(&event_bus));
        
        let plugin = create_test_plugin("test-plugin");
        let plugin_id = plugin.get_id();
        
        // プラグインを登録
        assert!(plugin_manager.register_plugin(plugin, None).is_ok());
        
        // 状態確認
        assert_eq!(
            plugin_manager.get_plugin_state(&plugin_id).unwrap(), 
            PluginState::Registered
        );
        
        // 初期化
        assert!(plugin_manager.initialize_plugin(&plugin_id).is_ok());
        assert_eq!(
            plugin_manager.get_plugin_state(&plugin_id).unwrap(), 
            PluginState::Initialized
        );
        
        // 有効化
        assert!(plugin_manager.activate_plugin(&plugin_id).is_ok());
        assert_eq!(
            plugin_manager.get_plugin_state(&plugin_id).unwrap(), 
            PluginState::Active
        );
    }
    
    #[test]
    fn test_feature_flag_management() {
        let event_bus = Arc::new(EventBus::new());
        let plugin_manager = PluginManager::new(Arc::clone(&event_bus));
        
        // フィーチャーフラグ付きのプラグイン登録
        let plugin = create_test_plugin("feature-plugin");
        
        // アクティブなフィーチャーフラグ（デフォルトで存在する可能性のあるもの）が
        // テスト環境で有効化されているか確認
        let plugin_result = plugin_manager.register_plugin(
            plugin, 
            Some("plugin-allviewer".to_string())
        );
        
        // フィーチャーフラグのチェックは現在のビルド設定に依存
        assert!(plugin_result.is_ok() || plugin_result.is_err());
    }
}
