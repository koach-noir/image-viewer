// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use image_viewer_lib::core::{self, event_bus::EventBus, plugin_manager::PluginManager};
use image_viewer_lib::plugins;

fn main() {
    // ロガーの初期化
    env_logger::init();
    
    // コアシステムの初期化
    match core::initialize() {
        Ok(_) => log::info!("Core system initialized successfully"),
        Err(e) => log::error!("Failed to initialize core system: {}", e),
    }
    
    // イベントバスの作成
    let event_bus = Arc::new(EventBus::new());
    
    // プラグインマネージャーの初期化
    let plugin_manager = PluginManager::new(Arc::clone(&event_bus));
    
    // プラグインシステムの初期化
    match plugins::initialize(Arc::clone(&event_bus)) {
        Ok(registry) => {
            log::info!("Plugin system initialized successfully");
            
            // 必要に応じて組み込みプラグインのロード処理をここに追加
            
        },
        Err(e) => log::error!("Failed to initialize plugin system: {}", e),
    }
    
    // Tauriアプリケーションの起動
    image_viewer_lib::run();
}