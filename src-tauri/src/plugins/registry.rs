// plugins/registry.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;
use serde_json::Value as JsonValue;

use crate::core::event_bus::EventBus;
use crate::plugins::plugin_trait::{Plugin, PluginDescriptor, PluginContext};

/// プラグインレジストリのエラー型
#[derive(Error, Debug)]
pub enum PluginRegistryError {
    #[error("Plugin with ID '{0}' already exists")]
    PluginAlreadyExists(String),
    
    #[error("Plugin with ID '{0}' not found")]
    PluginNotFound(String),
    
    #[error("Failed to initialize plugin '{0}': {1}")]
    InitializationFailed(String, String),
    
    #[error("Failed to load plugin from path '{0}': {1}")]
    LoadFailed(String, String),
    
    #[error("Dependency '{0}' required by plugin '{1}' not found")]
    DependencyNotFound(String, String),
    
    #[error("Error during plugin operation: {0}")]
    OperationError(String),
    
    #[error("Plugin system error: {0}")]
    SystemError(String),
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
struct RegistryEntry {
    /// プラグインインスタンス
    plugin: Box<dyn Plugin>,
    /// プラグインの状態
    state: PluginState,
    /// エラーメッセージ（エラー状態の場合）
    error: Option<String>,
    /// プラグインの依存関係
    dependencies: Vec<String>,
}

/// プラグインレジストリ
/// プラグインの登録と検出を管理する
pub struct PluginRegistry {
    /// 登録されたプラグイン
    plugins: RwLock<HashMap<String, RegistryEntry>>,
    /// イベントバス
    event_bus: Arc<EventBus>,
    /// プラグインコンテキスト
    context: Arc<PluginContext>,
    /// ディスカバリーパス（プラグインを検索するディレクトリ）
    discovery_paths: Mutex<Vec<String>>,
}

impl PluginRegistry {
    /// 新しいプラグインレジストリを作成
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let context = Arc::new(PluginContext {
            event_bus: Arc::clone(&event_bus),
        });
        
        Self {
            plugins: RwLock::new(HashMap::new()),
            event_bus,
            context,
            discovery_paths: Mutex::new(Vec::new()),
        }
    }
    
    /// プラグインを登録
    pub fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<(), PluginRegistryError> {
        let plugin_id = plugin.get_id();
        
        // 既存プラグインチェック
        {
            let plugins = self.plugins.read().map_err(|e| {
                PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
            })?;
            
            if plugins.contains_key(&plugin_id) {
                return Err(PluginRegistryError::PluginAlreadyExists(plugin_id));
            }
        }
        
        // 依存関係の抽出
        let descriptor = plugin.get_descriptor();
        let dependencies = self.extract_dependencies(&descriptor);
        
        // プラグイン登録
        {
            let mut plugins = self.plugins.write().map_err(|e| {
                PluginRegistryError::SystemError(format!("Failed to lock plugins registry for writing: {}", e))
            })?;
            
            plugins.insert(plugin_id.clone(), RegistryEntry {
                plugin,
                state: PluginState::Registered,
                error: None,
                dependencies,
            });
        }
        
        // イベント発行
        let _ = self.event_bus.publish("plugin:registered", serde_json::json!({
            "plugin_id": plugin_id.clone(),
            "descriptor": descriptor,
        }));
        
        log::info!("Plugin registered: {}", plugin_id);
        Ok(())
    }
    
    /// 依存関係情報を抽出（プラグイン記述子から）
    /// 注: 現在のプラグイン記述子に依存関係情報がないため、空の配列を返す
    /// 将来的に依存関係を追加する場合はここを拡張
    fn extract_dependencies(&self, _descriptor: &PluginDescriptor) -> Vec<String> {
        // 現在は依存関係サポートなし - 将来的に拡張予定
        Vec::new()
    }
    
    /// プラグインを初期化
    pub fn initialize_plugin(&self, plugin_id: &str) -> Result<(), PluginRegistryError> {
        // 依存関係の確認と初期化
        let dependencies = {
            let plugins = self.plugins.read().map_err(|e| {
                PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
            })?;
            
            let entry = plugins.get(plugin_id).ok_or_else(|| {
                PluginRegistryError::PluginNotFound(plugin_id.to_string())
            })?;
            
            entry.dependencies.clone()
        };
        
        // 依存関係を先に初期化
        for dep_id in &dependencies {
            if let Err(e) = self.initialize_plugin(dep_id) {
                return Err(PluginRegistryError::DependencyNotFound(
                    dep_id.clone(),
                    plugin_id.to_string(),
                ));
            }
        }
        
        // プラグインの状態を確認
        let plugin_state = self.get_plugin_state(plugin_id)?;
        if plugin_state != PluginState::Registered {
            // 既に初期化済みまたはエラー状態の場合
            if plugin_state == PluginState::Error {
                return Err(PluginRegistryError::OperationError(
                    format!("Plugin '{}' is in error state", plugin_id)
                ));
            }
            // 既に初期化済みの場合は成功とみなす
            return Ok(());
        }
        
        // プラグインを初期化
        let result = {
            let mut plugins = self.plugins.write().map_err(|e| {
                PluginRegistryError::SystemError(format!("Failed to lock plugins registry for writing: {}", e))
            })?;
            
            let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
                PluginRegistryError::PluginNotFound(plugin_id.to_string())
            })?;
            
