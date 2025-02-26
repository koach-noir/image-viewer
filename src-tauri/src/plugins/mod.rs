// plugins/mod.rs
// プラグインシステムのエントリポイント

// サブモジュールを公開
pub mod plugin_trait;
pub mod registry;

// プラグインのモジュールを条件付きで含める
#[cfg(feature = "plugin-allviewer")]
pub mod allviewer;

#[cfg(feature = "plugin-findme")]
pub mod findme;

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
/// * `Result<Arc<PluginRegistry>, String>` - 初期化されたプラグインレジストリ、またはエラーメッセージ
pub fn initialize(event_bus: Arc<EventBus>) -> Result<Arc<PluginRegistry>, String> {
    // プラグインレジストリを作成
    let registry = Arc::new(PluginRegistry::new(event_bus));
    
    // 選択されたプラグインを登録
    // 条件付きコンパイルを使用して有効なプラグインのみを登録
    register_enabled_plugins(&registry)?;
    
    // イベントログ
    log::info!("Plugin system initialized");
    
    Ok(registry)
}

/// フィーチャーフラグに基づいて有効なプラグインを登録する
fn register_enabled_plugins(registry: &Arc<PluginRegistry>) -> Result<(), String> {
    // AllViewerプラグイン
    #[cfg(feature = "plugin-allviewer")]
    {
        log::info!("Registering AllViewer plugin");
        let plugin = allviewer::create_plugin();
        registry.register_plugin(plugin)
            .map_err(|e| format!("Failed to register AllViewer plugin: {}", e))?;
    }
    
    // FindMeプラグイン
    #[cfg(feature = "plugin-findme")]
    {
        log::info!("Registering FindMe plugin");
        let plugin = findme::create_plugin();
        registry.register_plugin(plugin)
            .map_err(|e| format!("Failed to register FindMe plugin: {}", e))?;
    }
    
    Ok(())
}

/// フィーチャーフラグの状態を確認するヘルパー関数
/// フロントエンドとの連携に使用
pub fn get_enabled_plugins() -> Vec<String> {
    let mut enabled_plugins = Vec::new();
    
    #[cfg(feature = "plugin-allviewer")]
    enabled_plugins.push("allviewer".to_string());
    
    #[cfg(feature = "plugin-findme")]
    enabled_plugins.push("findme".to_string());
    
    enabled_plugins
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
        // 有効化されているプラグイン数の確認
        // 注: この数はフィーチャーフラグの設定によって変わる
        assert!(registry.get_plugin_count().unwrap() > 0, "プラグインが登録されていない");
        
        // 有効なプラグインの一覧を取得
        let enabled_plugins = get_enabled_plugins();
        println!("Enabled plugins: {:?}", enabled_plugins);
    }
}
