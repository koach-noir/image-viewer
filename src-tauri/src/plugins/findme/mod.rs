// findme/mod.rs - FindMeプラグイン（画像探しゲームアプリ）の実装

use std::sync::{Arc, Mutex};
use serde_json::{json, Value as JsonValue};
use crate::core::plugin_context::PluginContext;
use crate::plugins::plugin_trait::{Plugin, PluginDescriptor, PluginResult};

// UIモジュールをインポート
pub mod ui;

// プラグインの状態
#[derive(Debug, Default)]
struct FindMeState {
    // ゲームの難易度
    difficulty: String,
    // 現在のスコア
    score: u32,
    // 制限時間
    time_limit: u32,
    // ゲームの状態（準備中、進行中、終了）
    game_state: String,
}

// FindMeプラグインの実装
pub struct FindMePlugin {
    // プラグイン記述子
    descriptor: PluginDescriptor,
    // プラグインの状態
    state: Arc<Mutex<FindMeState>>,
    // プラグインコンテキスト
    context: Option<Arc<PluginContext>>,
}

// プラグインインスタンスを作成する関数
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(FindMePlugin::new())
}

impl FindMePlugin {
    // 新しいインスタンスを作成
    pub fn new() -> Self {
        Self {
            descriptor: PluginDescriptor {
                id: "findme".to_string(),
                name: "FindMe Game".to_string(),
                version: "0.1.0".to_string(),
                description: "画像探しゲーム".to_string(),
                author: "Your Name".to_string(),
            },
            state: Arc::new(Mutex::new(FindMeState {
                difficulty: "easy".to_string(),
                score: 0,
                time_limit: 60,
                game_state: "ready".to_string(),
            })),
            context: None,
        }
    }

    // ゲームを開始
    fn start_game(&self) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        state.game_state = "in_progress".to_string();
        state.score = 0;
        
        Ok(())
    }

    // ゲームを終了
    fn end_game(&self) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        state.game_state = "finished".to_string();
        
        Ok(())
    }

    // 難易度を設定
    fn set_difficulty(&self, difficulty: String) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        // 難易度のバリデーション
        if ["easy", "medium", "hard"].contains(&difficulty.as_str()) {
            state.difficulty = difficulty;
            Ok(())
        } else {
            Err(format!("Invalid difficulty: {}", difficulty))
        }
    }
}

impl Plugin for FindMePlugin {
    fn get_id(&self) -> String {
        self.descriptor.id.clone()
    }

    fn get_descriptor(&self) -> PluginDescriptor {
        self.descriptor.clone()
    }

    fn initialize(&mut self, context: Arc<PluginContext>) -> PluginResult<()> {
        self.context = Some(Arc::clone(&context));
        
        // 初期化イベントをログに記録
        if let Some(ctx) = &self.context {
            let _ = ctx.event_bus.publish("findme:initialized", json!({
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("Failed to get timestamp: {}", e))?
                    .as_secs()
            }));
        }
        
        Ok(())
    }

    fn activate(&mut self) -> PluginResult<()> {
        // 有効化イベントをログに記録
        if let Some(ctx) = &self.context {
            let _ = ctx.event_bus.publish("findme:activated", json!({
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("Failed to get timestamp: {}", e))?
                    .as_secs()
            }));
        }
        
        Ok(())
    }

    fn deactivate(&mut self) -> PluginResult<()> {
        // 無効化イベントをログに記録
        if let Some(ctx) = &self.context {
            let _ = ctx.event_bus.publish("findme:deactivated", json!({
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("Failed to get timestamp: {}", e))?
                    .as_secs()
            }));
        }
        
        Ok(())
    }

    fn get_config(&self) -> PluginResult<JsonValue> {
        let state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        Ok(json!({
            "difficulty": state.difficulty,
            "timeLimit": state.time_limit,
            "gameState": state.game_state,
        }))
    }

    fn update_config(&mut self, config: JsonValue) -> PluginResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            format!("Failed to lock state: {}", e)
        })?;

        // 設定値を更新
        if let Some(difficulty) = config.get("difficulty").and_then(|d| d.as_str()) {
            state.difficulty = difficulty.to_string();
        }

        if let Some(time_limit) = config.get("timeLimit").and_then(|t| t.as_u64()) {
            state.time_limit = time_limit as u32;
        }

        Ok(())
    }

    fn get_frontend_code(&self) -> Option<String> {
        Some(ui::get_frontend_code())
    }

    fn get_api_handlers(&self) -> Vec<(&'static str, Box<dyn Fn(JsonValue) -> PluginResult<JsonValue> + Send + Sync>)> {
        vec![
            ("start_game", {
                let state_clone = Arc::clone(&self.state);
                Box::new(move |_args: JsonValue| -> PluginResult<JsonValue> {
                    let plugin_state = state_clone.clone();
                    let mut state = plugin_state.lock().map_err(|e| {
                        format!("Failed to lock state: {}", e)
                    })?;

                    state.game_state = "in_progress".to_string();
                    state.score = 0;

                    Ok(json!({"success": true, "message": "Game started"}))
                })
            }),
            ("set_difficulty", {
                let state_clone = Arc::clone(&self.state);
                Box::new(move |args: JsonValue| -> PluginResult<JsonValue> {
                    let difficulty = args.get("difficulty")
                        .and_then(|d| d.as_str())
                        .ok_or("No difficulty provided")?
                        .to_string();

                    let plugin_state = state_clone.clone();
                    let mut state = plugin_state.lock().map_err(|e| {
                        format!("Failed to lock state: {}", e)
                    })?;

                    // 難易度のバリデーション
                    if ["easy", "medium", "hard"].contains(&difficulty.as_str()) {
                        state.difficulty = difficulty.clone();
                        Ok(json!({"success": true, "difficulty": difficulty}))
                    } else {
                        Err(format!("Invalid difficulty: {}", difficulty))
                    }
                })
            }),
        ]
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
    fn test_findme_plugin_basic() {
        let plugin = FindMePlugin::new();
        
        assert_eq!(plugin.get_id(), "findme");
        assert_eq!(plugin.get_descriptor().name, "FindMe Game");
        
        // 初期状態の確認
        let state = plugin.state.lock().unwrap();
        assert_eq!(state.difficulty, "easy");
        assert_eq!(state.score, 0);
    }
}