            // 初期化処理
            match entry.plugin.initialize(Arc::clone(&self.context)) {
                Ok(()) => {
                    entry.state = PluginState::Initialized;
                    entry.error = None;
                    Ok(())
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    entry.state = PluginState::Error;
                    entry.error = Some(error_msg.clone());
                    Err(PluginRegistryError::InitializationFailed(
                        plugin_id.to_string(),
                        error_msg,
                    ))
                }
            }
        };
        
        // 結果に基づいてイベント発行
        if result.is_ok() {
            let _ = self.event_bus.publish("plugin:initialized", serde_json::json!({
                "plugin_id": plugin_id,
            }));
            log::info!("Plugin initialized: {}", plugin_id);
        } else if let Err(PluginRegistryError::InitializationFailed(id, error)) = &result {
            let _ = self.event_bus.publish("plugin:error", serde_json::json!({
                "plugin_id": id,
                "error": error,
                "operation": "initialize",
            }));
            log::error!("Failed to initialize plugin {}: {}", id, error);
        }
        
        result
    }
    
    /// プラグインの状態を取得
    pub fn get_plugin_state(&self, plugin_id: &str) -> Result<PluginState, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            PluginRegistryError::PluginNotFound(plugin_id.to_string())
        })?;
        
        Ok(entry.state)
    }
    
    /// プラグインのエラーメッセージを取得（エラー状態の場合）
    pub fn get_plugin_error(&self, plugin_id: &str) -> Result<Option<String>, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            PluginRegistryError::PluginNotFound(plugin_id.to_string())
        })?;
        
        Ok(entry.error.clone())
    }
    
    /// 登録済みのプラグイン数を取得
    pub fn get_plugin_count(&self) -> Result<usize, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        Ok(plugins.len())
    }
    
    /// 登録済みの全プラグインIDを取得
    pub fn get_all_plugin_ids(&self) -> Result<Vec<String>, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        Ok(plugins.keys().cloned().collect())
    }
    
    /// 特定の状態のプラグインIDを取得
    pub fn get_plugins_by_state(&self, state: PluginState) -> Result<Vec<String>, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        Ok(plugins.iter()
            .filter_map(|(id, entry)| {
                if entry.state == state {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect())
    }
    
    /// プラグインの記述子を取得
    pub fn get_plugin_descriptor(&self, plugin_id: &str) -> Result<PluginDescriptor, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            PluginRegistryError::PluginNotFound(plugin_id.to_string())
        })?;
        
        Ok(entry.plugin.get_descriptor())
    }
    
    /// すべてのプラグイン記述子を取得
    pub fn get_all_plugin_descriptors(&self) -> Result<Vec<PluginDescriptor>, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        Ok(plugins.values()
            .map(|entry| entry.plugin.get_descriptor())
            .collect())
    }
    
    /// ディスカバリーパスを追加（プラグインを検索するディレクトリ）
    pub fn add_discovery_path(&self, path: &str) -> Result<(), PluginRegistryError> {
        let mut paths = self.discovery_paths.lock().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock discovery paths: {}", e))
        })?;
        
        if !paths.contains(&path.to_string()) {
            paths.push(path.to_string());
        }
        
        Ok(())
    }
    
    /// ディスカバリーパスから外部プラグインを検出してロード
    pub fn discover_plugins(&self) -> Result<Vec<String>, PluginRegistryError> {
        let paths = self.discovery_paths.lock().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock discovery paths: {}", e))
        })?;
        
        let mut loaded_plugins = Vec::new();
        
        // 各ディスカバリーパスを処理
        for path in paths.iter() {
            match self.scan_directory_for_plugins(path) {
                Ok(plugins) => {
                    loaded_plugins.extend(plugins);
                }
                Err(e) => {
                    log::warn!("Error scanning directory {}: {}", path, e);
                }
            }
        }
        
        Ok(loaded_plugins)
    }
    
    /// ディレクトリをスキャンしてプラグインを検出
    fn scan_directory_for_plugins(&self, _dir_path: &str) -> Result<Vec<String>, PluginRegistryError> {
        // 注: この実装はプロトタイプ - 実際のプラグイン検出ロジックは
        // アプリケーション固有の要件に応じて拡張する必要があります
        
        // 例えば、指定ディレクトリ内の .so/.dll/.dylib ファイルを検索し、
        // ダイナミックローディングでプラグインを読み込むなど
        
        // 現在の実装では空のリストを返す
        Ok(Vec::new())
    }
    
    /// プラグインを登録解除
    pub fn unregister_plugin(&self, plugin_id: &str) -> Result<(), PluginRegistryError> {
        // プラグインが存在するか確認
        {
            let plugins = self.plugins.read().map_err(|e| {
                PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
            })?;
            
            if !plugins.contains_key(plugin_id) {
                return Err(PluginRegistryError::PluginNotFound(plugin_id.to_string()));
            }
        }
        
        // 登録解除
        {
            let mut plugins = self.plugins.write().map_err(|e| {
                PluginRegistryError::SystemError(format!("Failed to lock plugins registry for writing: {}", e))
            })?;
            
            plugins.remove(plugin_id);
        }
        
        // イベント発行
        let _ = self.event_bus.publish("plugin:unregistered", serde_json::json!({
            "plugin_id": plugin_id,
        }));
        
        log::info!("Plugin unregistered: {}", plugin_id);
        Ok(())
    }
    
    /// プラグインの設定を取得
    pub fn get_plugin_config(&self, plugin_id: &str) -> Result<JsonValue, PluginRegistryError> {
        let plugins = self.plugins.read().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry: {}", e))
        })?;
        
        let entry = plugins.get(plugin_id).ok_or_else(|| {
            PluginRegistryError::PluginNotFound(plugin_id.to_string())
        })?;
        
        entry.plugin.get_config().map_err(|e| {
            PluginRegistryError::OperationError(format!("Failed to get plugin config: {}", e))
        })
    }
    
    /// プラグインの設定を更新
    pub fn update_plugin_config(&self, plugin_id: &str, config: JsonValue) -> Result<(), PluginRegistryError> {
        let mut plugins = self.plugins.write().map_err(|e| {
            PluginRegistryError::SystemError(format!("Failed to lock plugins registry for writing: {}", e))
        })?;
        
        let entry = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginRegistryError::PluginNotFound(plugin_id.to_string())
        })?;
        
        entry.plugin.update_config(config).map_err(|e| {
            PluginRegistryError::OperationError(format!("Failed to update plugin config: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::plugin_trait::MockPlugin;
    
    #[test]
    fn test_plugin_registration() {
        let event_bus = Arc::new(EventBus::new());
        let registry = PluginRegistry::new(Arc::clone(&event_bus));
        
        let plugin = Box::new(MockPlugin::new("test-plugin"));
        assert!(registry.register_plugin(plugin).is_ok());
        
        // 同じIDのプラグインを登録しようとするとエラーになる
        let duplicate_plugin = Box::new(MockPlugin::new("test-plugin"));
        assert!(registry.register_plugin(duplicate_plugin).is_err());
        
        // プラグインの状態確認
        let state = registry.get_plugin_state("test-plugin").unwrap();
        assert_eq!(state, PluginState::Registered);
        
        // プラグイン数の確認
        assert_eq!(registry.get_plugin_count().unwrap(), 1);
    }
    
    #[test]
    fn test_plugin_initialization() {
        let event_bus = Arc::new(EventBus::new());
        let registry = PluginRegistry::new(Arc::clone(&event_bus));
        
        let plugin = Box::new(MockPlugin::new("test-plugin"));
        registry.register_plugin(plugin).unwrap();
        
        // 初期化
        assert!(registry.initialize_plugin("test-plugin").is_ok());
        
        // 状態確認
        let state = registry.get_plugin_state("test-plugin").unwrap();
        assert_eq!(state, PluginState::Initialized);
    }
    
    #[test]
    fn test_unregister_plugin() {
        let event_bus = Arc::new(EventBus::new());
        let registry = PluginRegistry::new(Arc::clone(&event_bus));
        
        let plugin = Box::new(MockPlugin::new("test-plugin"));
        registry.register_plugin(plugin).unwrap();
        
        // 登録解除
        assert!(registry.unregister_plugin("test-plugin").is_ok());
        
        // プラグインが存在しないことを確認
        assert!(registry.get_plugin_state("test-plugin").is_err());
        assert_eq!(registry.get_plugin_count().unwrap(), 0);
    }
    
    #[test]
    fn test_plugin_descriptors() {
        let event_bus = Arc::new(EventBus::new());
        let registry = PluginRegistry::new(Arc::clone(&event_bus));
        
        let plugin1 = Box::new(MockPlugin::new("plugin1"));
        let plugin2 = Box::new(MockPlugin::new("plugin2"));
        
        registry.register_plugin(plugin1).unwrap();
        registry.register_plugin(plugin2).unwrap();
        
        // 個別のプラグイン記述子を取得
        let desc1 = registry.get_plugin_descriptor("plugin1").unwrap();
        assert_eq!(desc1.id, "plugin1");
        
        // 全プラグイン記述子を取得
        let all_descs = registry.get_all_plugin_descriptors().unwrap();
        assert_eq!(all_descs.len(), 2);
    }
}