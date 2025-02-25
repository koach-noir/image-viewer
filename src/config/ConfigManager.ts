import { invoke } from '@tauri-apps/api/core';
import { appConfigDir, join } from '@tauri-apps/api/path';
import { createDir, exists, readTextFile, writeTextFile } from '@tauri-apps/api/fs';
import { EventPayload } from '../core/EventSystem';
import getEventSystem from '../core/EventSystem';

/**
 * 設定値の型定義
 * 任意の設定値を格納できるようにする
 */
export type ConfigValue = 
  | string
  | number
  | boolean
  | null
  | ConfigValue[]
  | { [key: string]: ConfigValue };

/**
 * アプリケーション設定のインターフェース
 * アプリ全体の設定を格納する
 */
export interface AppConfig {
  // アプリ全体の設定
  app: {
    // テーマ設定
    theme: 'light' | 'dark' | 'system';
    // 言語設定
    language: string;
    // 最後に開いたディレクトリ
    lastOpenedDirectory?: string;
    // デバッグモード
    debugMode: boolean;
    // 自動的に有効化するプラグインのID
    autoActivatePlugins: string[];
  };
  // プラグイン設定（プラグインID -> 設定）
  plugins: Record<string, any>;
  // ユーザー定義のホットキー
  hotkeys?: Record<string, string>;
  // その他の設定
  [key: string]: any;
}

/**
 * プラグイン設定のインターフェース
 */
export interface PluginConfig {
  // プラグインの一意識別子
  pluginId: string;
  // 設定データ（任意の形式）
  data: Record<string, any>;
  // プラグイン設定のバージョン
  version?: string;
}

/**
 * 設定スキーマ検証結果
 */
export interface ValidationResult {
  valid: boolean;
  errors?: string[];
}

/**
 * 設定マネージャークラス
 * アプリケーション設定の読み込み、保存、検証を行う
 */
export class ConfigManager {
  private static instance: ConfigManager;
  private config: AppConfig;
  private configPath: string = '';
  private loaded: boolean = false;
  private saving: boolean = false;
  private eventSystem = getEventSystem();
  
  // デフォルト設定
  private readonly defaultConfig: AppConfig = {
    app: {
      theme: 'system',
      language: 'ja',
      debugMode: false,
      autoActivatePlugins: []
    },
    plugins: {}
  };

  /**
   * プライベートコンストラクタ（シングルトンパターン）
   */
  private constructor() {
    this.config = { ...this.defaultConfig };
  }

  /**
   * シングルトンインスタンスを取得
   */
  public static getInstance(): ConfigManager {
    if (!ConfigManager.instance) {
      ConfigManager.instance = new ConfigManager();
    }
    return ConfigManager.instance;
  }

  /**
   * 設定を初期化
   * 設定ファイルがない場合はデフォルト設定を使用
   */
  public async initialize(): Promise<void> {
    try {
      // 設定ディレクトリのパスを取得
      const configDir = await appConfigDir();
      
      // 設定ファイルのパスを構築
      this.configPath = await join(configDir, 'config.json');
      
      // 設定ディレクトリが存在するか確認
      const dirExists = await exists(configDir);
      if (!dirExists) {
        // 設定ディレクトリが存在しない場合は作成
        await createDir(configDir, { recursive: true });
      }
      
      // 設定ファイルが存在するか確認
      const fileExists = await exists(this.configPath);
      if (fileExists) {
        // 設定ファイルが存在する場合は読み込み
        await this.loadConfig();
      } else {
        // 設定ファイルが存在しない場合はデフォルト設定を保存
        this.config = { ...this.defaultConfig };
        await this.saveConfig();
      }
      
      this.loaded = true;
      
      // 初期化完了イベントを発行
      this.eventSystem.publish('config:initialized', this.config);
      
    } catch (error) {
      console.error('Failed to initialize config:', error);
      // 初期化エラーイベントを発行
      this.eventSystem.publish('config:error', {
        message: 'Failed to initialize config',
        error: String(error)
      });
      
      // エラーが発生した場合もデフォルト設定を使用
      this.config = { ...this.defaultConfig };
    }
  }

