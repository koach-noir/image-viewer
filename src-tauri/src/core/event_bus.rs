// event_bus.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue};

/// イベントのペイロードタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// イベントのタイプ/名前
    pub event_type: String,
    /// イベントデータ（任意のJSON）
    pub data: JsonValue,
    /// イベントの送信元ID（オプション）
    pub source: Option<String>,
    /// イベントの送信先ID（オプション、指定がなければブロードキャスト）
    pub target: Option<String>,
}

/// イベントハンドラー関数タイプ
pub type EventHandler = Box<dyn Fn(EventPayload) -> Result<(), String> + Send + Sync>;

/// イベントバスの実装
#[derive(Debug, Default)]
pub struct EventBus {
    /// イベントタイプごとのハンドラー
    handlers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    /// コンポーネントごとのハンドラー
    component_handlers: Arc<Mutex<HashMap<String, HashMap<String, Vec<EventHandler>>>>>,
}

impl EventBus {
    /// 新しいEventBusインスタンスを作成
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            component_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// イベントハンドラーを登録
    pub fn subscribe<F>(&self, event_type: &str, handler: F) -> Result<(), String>
    where
        F: Fn(EventPayload) -> Result<(), String> + Send + Sync + 'static,
    {
        match self.handlers.lock() {
            Ok(mut handlers) => {
                let entry = handlers.entry(event_type.to_string()).or_insert_with(Vec::new);
                entry.push(Box::new(handler));
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock handlers: {}", e)),
        }
    }

    /// 特定のコンポーネントにイベントハンドラーを登録
    pub fn subscribe_component<F>(&self, component_id: &str, event_type: &str, handler: F) -> Result<(), String>
    where
        F: Fn(EventPayload) -> Result<(), String> + Send + Sync + 'static,
    {
        match self.component_handlers.lock() {
            Ok(mut comp_handlers) => {
                let component_entry = comp_handlers.entry(component_id.to_string()).or_insert_with(HashMap::new);
                let entry = component_entry.entry(event_type.to_string()).or_insert_with(Vec::new);
                entry.push(Box::new(handler));
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock component handlers: {}", e)),
        }
    }

    /// イベントを発行（グローバル）
    pub fn publish(&self, event_type: &str, data: JsonValue) -> Result<(), String> {
        let payload = EventPayload {
            event_type: event_type.to_string(),
            data,
            source: None,
            target: None,
        };
        self.dispatch_event(payload)
    }

    /// コンポーネントからイベントを発行
    pub fn publish_from(&self, source_id: &str, event_type: &str, data: JsonValue) -> Result<(), String> {
        let payload = EventPayload {
            event_type: event_type.to_string(),
            data,
            source: Some(source_id.to_string()),
            target: None,
        };
        self.dispatch_event(payload)
    }

    /// 特定のコンポーネントへイベントを発行
    pub fn publish_to(&self, target_id: &str, event_type: &str, data: JsonValue) -> Result<(), String> {
        let payload = EventPayload {
            event_type: event_type.to_string(),
            data,
            source: None,
            target: Some(target_id.to_string()),
        };
        self.dispatch_event(payload)
    }

    /// コンポーネント間でイベントを発行
    pub fn publish_between(&self, source_id: &str, target_id: &str, event_type: &str, data: JsonValue) -> Result<(), String> {
        let payload = EventPayload {
            event_type: event_type.to_string(),
            data,
            source: Some(source_id.to_string()),
            target: Some(target_id.to_string()),
        };
        self.dispatch_event(payload)
    }

    /// イベントをディスパッチ
    fn dispatch_event(&self, payload: EventPayload) -> Result<(), String> {
        let mut dispatch_errors = Vec::new();

        // グローバルハンドラーに配信
        if let Ok(handlers) = self.handlers.lock() {
            if let Some(event_handlers) = handlers.get(&payload.event_type) {
                for handler in event_handlers {
                    if let Err(e) = handler(payload.clone()) {
                        dispatch_errors.push(format!("Global handler error: {}", e));
                    }
                }
            }
        } else {
            return Err("Failed to lock handlers".to_string());
        }

        // 特定のターゲットがある場合、そのコンポーネントのハンドラーに配信
        if let Some(target_id) = &payload.target {
            if let Ok(comp_handlers) = self.component_handlers.lock() {
                if let Some(component_handlers) = comp_handlers.get(target_id) {
                    if let Some(event_handlers) = component_handlers.get(&payload.event_type) {
                        for handler in event_handlers {
                            if let Err(e) = handler(payload.clone()) {
                                dispatch_errors.push(format!("Component handler error for {}: {}", target_id, e));
                            }
                        }
                    }
                }
            } else {
                return Err("Failed to lock component handlers".to_string());
            }
        }

        if dispatch_errors.is_empty() {
            Ok(())
        } else {
            Err(dispatch_errors.join("; "))
        }
    }

    /// コンポーネントの全ハンドラーを解除
    pub fn unsubscribe_component(&self, component_id: &str) -> Result<(), String> {
        match self.component_handlers.lock() {
            Ok(mut comp_handlers) => {
                comp_handlers.remove(component_id);
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock component handlers: {}", e)),
        }
    }

    /// 全てのハンドラーをクリア
    pub fn clear_all_handlers(&self) -> Result<(), String> {
        match self.handlers.lock() {
            Ok(mut handlers) => {
                handlers.clear();
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock handlers: {}", e)),
        }?;

        match self.component_handlers.lock() {
            Ok(mut comp_handlers) => {
                comp_handlers.clear();
                Ok(())
            },
            Err(e) => Err(format!("Failed to lock component handlers: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_subscribe_and_publish() {
        let event_bus = EventBus::new();
        let received = Arc::new(Mutex::new(false));
        let received_clone = Arc::clone(&received);

        // ハンドラーを登録
        event_bus.subscribe("test_event", move |payload| {
            let mut received = received_clone.lock().unwrap();
            *received = true;
            assert_eq!(payload.event_type, "test_event");
            assert_eq!(payload.data.as_str().unwrap(), "test_data");
            Ok(())
        }).unwrap();

        // イベント発行
        event_bus.publish("test_event", json!("test_data")).unwrap();

        // 受信確認
        assert!(*received.lock().unwrap());
    }

    #[test]
    fn test_component_specific_events() {
        let event_bus = EventBus::new();
        let component_a_received = Arc::new(Mutex::new(false));
        let component_a_clone = Arc::clone(&component_a_received);

        // コンポーネント固有のハンドラー登録
        event_bus.subscribe_component("component_a", "component_event", move |payload| {
            let mut received = component_a_clone.lock().unwrap();
            *received = true;
            assert_eq!(payload.target.unwrap(), "component_a");
            Ok(())
        }).unwrap();

        // 特定コンポーネントへのイベント発行
        event_bus.publish_to("component_a", "component_event", json!({})).unwrap();

        // 受信確認
        assert!(*component_a_received.lock().unwrap());
    }
}
