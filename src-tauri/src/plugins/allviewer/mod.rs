// allviewer/mod.rs - AllViewerプラグイン（大量画像表示アプリ）の実装

use std::sync::{Arc, Mutex};
use serde_json::{json, Value as JsonValue};
use crate::core::plugin_context::PluginContext;
use crate::plugins::plugin_trait::{Plugin, PluginDescriptor, PluginResult};
use crate::core::resource_manager::ResourceConfig;
use std::time::{SystemTime, UNIX_EPOCH};

// UIモジュールをインポート
pub mod ui;

// プラグインの状態
#[derive(Debug, Default)]
struct AllViewerState {
    // 表示モード
    view_mode: String,
    // サムネイルサイズ
    thumbnail_size: u32,
    // グリッドモードでラベルを表示するかどうか
    show_labels: bool,
    // 現在選択されている画像のインデックス
    current_index: usize,
    // 現在選択されているディレクトリパス
    current_directory: Option<String>,
    // リソース設定
    resource_config: Option<ResourceConfig>,
}

// AllViewerプラグインの実装
pub struct AllViewerPlugin {
    // プラグイン記述子
    descriptor: PluginDescriptor,
    // プラグインの状態
    state: Arc<Mutex<AllViewerState>>,
    // プラグインコンテキスト
    context: Option<Arc<PluginContext>>,
}

// プラグインインスタンスを作成する関数
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(AllViewerPlugin::new())
}

impl AllViewerPlugin {
    // 新しいインスタンスを作成
    pub fn new() -> Self {
        Self {
            descriptor: PluginDescriptor {
                id: "allviewer".to_string(),
                name: "ALL Viewing".to_string(),
                version: "0.1.0".to_string(),
                description: "フレキシブルサムネイル＆ポップアップビューワー".to_string(),
                author: "Your Name".to_string(),
            },
            state: Arc::new(Mutex::new(AllViewerState {
                view_mode: "grid".to_string(),
                thumbnail_size: 150,
                show_labels: true,
                current_index: 0,
                current_directory: None,
                resource_config: None,
            })),
            context: None,
        }
    }

    // フロントエンドに渡すUI設定を生成
    fn generate_ui_config(&self) -> PluginResult<JsonValue> {
        let state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        Ok(json!({
            "viewMode": state.view_mode,
            "thumbnailSize": state.thumbnail_size,
            "showLabels": state.show_labels,
            "currentIndex": state.current_index,
            "currentDirectory": state.current_directory,
        }))
    }

    // サムネイルサイズを設定
    fn set_thumbnail_size(&self, size: u32) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        // サイズを妥当な範囲に制限
        let size = size.clamp(50, 300);
        state.thumbnail_size = size;

        Ok(())
    }

    // 表示モードを設定
    fn set_view_mode(&self, mode: String) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        // モードを検証
        if mode == "grid" || mode == "list" || mode == "detail" {
            state.view_mode = mode;
            Ok(())
        } else {
            Err(format!("Invalid view mode: {}", mode))
        }
    }

    // ラベル表示設定を切り替え
    fn toggle_labels(&self) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        state.show_labels = !state.show_labels;
        Ok(())
    }

    // ディレクトリを設定
    fn set_directory(&self, path: String) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        state.current_directory = Some(path);
        state.current_index = 0;

        // ここで必要に応じてリソース設定も更新
        let resource_config = ResourceConfig {
            id: "allviewer-current".to_string(),
            name: "AllViewer Current Directory".to_string(),
            filters: crate::core::resource_manager::ResourceFilter {
                include: vec![state.current_directory.clone().unwrap_or_default()],
                exclude: vec![],
            },
        };
        state.resource_config = Some(resource_config);

        Ok(())
    }

    // APIハンドラを追加
    fn setup_api_handlers(&self) -> Vec<(&'static str, Box<dyn Fn(JsonValue) -> PluginResult<JsonValue> + Send + Sync>)> {
        // それぞれのハンドラに別々のクローンを渡す
        vec![
            // set_thumbnail_size ハンドラ
            ("set_thumbnail_size", {
                let state_clone = Arc::clone(&self.state);
                Box::new(move |args: JsonValue| -> PluginResult<JsonValue> {
                    if let Some(size) = args.get("size").and_then(|s| s.as_u64()) {
                        let mut state = state_clone.lock().map_err(|e| {
                            format!("Failed to lock state: {}", e)
                        })?;
                        state.thumbnail_size = size as u32;
                        Ok(json!({"success": true}))
                    } else {
                        Err("Invalid size parameter".to_string())
                    }
                })
            }),
            
            // set_view_mode ハンドラ
            ("set_view_mode", {
                let state_clone = Arc::clone(&self.state);
                Box::new(move |args: JsonValue| -> PluginResult<JsonValue> {
                    if let Some(mode) = args.get("mode").and_then(|m| m.as_str()) {
                        let mut state = state_clone.lock().map_err(|e| {
                            format!("Failed to lock state: {}", e)
                        })?;
                        state.view_mode = mode.to_string();
                        Ok(json!({"success": true}))
                    } else {
                        Err("Invalid mode parameter".to_string())
                    }
                })
            }),
        ]
    }
}

