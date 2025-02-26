// import { PluginInfo, PluginConfig, BasePlugin } from '../PluginInterface';
import { PluginInfo, BasePlugin } from '../PluginInterface';
import { invoke } from '@tauri-apps/api/core';
import FindMeUI from './FindMeUI';
import ConfigManager from '../../config/ConfigManager';
import getEventSystem from '../../core/EventSystem';

// プラグインの設定型定義
interface FindMeConfig {
  difficulty: 'easy' | 'medium' | 'hard';
  timeLimit: number;
  showHints: boolean;
}

// バックエンドレスポンスの型定義
interface PluginResult {
  success: boolean;
  error?: string;
  difficulty?: string;
}

class FindMePlugin extends BasePlugin {
  private eventSystem = getEventSystem();

  getInfo(): PluginInfo {
    return {
      id: 'findme',
      name: 'FindMe Game',
      version: '0.1.0',
      description: '画像探しゲームプラグイン',
      author: 'Your Name',
    };
  }

  getUIComponent() {
    return FindMeUI;
  }

  async initialize() {
    try {
      // プラグイン固有の初期設定をロード
      const savedConfig = await this.loadConfig();
      if (savedConfig) {
        // 設定がある場合は適用
        // 今は特に処理なし
      }

      // イベント購読
      this.subscribeToEvents();

      return true;
    } catch (error) {
      console.error('FindMe plugin initialization failed:', error);
      return false;
    }
  }

  async activate() {
    try {
      // 必要に応じて実装
      return true;
    } catch (error) {
      console.error('FindMe plugin activation failed:', error);
      return false;
    }
  }

  async deactivate() {
    // 必要に応じて実装
    return true;
  }

  // 設定をロード
  private async loadConfig(): Promise<FindMeConfig | null> {
    try {
      const config = await ConfigManager.get('plugins.findme');
      return config as FindMeConfig | null;
    } catch (error) {
      console.warn('Failed to load FindMe config:', error);
      return null;
    }
  }

  // イベント購読
  private subscribeToEvents() {
    // ゲーム開始イベント
    this.eventSystem.subscribe('findme:start_game', async () => {
      try {
        await this.startGame();
      } catch (error) {
        console.error('Failed to start game:', error);
      }
    });

    // 難易度変更イベント
    this.eventSystem.subscribe('findme:set_difficulty', async (data: { difficulty: 'easy' | 'medium' | 'hard' }) => {
      try {
        await this.setDifficulty(data.difficulty);
      } catch (error) {
        console.error('Failed to set difficulty:', error);
      }
    });
  }

  // ゲーム開始メソッド
  private async startGame(): Promise<void> {
    try {
      // ゲーム開始のバックエンド処理を呼び出し
      const result = await invoke<PluginResult>('plugin:findme:start_game', {});
    
      if (result.success) {
        // UIにゲーム開始を通知
        this.eventSystem.publish('findme:game_started', {
          message: 'Game has started!',
          timestamp: Date.now()
        });
      }
    } catch (error) {
      console.error('Game start failed:', error);
      throw error;
    }
  }

  // 難易度設定メソッド
  private async setDifficulty(difficulty: 'easy' | 'medium' | 'hard'): Promise<void> {
    try {
      // バックエンドに難易度を設定
      const result = await invoke<PluginResult>('plugin:findme:set_difficulty', { difficulty });
    
      if (result.success) {
        // 設定を保存
        await ConfigManager.set('plugins.findme.difficulty', difficulty);
        
        // UIに難易度変更を通知
        this.eventSystem.publish('findme:difficulty_changed', {
          difficulty,
          timestamp: Date.now()
        });
      }
    } catch (error) {
      console.error('Difficulty setting failed:', error);
      throw error;
    }
  }

  // プラグイン固有のAPIハンドラを取得
  getApiHandlers() {
    return [
      {
        name: 'start_game',
        // handler: async (args?: any) => {
        handler: async (_args?: any) => {
          try {
            await this.startGame();
            return { success: true };
          } catch (error) {
            return { 
              success: false, 
              error: error instanceof Error ? error.message : String(error) 
            };
          }
        }
      },
      {
        name: 'set_difficulty',
        handler: async (args: { difficulty: 'easy' | 'medium' | 'hard' }) => {
          try {
            const difficulty = args.difficulty;
            if (!['easy', 'medium', 'hard'].includes(difficulty)) {
              throw new Error('Invalid difficulty');
            }
            await this.setDifficulty(difficulty);
            return { success: true, difficulty };
          } catch (error) {
            return { 
              success: false, 
              error: error instanceof Error ? error.message : String(error) 
            };
          }
        }
      }
    ];
  }

  // イベントシステムを取得するメソッドを追加
  getEventSystem() {
    return this.eventSystem;
  }
}

export default new FindMePlugin();
