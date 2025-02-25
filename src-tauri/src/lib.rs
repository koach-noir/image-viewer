use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use tauri::AppHandle;
use tauri::Manager;

// コアモジュールのエクスポート
pub mod core;
// プラグインシステムのエクスポート
pub mod plugins;
// ユーティリティ関数（将来的に実装予定）
// pub mod utils;

// イベントバスとプラグインマネージャーのインスタンスを保持するグローバル状態
struct AppState {
    event_bus: Arc<core::event_bus::EventBus>,
    plugin_manager: Arc<core::plugin_manager::PluginManager>,
    resource_manager: Arc<core::resource_manager::ResourceManager>,
}

#[derive(Debug, Serialize)]
pub struct ImageData {
    base64: String,
    file_name: String,
}

#[derive(Debug, Serialize)]
pub struct DirectoryContent {
    images: Vec<String>,
    current_index: usize,
}

// 対応している画像拡張子
fn is_image_file(file_name: &str) -> bool {
    let lower_case = file_name.to_lowercase();
    lower_case.ends_with(".jpg") || 
    lower_case.ends_with(".jpeg") || 
    lower_case.ends_with(".png") || 
    lower_case.ends_with(".gif") || 
    lower_case.ends_with(".bmp") || 
    lower_case.ends_with(".webp")
}

#[tauri::command]
async fn load_image(path: String, _app: AppHandle) -> Result<ImageData, String> {
    let file_path = std::path::Path::new(&path);

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    match fs::read(file_path) {
        Ok(bytes) => {
            let base64 = general_purpose::STANDARD.encode(&bytes);
            Ok(ImageData {
                base64,
                file_name,
            })
        }
        Err(e) => Err(format!("Failed to read file: {}", e))
    }
}

#[tauri::command]
async fn get_directory_images(dir_path: String) -> Result<DirectoryContent, String> {
    let path = Path::new(&dir_path);
    
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", dir_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", dir_path));
    }
    
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    };
    
    let mut image_paths = Vec::new();
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                log::warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };
        
        let file_path = entry.path();
        
        // ファイルのみを対象とし、ディレクトリは無視
        if file_path.is_file() {
            if let Some(file_name) = file_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if is_image_file(file_name_str) {
                        if let Some(path_str) = file_path.to_str() {
                            image_paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // ファイル名でソート
    image_paths.sort();
    
    Ok(DirectoryContent {
        images: image_paths,
        current_index: 0,
    })
}

// プラグインシステムコマンド
#[tauri::command]
async fn load_plugin(path: String, app_handle: AppHandle) -> Result<String, String> {
    let state = app_handle.state::<AppState>();
    let plugin_manager = &state.plugin_manager;
    
    // プラグインのロード処理（実際の実装はプラグインシステムによる）
    log::info!("Loading plugin from path: {}", path);
    
    // TODO: 実際のプラグインロード処理を実装
    
    Ok("Plugin loaded successfully".to_string())
}

// リソース解決コマンド
#[tauri::command]
async fn resolve_resources(
    config: core::resource_manager::ResourceConfig,
    app_handle: AppHandle
) -> Result<core::resource_manager::PathResolutionResult, String> {
    let state = app_handle.state::<AppState>();
    let resource_manager = &state.resource_manager;
    
    resource_manager.resolve_resources(config).await
}

// 画像コレクション読み込みコマンド
#[tauri::command]
async fn load_images_from_paths(
    paths: Vec<String>,
    app_handle: AppHandle
) -> Result<core::image_collection::ImageCollection, String> {
    let state = app_handle.state::<AppState>();
    let resource_manager = &state.resource_manager;
    
    resource_manager.load_images_from_paths(paths).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // イベントバスの作成
    let event_bus = Arc::new(core::event_bus::EventBus::new());
    
    // プラグインマネージャーの作成
    let plugin_manager = Arc::new(core::plugin_manager::PluginManager::new(Arc::clone(&event_bus)));
    
    // リソースマネージャーの作成
    let resource_manager = Arc::new(core::resource_manager::ResourceManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            event_bus,
            plugin_manager,
            resource_manager,
        })
        .invoke_handler(tauri::generate_handler![
            load_image,
            get_directory_images,
            load_plugin,
            resolve_resources,
            load_images_from_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}