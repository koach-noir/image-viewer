use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::image_collection::{ImageCollection, ImageMetadata};

/// リソースフィルタ - 対象と除外パスのセット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFilter {
    /// 対象となるパスリスト (ディレクトリまたはファイル)
    pub include: Vec<String>,
    /// 除外するパスリスト (ディレクトリまたはファイル)
    pub exclude: Vec<String>,
}

/// リソース設定 - 識別子、名前、フィルタ情報を含む
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// 設定の一意識別子
    pub id: String,
    /// 設定の表示名
    pub name: String,
    /// リソースフィルタ
    pub filters: ResourceFilter,
}

/// パス展開結果
#[derive(Debug, Serialize)]
pub struct PathResolutionResult {
    /// 解決されたパスリスト
    pub paths: Vec<String>,
    /// 見つかったファイル数
    pub count: usize,
}

/// 画像ファイル拡張子判定
fn is_image_file(file_name: &str) -> bool {
    let lower_case = file_name.to_lowercase();
    lower_case.ends_with(".jpg") || 
    lower_case.ends_with(".jpeg") || 
    lower_case.ends_with(".png") || 
    lower_case.ends_with(".gif") || 
    lower_case.ends_with(".bmp") || 
    lower_case.ends_with(".webp")
}

/// リソース管理クラス
#[derive(Debug, Default)]
pub struct ResourceManager {
    /// 設定キャッシュ (設定ID -> 設定)
    config_cache: Arc<Mutex<HashMap<String, ResourceConfig>>>,
    /// パス解決キャッシュ (設定ID -> 解決済みパスリスト)
    path_cache: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ResourceManager {
    /// 新しいResourceManagerインスタンスを作成
    pub fn new() -> Self {
        Self {
            config_cache: Arc::new(Mutex::new(HashMap::new())),
            path_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 指定されたJSONパスから設定をロード
    pub fn load_config(&self, path: &str) -> Result<ResourceConfig, String> {
        let file_content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))
    }

    /// 設定をJSONファイルに保存
    pub fn save_config(&self, config: &ResourceConfig, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        // キャッシュに保存
        if let Ok(mut cache) = self.config_cache.lock() {
            cache.insert(config.id.clone(), config.clone());
        }
        
        Ok(())
    }

    /// 指定されたパスを展開し、すべての画像ファイルを取得
    fn resolve_path(&self, path: &str, exclude_paths: &[String]) -> Result<Vec<String>, String> {
        let path_obj = Path::new(path);
        
        if !path_obj.exists() {
            return Err(format!("Path does not exist: {}", path));
        }
        
        let mut result = Vec::new();
        
        // ファイルの場合は直接追加
        if path_obj.is_file() {
            if let Some(file_name) = path_obj.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if is_image_file(file_name_str) {
                        result.push(path.to_string());
                    }
                }
            }
            return Ok(result);
        }
        
        // ディレクトリの場合は再帰的に探索
        if path_obj.is_dir() {
            self.scan_directory(path_obj, &mut result, exclude_paths)?;
        }
        