impl Plugin for AllViewerPlugin {
    fn get_id(&self) -> String {
        self.descriptor.id.clone()
    }

    fn get_descriptor(&self) -> PluginDescriptor {
        self.descriptor.clone()
    }

    fn initialize(&mut self, context: Arc<PluginContext>) -> PluginResult<()> {
        self.context = Some(Arc::clone(&context));
        
        // 現在のUnixタイムスタンプを取得
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();
        
        // 初期化イベントをログに記録
        if let Some(ctx) = &self.context {
            let _ = ctx.event_bus.publish("allviewer:initialized", json!({
                "timestamp": timestamp
            }));
        }
        
        Ok(())
    }

    fn activate(&mut self) -> PluginResult<()> {
        // 現在のUnixタイムスタンプを取得
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();
        
        // 有効化イベントをログに記録
        if let Some(ctx) = &self.context {
            let _ = ctx.event_bus.publish("allviewer:activated", json!({
                "timestamp": timestamp
            }));
        }
        
        Ok(())
    }

    fn deactivate(&mut self) -> PluginResult<()> {
        // 現在のUnixタイムスタンプを取得
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();
        
        // 無効化イベントをログに記録
        if let Some(ctx) = &self.context {
            let _ = ctx.event_bus.publish("allviewer:deactivated", json!({
                "timestamp": timestamp
            }));
        }
        
        Ok(())
    }

    fn get_config(&self) -> PluginResult<JsonValue> {
        let state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        Ok(json!({
            "viewMode": state.view_mode,
            "thumbnailSize": state.thumbnail_size,
            "showLabels": state.show_labels,
            "currentDirectory": state.current_directory,
        }))
    }

    fn update_config(&mut self, config: JsonValue) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        // 設定値を更新
        if let Some(view_mode) = config.get("viewMode").and_then(|v| v.as_str()) {
            state.view_mode = view_mode.to_string();
        }

        if let Some(thumbnail_size) = config.get("thumbnailSize").and_then(|t| t.as_u64()) {
            state.thumbnail_size = thumbnail_size as u32;
        }

        if let Some(show_labels) = config.get("showLabels").and_then(|s| s.as_bool()) {
            state.show_labels = show_labels;
        }

        if let Some(current_dir) = config.get("currentDirectory").and_then(|d| d.as_str()) {
            state.current_directory = Some(current_dir.to_string());
        }

        Ok(())
    }

    fn get_frontend_code(&self) -> Option<String> {
        // フロントエンドコードを提供（オプション）
        // 実際のUIは主にTypeScriptで実装され、このRust側のコードは
        // バックエンド処理とフロントエンドへのデータ提供に焦点を当てています
        Some(ui::get_frontend_code())
    }

    fn get_api_handlers(&self) -> Vec<(&'static str, Box<dyn Fn(JsonValue) -> PluginResult<JsonValue> + Send + Sync>)> {
        self.setup_api_handlers()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event_bus::EventBus;

    #[test]
    fn test_allviewer_plugin_basic() {
        let plugin = AllViewerPlugin::new();
        
        assert_eq!(plugin.get_id(), "allviewer");
        assert_eq!(plugin.get_descriptor().name, "ALL Viewing");
        
        // 初期状態の確認
        let state = plugin.state.lock().unwrap();
        assert_eq!(state.view_mode, "grid");
        assert_eq!(state.thumbnail_size, 150);
        assert!(state.show_labels);
    }

    #[test]
    fn test_allviewer_config_update() {
        let mut plugin = AllViewerPlugin::new();
        
        // 設定を更新
        let config = json!({
            "viewMode": "list",
            "thumbnailSize": 200,
            "showLabels": false,
        });
        
        assert!(plugin.update_config(config).is_ok());
        
        // 更新後の状態を確認
        let state = plugin.state.lock().unwrap();
        assert_eq!(state.view_mode, "list");
        assert_eq!(state.thumbnail_size, 200);
        assert!(!state.show_labels);
    }

    #[test]
    fn test_allviewer_initialize() {
        let event_bus = Arc::new(EventBus::new());
        let context = Arc::new(PluginContext::new(Arc::clone(&event_bus)));
        
        let mut plugin = AllViewerPlugin::new();
        
        assert!(plugin.initialize(context).is_ok());
        assert!(plugin.context.is_some());
    }
}
