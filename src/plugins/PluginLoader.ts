import { PluginInterface, PluginInfo, PluginContext } from './PluginInterface';
// import { invoke } from '@tauri-apps/api/core';

/**
 * プラグイン管理の状態
 */
export enum PluginState {
  /** 未ロード */
  UNLOADED = 'unloaded',
  /** 初期化済み */
  INITIALIZED = 'initialized',
  /** 有効化済み */
  ACTIVE = 'active',
  /** エラー */
  ERROR = 'error'
}

/**
 * プラグインの登録情報
 */
export interface PluginRegistration {
  /** プラグインインスタンス */
  instance: PluginInterface;
  /** プラグインの状態 */
  state: PluginState;
  /** エラーメッセージ（あれば） */
  error?: string;
}

/**
 * プラグインローダー設定
 */
export interface PluginLoaderConfig {
  /** 自動的に有効化するプラグインID */
  autoActivatePlugins?: string[];
  /** プラグインのロード時に有効化も行うか */
  activateOnLoad?: boolean;
}

/**
 * プラグインローダークラス
 * プラグインの読み込みと管理を担当
 */
export class PluginLoader {
  private static instance: PluginLoader;
  
  /** 登録済みプラグイン */
  private plugins: Map<string, PluginRegistration> = new Map();
  /** プラグインコンテキスト */
  private context: PluginContext;
  /** 設定情報 */
  private config: PluginLoaderConfig;
  /** イベントリスナー */
  private eventListeners: Map<string, Set<Function>> = new Map();

  /**
   * プライベートコンストラクタ（シングルトンパターン）
   */
  private constructor(context: PluginContext, config: PluginLoaderConfig = {}) {
    this.context = context;
    this.config = {
      autoActivatePlugins: [],
      activateOnLoad: false,
      ...config
    };
  }

  /**
   * シングルトンインスタンスを取得
   */
  public static getInstance(context?: PluginContext, config?: PluginLoaderConfig): PluginLoader {
    if (!PluginLoader.instance) {
      if (!context) {
        throw new Error('PluginLoader must be initialized with a context first');
      }
      PluginLoader.instance = new PluginLoader(context, config);
    }
    return PluginLoader.instance;
  }

  /**
   * プラグインを登録
   * @param plugin プラグインインスタンス
   * @returns 登録が成功したかどうか
   */
  public async registerPlugin(plugin: PluginInterface): Promise<boolean> {
    try {
      const info = plugin.getInfo();
      
      // 重複チェック
      if (this.plugins.has(info.id)) {
        this.context.logger.warn(`Plugin with ID ${info.id} is already registered`);
        return false;
      }
      
      // 登録
      this.plugins.set(info.id, {
        instance: plugin,
        state: PluginState.UNLOADED
      });
      
      // 初期化
      await this.initializePlugin(info.id);
      
      // 自動有効化が設定されていれば有効化
      if (this.config.activateOnLoad || 
          (this.config.autoActivatePlugins && 
           this.config.autoActivatePlugins.includes(info.id))) {
        await this.activatePlugin(info.id);
      }
      
      // イベント発火
      this.emit('plugin:registered', { pluginId: info.id, info });
      
      return true;
    } catch (error) {
      this.context.logger.error(`Failed to register plugin: ${error}`);
      return false;
    }
  }

  /**
   * プラグインコレクションを登録
   * @param plugins プラグインの配列
   */
  public async registerPlugins(plugins: PluginInterface[]): Promise<void> {
    for (const plugin of plugins) {
      await this.registerPlugin(plugin);
    }
  }

  /**
   * プラグインを初期化
   * @param pluginId プラグインID
   */
  private async initializePlugin(pluginId: string): Promise<boolean> {
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      this.context.logger.error(`Plugin ${pluginId} not found`);
      return false;
    }
    