        Ok(result)
    }
    
    /// ディレクトリを再帰的に走査して画像ファイルを検索
    fn scan_directory(&self, dir: &Path, result: &mut Vec<String>, exclude_paths: &[String]) -> Result<(), String> {
        // 除外パスチェック
        if let Some(dir_str) = dir.to_str() {
            if exclude_paths.iter().any(|exclude| dir_str.starts_with(exclude)) {
                return Ok(());
            }
        }
        
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    log::warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            
            let path = entry.path();
            
            // 除外パスチェック
            if let Some(path_str) = path.to_str() {
                if exclude_paths.iter().any(|exclude| path_str.starts_with(exclude)) {
                    continue;
                }
            }
            
            if path.is_dir() {
                self.scan_directory(&path, result, exclude_paths)?;
            } else if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        if is_image_file(file_name_str) {
                            if let Some(path_str) = path.to_str() {
                                result.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 設定に基づいてリソースを内部で解決する関数
    pub async fn internal_resolve_resources(&self, config: ResourceConfig) -> Result<PathResolutionResult, String> {
        // キャッシュに設定を保存
        if let Ok(mut cache) = self.config_cache.lock() {
            cache.insert(config.id.clone(), config.clone());
        }
        
        // パスキャッシュをチェック
        if let Ok(cache) = self.path_cache.lock() {
            if let Some(paths) = cache.get(&config.id) {
                return Ok(PathResolutionResult {
                    paths: paths.clone(),
                    count: paths.len(),
                });
            }
        }
        
        let mut all_paths = Vec::new();
        
        // include パスを処理
        for include_path in &config.filters.include {
            let paths = self.resolve_path(include_path, &config.filters.exclude)?;
            all_paths.extend(paths);
        }
        
        // 重複を除去
        all_paths.sort();
        all_paths.dedup();
        
        // キャッシュに保存
        if let Ok(mut cache) = self.path_cache.lock() {
            cache.insert(config.id.clone(), all_paths.clone());
        }
        
        Ok(PathResolutionResult {
            paths: all_paths.clone(),
            count: all_paths.len(),
        })
    }

    /// パスリストから内部で画像コレクションを作成する関数
    pub async fn internal_load_images_from_paths(&self, paths: Vec<String>) -> Result<ImageCollection, String> {
        let mut metadata_list = Vec::new();
        
        for path in paths {
            let path_obj = PathBuf::from(&path);
            
            if !path_obj.exists() || !path_obj.is_file() {
                log::warn!("Skipping invalid path: {}", path);
                continue;
            }
            
            let file_name = path_obj.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            let file_size = match fs::metadata(&path_obj) {
                Ok(metadata) => metadata.len(),
                Err(_) => 0,
            };
            
            metadata_list.push(ImageMetadata {
                path: path.clone(),
                file_name,
                file_size,
                dimensions: None, // 初期段階では次元情報は取得しない
                date_created: None,
                date_modified: None,
            });
        }
        
        Ok(ImageCollection::new(metadata_list))
    }

    /// 設定IDに基づいて内部で画像コレクションを直接ロードする関数
    pub async fn internal_load_images_from_config(&self, config_id: String) -> Result<ImageCollection, String> {
        let config = {
            if let Ok(cache) = self.config_cache.lock() {
                if let Some(config) = cache.get(&config_id) {
                    config.clone()
                } else {
                    return Err(format!("Config not found for ID: {}", config_id));
                }
            } else {
                return Err("Failed to access config cache".to_string());
            }
        };
        
        let result = self.internal_resolve_resources(config).await?;
        self.internal_load_images_from_paths(result.paths).await
    }

    /// キャッシュをクリア
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.path_cache.lock() {
            cache.clear();
        }
    }

    /// 特定の設定IDのキャッシュをクリア
    pub fn clear_config_cache(&self, config_id: &str) {
        if let Ok(mut cache) = self.path_cache.lock() {
            cache.remove(config_id);
        }
    }
}

// 以下、Tauriコマンド関数（実装ブロックの外に移動）

/// 設定に基づいてリソースを解決するTauriコマンド
#[tauri::command]
pub async fn resolve_resources(config: ResourceConfig, resource_manager: tauri::State<'_, Arc<ResourceManager>>) -> Result<PathResolutionResult, String> {
    resource_manager.internal_resolve_resources(config).await
}

/// パスリストから画像コレクションを作成するTauriコマンド
#[tauri::command]
pub async fn load_images_from_paths(paths: Vec<String>, resource_manager: tauri::State<'_, Arc<ResourceManager>>) -> Result<ImageCollection, String> {
    resource_manager.internal_load_images_from_paths(paths).await
}

/// 設定IDに基づいて画像コレクションを直接ロードするTauriコマンド
#[tauri::command]
pub async fn load_images_from_config(config_id: String, resource_manager: tauri::State<'_, Arc<ResourceManager>>) -> Result<ImageCollection, String> {
    resource_manager.internal_load_images_from_config(config_id).await
}