// plugin_context.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::any::Any;

use crate::core::event_bus::EventBus;

/// プラグインコンテキスト - プラグインに提供される機能
/// コアシステムとプラグインの間の共通インターフェース
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// イベントバス
    pub event_bus: Arc<EventBus>,
    /// 共有データストア
    pub shared_data: Arc<Mutex<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl PluginContext {
    /// 新しいPluginContextインスタンスを作成
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            shared_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 共有データを設定
    pub fn set_shared_data<T: 'static + Send + Sync>(&self, key: &str, value: T) -> Result<(), String> {
        match self.shared_data.lock() {
            Ok(mut data) => {
                data.insert(key.to_string(), Box::new(value));
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock shared data: {}", e)),
        }
    }

    /// 共有データを取得
    pub fn get_shared_data<T: 'static + Clone>(&self, key: &str) -> Result<Option<T>, String> {
        match self.shared_data.lock() {
            Ok(data) => {
                if let Some(value) = data.get(key) {
                    if let Some(typed_value) = value.downcast_ref::<T>() {
                        Ok(Some(typed_value.clone()))
                    } else {
                        Err(format!("Type mismatch for key: {}", key))
                    }
                } else {
                    Ok(None)
                }
            },
            Err(e) => Err(format!("Failed to lock shared data: {}", e)),
        }
    }
}