  /**
   * 設定ファイルを読み込む
   */
  private async loadConfig(): Promise<void> {
    try {
      const content = await readTextFile(this.configPath);
      
      // JSONをパース
      const loadedConfig = JSON.parse(content);
      
      // 設定を検証
      const validationResult = this.validateConfig(loadedConfig);
      
      if (!validationResult.valid) {
        console.warn('Config validation failed:', validationResult.errors);
        
        // エラーイベントを発行
        this.eventSystem.publish('config:validationError', {
          errors: validationResult.errors
        });
        
        // 不正な設定の場合は、既存の設定とマージしてデフォルト値を保持
        this.config = this.mergeConfigs(this.defaultConfig, loadedConfig);
      } else {
        // 正常な設定の場合は、デフォルト設定とマージして不足値を補完
        this.config = this.mergeConfigs(this.defaultConfig, loadedConfig);
      }
      
      // 設定読み込みイベントを発行
      this.eventSystem.publish('config:loaded', this.config);
      
    } catch (error) {
      console.error('Failed to load config:', error);
      
      // エラーイベントを発行
      this.eventSystem.publish('config:error', {
        message: 'Failed to load config',
        error: String(error)
      });
      
      // エラーが発生した場合はデフォルト設定を使用
      this.config = { ...this.defaultConfig };
    }
  }

  /**
   * 設定を保存
   */
  public async saveConfig(): Promise<boolean> {
    if (this.saving) {
      return false;
    }
    
    this.saving = true;
    
    try {
      // 設定を検証
      const validationResult = this.validateConfig(this.config);
      if (!validationResult.valid) {
        console.error('Cannot save invalid config:', validationResult.errors);
        this.saving = false;
        return false;
      }
      
      // 設定をJSON形式で保存
      const content = JSON.stringify(this.config, null, 2);
      await writeTextFile(this.configPath, content);
      
      // 保存完了イベントを発行
      this.eventSystem.publish('config:saved', { path: this.configPath });
      
      this.saving = false;
      return true;
    } catch (error) {
      console.error('Failed to save config:', error);
      
      // エラーイベントを発行
      this.eventSystem.publish('config:error', {
        message: 'Failed to save config',
        error: String(error)
      });
      
      this.saving = false;
      return false;
    }
  }

  /**
   * 設定を検証
   * @param config 検証する設定
   */
  private validateConfig(config: any): ValidationResult {
    const errors: string[] = [];
    
    // 設定がオブジェクトかどうか確認
    if (typeof config !== 'object' || config === null) {
      errors.push('Config must be an object');
      return { valid: false, errors };
    }
    
    // app設定が存在するか確認
    if (!config.app) {
      errors.push('App settings missing');
    } else {
      // 必須フィールドの確認
      if (config.app.theme && !['light', 'dark', 'system'].includes(config.app.theme)) {
        errors.push('Invalid theme value, must be "light", "dark", or "system"');
      }
    }
    
    // plugins設定がオブジェクトかどうか確認
    if (config.plugins && typeof config.plugins !== 'object') {
      errors.push('Plugins config must be an object');
    }
    
    return {
      valid: errors.length === 0,
      errors: errors.length > 0 ? errors : undefined
    };
  }

  /**
   * 設定をマージ
   * @param defaultConfig デフォルト設定
   * @param userConfig ユーザー設定
   */
  private mergeConfigs(defaultConfig: AppConfig, userConfig: any): AppConfig {
    // 深いマージを行う関数
    const deepMerge = (target: any, source: any): any => {
      const result: any = { ...target };
      
      for (const key in source) {
        if (source.hasOwnProperty(key)) {
          if (
            source[key] && 
            typeof source[key] === 'object' && 
            !Array.isArray(source[key])
          ) {
            if (target[key] && typeof target[key] === 'object' && !Array.isArray(target[key])) {
              result[key] = deepMerge(target[key], source[key]);
            } else {
              result[key] = source[key];
            }
          } else {
            result[key] = source[key];
          }
        }
      }
      
      return result;
    };
    
    return deepMerge(defaultConfig, userConfig);
  }

  /**
   * 設定の一部を取得
   * @param key 設定キー（ドット区切りで階層指定可能）
   * @param defaultValue デフォルト値
   */
  public get<T = any>(key: string, defaultValue?: T): T {
    if (!this.loaded) {
      console.warn('Trying to get config before initialization');
    }
    
    const keys = key.split('.');
    let result: any = this.config;
    
    for (const k of keys) {
      if (result === undefined || result === null) {
        return defaultValue as T;
      }
      result = result[k];
    }
    
    return result === undefined ? defaultValue as T : result;
  }