    try {
      const success = await registration.instance.initialize(this.context);
      
      if (success) {
        registration.state = PluginState.INITIALIZED;
        this.emit('plugin:initialized', { pluginId });
        return true;
      } else {
        registration.state = PluginState.ERROR;
        registration.error = 'Initialization failed';
        this.emit('plugin:error', { 
          pluginId, 
          error: registration.error 
        });
        return false;
      }
    } catch (error) {
      registration.state = PluginState.ERROR;
      registration.error = `Initialization error: ${error}`;
      this.emit('plugin:error', { 
        pluginId, 
        error: registration.error 
      });
      return false;
    }
  }

  /**
   * プラグインを有効化
   * @param pluginId プラグインID
   */
  public async activatePlugin(pluginId: string): Promise<boolean> {
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      this.context.logger.error(`Plugin ${pluginId} not found`);
      return false;
    }
    
    // 既に有効化されている場合は何もしない
    if (registration.state === PluginState.ACTIVE) {
      return true;
    }
    
    // 未初期化の場合は初期化を試みる
    if (registration.state === PluginState.UNLOADED) {
      const initialized = await this.initializePlugin(pluginId);
      if (!initialized) {
        return false;
      }
    }
    
    // エラー状態の場合は有効化できない
    if (registration.state === PluginState.ERROR) {
      this.context.logger.error(`Cannot activate plugin ${pluginId} due to error: ${registration.error}`);
      return false;
    }
    
    try {
      const success = await registration.instance.activate();
      
      if (success) {
        registration.state = PluginState.ACTIVE;
        this.emit('plugin:activated', { pluginId });
        return true;
      } else {
        registration.error = 'Activation failed';
        this.emit('plugin:activationFailed', { 
          pluginId, 
          error: registration.error 
        });
        return false;
      }
    } catch (error) {
      registration.error = `Activation error: ${error}`;
      this.emit('plugin:activationFailed', { 
        pluginId, 
        error: registration.error 
      });
      return false;
    }
  }

  /**
   * プラグインを無効化
   * @param pluginId プラグインID
   */
  public async deactivatePlugin(pluginId: string): Promise<boolean> {
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      this.context.logger.error(`Plugin ${pluginId} not found`);
      return false;
    }
    
    // 有効化されていない場合は何もしない
    if (registration.state !== PluginState.ACTIVE) {
      return true;
    }
    
    try {
      const success = await registration.instance.deactivate();
      
      if (success) {
        registration.state = PluginState.INITIALIZED;
        this.emit('plugin:deactivated', { pluginId });
        return true;
      } else {
        registration.error = 'Deactivation failed';
        this.emit('plugin:deactivationFailed', { 
          pluginId, 
          error: registration.error 
        });
        return false;
      }
    } catch (error) {
      registration.error = `Deactivation error: ${error}`;
      this.emit('plugin:deactivationFailed', { 
        pluginId, 
        error: registration.error 
      });
      return false;
    }
  }

  /**
   * プラグインの登録を解除
   * @param pluginId プラグインID
   */
  public async unregisterPlugin(pluginId: string): Promise<boolean> {
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      return false;
    }
    
    // 有効化されている場合は先に無効化
    if (registration.state === PluginState.ACTIVE) {
      await this.deactivatePlugin(pluginId);
    }
    
    // 登録解除
    this.plugins.delete(pluginId);
    this.emit('plugin:unregistered', { pluginId });
    
    return true;
  }

  /**
   * プラグインインスタンスを取得
   * @param pluginId プラグインID
   */
  public getPlugin(pluginId: string): PluginInterface | null {
    const registration = this.plugins.get(pluginId);
    return registration ? registration.instance : null;
  }

  /**
   * プラグイン登録情報を取得
   * @param pluginId プラグインID
   */
  public getPluginRegistration(pluginId: string): PluginRegistration | null {
    return this.plugins.get(pluginId) || null;
  }

  /**
   * すべてのプラグインを取得
   */
  public getAllPlugins(): PluginInterface[] {
    return Array.from(this.plugins.values()).map(reg => reg.instance);
  }

  /**
   * すべてのプラグイン情報を取得
   */
  public getAllPluginInfo(): PluginInfo[] {
    return this.getAllPlugins().map(plugin => plugin.getInfo());
  }

  /**
   * 有効化されているプラグインを取得
   */
  public getActivePlugins(): PluginInterface[] {
    return Array.from(this.plugins.values())
      .filter(reg => reg.state === PluginState.ACTIVE)
      .map(reg => reg.instance);
  }

  /**
   * 有効化されているプラグイン情報を取得
   */
  public getActivePluginInfo(): PluginInfo[] {
    return this.getActivePlugins().map(plugin => plugin.getInfo());
  }

  /**
   * プラグインの状態を取得
   * @param pluginId プラグインID
   */
  public getPluginState(pluginId: string): PluginState | null {
    const registration = this.plugins.get(pluginId);
    return registration ? registration.state : null;
  }

  /**
   * イベントリスナーを登録
   * @param event イベント名
   * @param listener リスナー関数
   */
  public on(event: string, listener: Function): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event)!.add(listener);
  }

  /**
   * イベントリスナーを解除
   * @param event イベント名
   * @param listener リスナー関数
   */
  public off(event: string, listener: Function): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.delete(listener);
    }
  }

  /**
   * イベントを発行
   * @param event イベント名
   * @param data イベントデータ
   */
  private emit(event: string, data: any): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      for (const listener of listeners) {
        try {
          listener(data);
        } catch (error) {
          this.context.logger.error(`Error in event listener for ${event}: ${error}`);
        }
      }
    }
  }

  /**
   * 外部ソースからプラグインをロード
   * @param source プラグインソース情報
   */
  public async loadExternalPlugin(source: string): Promise<boolean> {
    // 注: この関数はプロトタイプとして提供され、実際の実装はプラグインの
    // ロード方法に応じて変更が必要です。現在はTauriのinvokeを使用して
    // バックエンドからプラグインをロードする例を示しています。
    try {
      // バックエンドからプラグインをロード
      // const pluginData = await invoke<any>('load_external_plugin', { source });
      
      // ここでは、プラグインのデータ構造をフロントエンドのプラグインクラスに変換
      // する処理が必要です。これはプラグインの実装方法によって異なります。
      
      // 例えば、プラグインのクラス定義を動的に評価して、インスタンスを作成
      // const PluginClass = eval(`(${pluginData.code})`);
      // const pluginInstance = new PluginClass();
      
      // 実際のシステムでは、動的評価ではなく、より安全な方法でプラグインを
      // ロードすることをお勧めします。
      
      // this.registerPlugin(pluginInstance);
      
      this.context.logger.info(`External plugin loaded from ${source}`);
      return true;
    } catch (error) {
      this.context.logger.error(`Failed to load external plugin: ${error}`);
      return false;
    }
  }
}

// デフォルトエクスポートをシングルトンアクセサに設定
export default {
  /**
   * PluginLoaderのシングルトンインスタンスを取得
   */
  getInstance: PluginLoader.getInstance
};
