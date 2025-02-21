use std::fs;
use base64::{Engine as _, engine::general_purpose};
use serde::Serialize;
use tauri::AppHandle;

#[derive(Debug, Serialize)]
pub struct ImageData {
    base64: String,
    file_name: String,
}

#[tauri::command]
async fn load_image(path: String, _app: AppHandle) -> Result<ImageData, String> {
    // 直接パスを使用
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
