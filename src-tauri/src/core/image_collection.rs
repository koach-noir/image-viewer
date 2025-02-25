use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 画像メタデータ構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// 画像ファイルの絶対パス
    pub path: String,
    /// ファイル名
    pub file_name: String,
    /// ファイルサイズ（バイト）
    pub file_size: u64,
    /// 画像の寸法（幅 x 高さ）- オプショナル
    pub dimensions: Option<(u32, u32)>,
    /// 作成日時 - オプショナル
    pub date_created: Option<String>,
    /// 更新日時 - オプショナル
    pub date_modified: Option<String>,
}

/// 画像データ構造体（Base64エンコード済み）
#[derive(Debug, Clone, Serialize)]
pub struct ImageData {
    /// Base64エンコードされた画像データ
    pub base64: String,
    /// ファイル名
    pub file_name: String,
    /// メタデータ
    pub metadata: ImageMetadata,
}

/// 画像コレクション構造体
#[derive(Debug, Clone, Serialize)]
pub struct ImageCollection {
    /// 画像メタデータのリスト
    metadata_list: Vec<ImageMetadata>,
    /// キャッシュされた画像データ
    #[serde(skip)]
    image_cache: Arc<Mutex<Vec<Option<ImageData>>>>,
}

impl ImageCollection {
    /// 新しい ImageCollection インスタンスを作成
    pub fn new(metadata_list: Vec<ImageMetadata>) -> Self {
        let cache_size = metadata_list.len();
        let image_cache = Arc::new(Mutex::new(vec![None; cache_size]));
        
        Self {
            metadata_list,
            image_cache,
        }
    }
    
    /// すべての画像メタデータを取得
    pub fn get_all_metadata(&self) -> Vec<ImageMetadata> {
        self.metadata_list.clone()
    }
    
    /// コレクション内の画像数を取得
    pub fn len(&self) -> usize {
        self.metadata_list.len()
    }

    /// コレクションが空かどうかを確認
    pub fn is_empty(&self) -> bool {
        self.metadata_list.is_empty()
    }
    
    /// インデックスで特定の画像のメタデータを取得
    pub fn get_metadata_at(&self, index: usize) -> Option<ImageMetadata> {
        self.metadata_list.get(index).cloned()
    }
    
    /// パスで特定の画像のメタデータを取得
    pub fn get_metadata_by_path(&self, path: &str) -> Option<ImageMetadata> {
        self.metadata_list.iter()
            .find(|metadata| metadata.path == path)
            .cloned()
    }
    
    /// インデックスで特定の画像を読み込み
    pub fn load_image_at(&self, index: usize) -> Result<ImageData, String> {
        // 範囲チェック
        if index >= self.metadata_list.len() {
            return Err(format!("Index out of bounds: {}", index));
        }
        
        // キャッシュチェック
        if let Ok(mut cache) = self.image_cache.lock() {
            if let Some(Some(image_data)) = cache.get(index) {
                return Ok(image_data.clone());
            }
            
            // キャッシュにない場合は読み込み
            let metadata = &self.metadata_list[index];
            let image_data = self.load_image_from_path(&metadata.path)?;
            
            // キャッシュに保存
            if let Some(slot) = cache.get_mut(index) {
                *slot = Some(image_data.clone());
            }
            
            Ok(image_data)
        } else {
            Err("Failed to access image cache".to_string())
        }
    }
    
    /// パスから画像を読み込み
    fn load_image_from_path(&self, path: &str) -> Result<ImageData, String> {
        let path_obj = Path::new(path);
        
        let metadata = self.get_metadata_by_path(path)
            .ok_or_else(|| format!("Metadata not found for path: {}", path))?;
        
        match fs::read(path_obj) {
            Ok(bytes) => {
                let base64 = general_purpose::STANDARD.encode(&bytes);
                Ok(ImageData {
                    base64,
                    file_name: metadata.file_name.clone(),
                    metadata: metadata.clone(),
                })
            },
            Err(e) => Err(format!("Failed to read file: {}", e))
        }
    }
    
    /// 指定された数のランダムな画像を取得
    pub fn get_random_images(&self, count: usize) -> Result<Vec<ImageData>, String> {
        if self.is_empty() {
            return Err("Collection is empty".to_string());
        }
        
        let mut rng = thread_rng();
        let mut indices: Vec<usize> = (0..self.len()).collect();
        indices.shuffle(&mut rng);
        
        let count = std::cmp::min(count, self.len());
        let selected_indices = &indices[0..count];
        
        let mut result = Vec::with_capacity(count);
        for &index in selected_indices {
            match self.load_image_at(index) {
                Ok(image_data) => result.push(image_data),
                Err(e) => log::warn!("Failed to load image at index {}: {}", index, e),
            }
        }
        
        Ok(result)
    }
    
