use std::fs;
use std::path::{Path, PathBuf};
use base64::{Engine as _, engine::general_purpose};
use serde::Serialize;
use tauri::AppHandle;

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
                println!("Warning: Failed to read directory entry: {}", e);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_image,
            get_directory_images,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
