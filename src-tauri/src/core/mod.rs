// core/mod.rs
// コアモジュールのエントリポイント

pub mod resource_manager;
pub mod image_collection;
pub mod plugin_manager;
pub mod event_bus;

// コアモジュールを一括でエクスポート
pub use resource_manager::ResourceManager;
pub use image_collection::{ImageCollection, ImageData, ImageMetadata};
pub use plugin_manager::PluginManager;
pub use event_bus::EventBus;

/// コアシステムの初期化
pub fn initialize() -> Result<(), String> {
    // 必要に応じてコアコンポーネントの初期化処理を追加
    log::info!("Core system initialized");
    Ok(())
}