    /// 条件に基づいてフィルタリングされた新しいコレクションを作成
    pub fn filter<F>(&self, predicate: F) -> Self 
    where 
        F: FnMut(&ImageMetadata) -> bool + Clone
    {
        let filtered_metadata = self.metadata_list.iter()
            .cloned()
            .filter(predicate)
            .collect();
        
        Self::new(filtered_metadata)
    }
    
    /// 比較関数に基づいてソートされた新しいコレクションを作成
    pub fn sort<F>(&self, mut compare_fn: F) -> Self 
    where 
        F: FnMut(&ImageMetadata, &ImageMetadata) -> std::cmp::Ordering
    {
        let mut sorted_metadata = self.metadata_list.clone();
        sorted_metadata.sort_by(|a, b| compare_fn(a, b));
        
        Self::new(sorted_metadata)
    }
    
    /// キャッシュをクリア
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.image_cache.lock() {
            for slot in cache.iter_mut() {
                *slot = None;
            }
        }
    }

    /// コレクションのダイジェスト情報を取得
    pub fn get_digest(&self) -> ImageCollectionDigest {
        ImageCollectionDigest {
            total_images: self.len(),
            total_size_bytes: self.metadata_list.iter()
                .map(|meta| meta.file_size)
                .sum(),
        }
    }
}

/// 画像コレクションのダイジェスト情報
#[derive(Debug, Clone, Serialize)]
pub struct ImageCollectionDigest {
    /// 総画像数
    pub total_images: usize,
    /// 総サイズ（バイト）
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collection() {
        let metadata = vec![
            ImageMetadata {
                path: "/path/to/image1.jpg".to_string(),
                file_name: "image1.jpg".to_string(),
                file_size: 1024,
                dimensions: Some((800, 600)),
                date_created: None,
                date_modified: None,
            },
            ImageMetadata {
                path: "/path/to/image2.png".to_string(),
                file_name: "image2.png".to_string(),
                file_size: 2048,
                dimensions: Some((1024, 768)),
                date_created: None,
                date_modified: None,
            },
        ];
        
        let collection = ImageCollection::new(metadata);
        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());
    }

    #[test]
    fn test_get_metadata() {
        let metadata = vec![
            ImageMetadata {
                path: "/path/to/image1.jpg".to_string(),
                file_name: "image1.jpg".to_string(),
                file_size: 1024,
                dimensions: Some((800, 600)),
                date_created: None,
                date_modified: None,
            },
        ];
        
        let collection = ImageCollection::new(metadata);
        
        let retrieved = collection.get_metadata_at(0).unwrap();
        assert_eq!(retrieved.file_name, "image1.jpg");
        
        let retrieved_by_path = collection.get_metadata_by_path("/path/to/image1.jpg").unwrap();
        assert_eq!(retrieved_by_path.file_size, 1024);
        
        assert!(collection.get_metadata_at(1).is_none());
        assert!(collection.get_metadata_by_path("/nonexistent/path.jpg").is_none());
    }

    #[test]
    fn test_filter_and_sort() {
        let metadata = vec![
            ImageMetadata {
                path: "/path/to/image1.jpg".to_string(),
                file_name: "image1.jpg".to_string(),
                file_size: 1024,
                dimensions: Some((800, 600)),
                date_created: None,
                date_modified: None,
            },
            ImageMetadata {
                path: "/path/to/image2.png".to_string(),
                file_name: "image2.png".to_string(),
                file_size: 2048,
                dimensions: Some((1024, 768)),
                date_created: None,
                date_modified: None,
            },
            ImageMetadata {
                path: "/path/to/image3.gif".to_string(),
                file_name: "image3.gif".to_string(),
                file_size: 512,
                dimensions: Some((400, 300)),
                date_created: None,
                date_modified: None,
            },
        ];
        
        let collection = ImageCollection::new(metadata);
        
        // フィルタリングテスト
        let jpg_only = collection.filter(|meta| meta.file_name.ends_with(".jpg"));
        assert_eq!(jpg_only.len(), 1);
        assert_eq!(jpg_only.get_metadata_at(0).unwrap().file_name, "image1.jpg");
        
        // ソートテスト
        let sorted_by_size = collection.sort(|a, b| a.file_size.cmp(&b.file_size));
        assert_eq!(sorted_by_size.get_metadata_at(0).unwrap().file_size, 512);
        assert_eq!(sorted_by_size.get_metadata_at(2).unwrap().file_size, 2048);
    }
}
