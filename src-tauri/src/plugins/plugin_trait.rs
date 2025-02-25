// plugin_trait.rs
// プラグインのインターフェース定義

use std::any::Any;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::core::plugin_context::PluginContext; // 共通のPluginContextをインポート

/// プラグイン記述子 - プラグインの基本情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    /// プラグインの一意識別子
    pub id: String,
    /// プラグインの表示名
    pub name: String,
    /// プラグインのバージョン
    pub version: String,
    /// プラグインの説明
    pub description: String,
    /// 作者情報
    pub author: String,
}

/// プラグイン結果 - プラグイン操作の結果を表す型
pub type PluginResult<T> = Result<T, String>;

/// プラグイントレイト - すべてのプラグインが実装すべきインターフェース
pub trait Plugin: Send + Sync {
    /// プラグインのIDを取得
    fn get_id(&self) -> String;
    
    /// プラグインの基本情報を取得
    fn get_descriptor(&self) -> PluginDescriptor;
    
    /// プラグインを初期化
    fn initialize(&mut self, context: Arc<PluginContext>) -> PluginResult<()>;
    
    /// プラグインを有効化
    fn activate(&mut self) -> PluginResult<()>;
    
    /// プラグインを無効化
    fn deactivate(&mut self) -> PluginResult<()>;
    
    /// プラグインの設定を取得
    fn get_config(&self) -> PluginResult<JsonValue> {
        // デフォルト実装では空のオブジェクトを返す
        Ok(serde_json::json!({}))
    }
    
    /// プラグインの設定を更新
    fn update_config(&mut self, _config: JsonValue) -> PluginResult<()> {
        // デフォルト実装では何もしない
        Ok(())
    }
    
    /// フロントエンドのUIコードを取得（オプション）
    fn get_frontend_code(&self) -> Option<String> {
        None
    }
    
    /// プラグインが提供するAPIハンドラを取得
    fn get_api_handlers(&self) -> Vec<(&'static str, Box<dyn Fn(JsonValue) -> PluginResult<JsonValue> + Send + Sync>)> {
        Vec::new()
    }
    
    /// プラグインを型変換（Any経由）
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(test)]
pub struct MockPlugin {
    id: String,
    descriptor: PluginDescriptor,
    initialized: bool,
    active: bool,
}

#[cfg(test)]
impl MockPlugin {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            descriptor: PluginDescriptor {
                id: id.to_string(),
                name: format!("Mock Plugin {}", id),
                version: "1.0.0".to_string(),
                description: "A mock plugin for testing".to_string(),
                author: "Test Author".to_string(),
            },
            initialized: false,
            active: false,
        }
    }
}

#[cfg(test)]
impl Plugin for MockPlugin {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    
    fn get_descriptor(&self) -> PluginDescriptor {
        self.descriptor.clone()
    }
    
    fn initialize(&mut self, _context: Arc<PluginContext>) -> PluginResult<()> {
        self.initialized = true;
        Ok(())
    }
    
    fn activate(&mut self) -> PluginResult<()> {
        if !self.initialized {
            return Err("Plugin not initialized".to_string());
        }
        self.active = true;
        Ok(())
    }
    
    fn deactivate(&mut self) -> PluginResult<()> {
        self.active = false;
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EventBus;
    
    #[test]
    fn test_mock_plugin() {
        let event_bus = Arc::new(EventBus::new());
        let context = Arc::new(PluginContext::new(event_bus));
        
        let mut plugin = MockPlugin::new("test-plugin");
        
        // 初期状態確認
        assert_eq!(plugin.get_id(), "test-plugin");
        assert!(!plugin.initialized);
        assert!(!plugin.active);
        
        // 初期化
        let init_result = plugin.initialize(context);
        assert!(init_result.is_ok());
        assert!(plugin.initialized);
        
        // 有効化
        let activate_result = plugin.activate();
        assert!(activate_result.is_ok());
        assert!(plugin.active);
        
        // 無効化
        let deactivate_result = plugin.deactivate();
        assert!(deactivate_result.is_ok());
        assert!(!plugin.active);
    }
}