  /**
   * 設定の一部を更新
   * @param key 設定キー（ドット区切りで階層指定可能）
   * @param value 設定値
   * @param autoSave 自動保存するかどうか
   */
  public async set<T = any>(key: string, value: T, autoSave: boolean = true): Promise<boolean> {
    if (!this.loaded) {
      console.warn('Trying to set config before initialization');
      return false;
    }
    
    const keys = key.split('.');
    const lastKey = keys.pop();
    
    if (!lastKey) {
      console.error('Invalid key');
      return false;
    }
    
    let target: any = this.config;
    
    // 対象のオブジェクトまで移動
    for (const k of keys) {
      if (target[k] === undefined || target[k] === null) {
        target[k] = {};
      }
      if (typeof target[k] !== 'object') {
        console.error(`Cannot set property ${key} because ${k} is not an object`);
        return false;
      }
      target = target[k];
    }
    
    // 値を設定
    target[lastKey] = value;
    
    // 設定変更イベントを発行
    this.eventSystem.publish('config:changed', { key, value });
    
    // 自動保存が有効な場合は保存
    if (autoSave) {
      return await this.saveConfig();
    }
    
    return true;
  }

  /**
   * プラグイン設定を取得
   * @param pluginId プラグインID
   */
  public getPluginConfig(pluginId: string): PluginConfig {
    if (!this.loaded) {
      console.warn('Trying to get plugin config before initialization');
    }
    
    const pluginData = this.config.plugins[pluginId] || {};
    
    return {
      pluginId,
      data: pluginData,
      version: pluginData.version
    };
  }

  /**
   * プラグイン設定を保存
   * @param config プラグイン設定
   * @param autoSave 自動保存するかどうか
   */
  public async savePluginConfig(config: PluginConfig, autoSave: boolean = true): Promise<boolean> {
    if (!this.loaded) {
      console.warn('Trying to save plugin config before initialization');
      return false;
    }
    
    // プラグイン設定が存在しない場合は作成
    if (!this.config.plugins) {
      this.config.plugins = {};
    }
    
    // プラグイン設定を更新
    this.config.plugins[config.pluginId] = config.data;
    
    // 設定変更イベントを発行
    this.eventSystem.publish('config:pluginChanged', { 
      pluginId: config.pluginId, 
      config: config.data 
    });
    
    // 自動保存が有効な場合は保存
    if (autoSave) {
      return await this.saveConfig();
    }
    
    return true;
  }

  /**
   * プラグイン設定を削除
   * @param pluginId プラグインID
   * @param autoSave 自動保存するかどうか
   */
  public async removePluginConfig(pluginId: string, autoSave: boolean = true): Promise<boolean> {
    if (!this.loaded) {
      console.warn('Trying to remove plugin config before initialization');
      return false;
    }
    
    if (this.config.plugins && this.config.plugins[pluginId]) {
      // プラグイン設定を削除
      delete this.config.plugins[pluginId];
      
      // 設定変更イベントを発行
      this.eventSystem.publish('config:pluginRemoved', { pluginId });
      
      // 自動保存が有効な場合は保存
      if (autoSave) {
        return await this.saveConfig();
      }
      
      return true;
    }
    
    return false;
  }

  /**
   * 設定イベントを購読
   * @param eventType イベントタイプ
   * @param handler イベントハンドラ
   */
  public subscribe<T extends EventPayload<any>>(
    eventType: string, 
    handler: (data: T) => void
  ): () => void {
    return this.eventSystem.subscribe<T>(eventType, handler);
  }

  /**
   * 設定を初期化（テスト用）
   */
  public reset(): void {
    this.config = { ...this.defaultConfig };
    this.loaded = false;
  }

  /**
   * バックエンド側で設定を更新（必要に応じて）
   * @param key 設定キー
   * @param value 設定値
   */
  public async updateBackendConfig(key: string, value: any): Promise<boolean> {
    try {
      await invoke('update_config', { key, value });
      return true;
    } catch (error) {
      console.error('Failed to update backend config:', error);
      return false;
    }
  }

  /**
   * 現在の設定全体を取得
   */
  public getFullConfig(): AppConfig {
    return { ...this.config };
  }
}

// シングルトンインスタンスをエクスポート
export default ConfigManager.getInstance();