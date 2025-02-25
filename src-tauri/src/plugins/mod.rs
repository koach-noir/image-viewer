// plugins/mod.rs
// プラグインシステムのエントリポイント

// サブモジュールを公開
pub mod plugin_trait;
pub mod registry;

// すべてのサブモジュールから選択的に再エクスポート
pub use plugin_trait::{Plugin, PluginDescriptor, PluginResult};
pub use registry::{PluginRegistry, PluginRegistryError};

use crate::core::EventBus;
use std::sync::Arc;

/// プラグインシステムの初期化を行う
/// 
/// # 引数
/// 
/// * `event_bus` - システム全体で共有されるイベントバス
/// 
/// # 戻り値
/// 
/// * `Result<PluginRegistry, String>` - 初期化されたプラグインレジストリ、またはエラーメッセージ
pub fn initialize(event_bus: Arc<EventBus>) -> Result<Arc<PluginRegistry>, String> {
    // プラグインレジストリを作成
    let registry = Arc::new(PluginRegistry::new(event_bus));
    
    // 組み込みプラグインの登録処理をここに追加（必要に応じて）
    
    // イベントログ
    log::info!("Plugin system initialized");
    
    Ok(registry)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::core::EventBus;
    
    /// プラグインシステムの初期化をテスト
    #[test]
    fn test_initialize() {
        let event_bus = Arc::new(EventBus::new());
        let registry_result = initialize(event_bus);
        
        assert!(registry_result.is_ok(), "プラグインシステムの初期化に失敗: {:?}", registry_result.err());
        
        let registry = registry_result.unwrap();
        assert_eq!(registry.get_plugin_count().unwrap(), 0, "初期状態ではプラグインがゼロのはず");
    }
}
