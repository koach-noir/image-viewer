import { invoke } from '@tauri-apps/api/core';
import { PluginInterface, PluginInfo, PluginContext, PluginConfig } from '../plugins/PluginInterface';
import ConfigManager from '../config/ConfigManager';
import getEventSystem from './EventSystem';

/**
 * プラグインの状態を表す列挙型
 */
export enum PluginState {
  /** 未ロード */
  UNLOADED = 'unloaded',
  /** 初期化済み */
  INITIALIZED = 'initialized', 
  /** 有効化済み */
  ACTIVE = 'active',
  /** 無効化済み */
  INACTIVE = 'inactive',
  /** エラー */
  ERROR = 'error'
}

/**
 * プラグインの登録情報
 */
export interface PluginRegistration {
  /** プラグイン情報 */
  info: PluginInfo;
  /** プラグインインスタンス */
  instance: PluginInterface;
  /** プラグインの状態 */
  state: PluginState;
  /** エラーメッセージ（あれば） */
  error?: string;
  /** 依存関係 */
  dependencies: string[];
}

/**
 * プラグインレジストリの設定
 */
export interface PluginRegistryConfig {
  /** 自動的に初期化するかどうか */
  autoInitialize?: boolean;
  /** 自動的に有効化するプラグインID */
  autoActivatePlugins?: string[];
  /** プラグインのロード時に有効化も行うか */
  activateOnLoad?: boolean;
  /** デバッグモード */
  debugMode?: boolean;
}

/**
 * プラグインレジストリクラス
 * プラグインの登録と管理を行う
 */
export class PluginRegistry {
  private static instance: PluginRegistry;
  
  /** 登録済みプラグイン */
  private plugins: Map<string, PluginRegistration> = new Map();
  /** プラグインコンテキスト */
  private context: PluginContext;
  /** 登録済みイベントリスナー */
  private eventListeners: Map<string, Set<Function>> = new Map();
  /** 設定 */
  private config: PluginRegistryConfig;
  /** 初期化済みフラグ */
  private initialized: boolean = false;
  /** イベントシステム */
  private eventSystem = getEventSystem();
  /** 設定マネージャー */
  private configManager = ConfigManager;

  /**
   * プライベートコンストラクタ（シングルトンパターン）
   */
  private constructor(config: PluginRegistryConfig = {}) {
    this.config = {
      autoInitialize: true,
      autoActivatePlugins: [],
      activateOnLoad: false,
      debugMode: false,
      ...config
    };

    // コンテキストの作成
    this.context = {
      resourceManager: {}, // リソースマネージャーは後で追加
      eventBus: this.eventSystem,
      configManager: this.configManager,
      logger: {
        debug: (message: string) => {
          if (this.config.debugMode) {
            console.debug(`[Plugin] ${message}`);
          }
        },
        info: (message: string) => console.info(`[Plugin] ${message}`),
        warn: (message: string) => console.warn(`[Plugin] ${message}`),
        error: (message: string) => console.error(`[Plugin] ${message}`)
      }
    };
  }

  /**
   * シングルトンインスタンスを取得
   */
  public static getInstance(config?: PluginRegistryConfig): PluginRegistry {
    if (!PluginRegistry.instance) {
      PluginRegistry.instance = new PluginRegistry(config);
    } else if (config) {
      // 既存インスタンスの設定を更新
      PluginRegistry.instance.updateConfig(config);
    }
    return PluginRegistry.instance;
  }

  /**
   * 設定を更新
   */
  private updateConfig(config: Partial<PluginRegistryConfig>): void {
    this.config = {
      ...this.config,
      ...config
    };
  }

  /**
   * リソースマネージャーを設定
   * @param resourceManager リソースマネージャーインスタンス
   */
  public setResourceManager(resourceManager: any): void {
    this.context.resourceManager = resourceManager;
  }

  /**
   * プラグインレジストリを初期化
   */
  public async initialize(): Promise<boolean> {
    if (this.initialized) {
      return true;
    }

    try {
      // 設定マネージャーが初期化されていることを確認
      if (!this.configManager) {
        this.context.logger.error("ConfigManager is not available");
        return false;
      }

      // 設定からプラグイン初期化設定を読み込む
      const autoActivatePlugins = this.configManager.get<string[]>(
        'app.autoActivatePlugins', 
        this.config.autoActivatePlugins || []
      );
      
      this.config.autoActivatePlugins = autoActivatePlugins;
      this.initialized = true;

      // 初期化イベントを発行
      this.emit('registry:initialized', {});
      
      // 自動有効化プラグインの初期化を設定から取得
      this.context.logger.info(`Plugin registry initialized, ${autoActivatePlugins.length} plugins configured for auto-activation`);
      
      return true;
    } catch (error) {
      this.context.logger.error(`Failed to initialize plugin registry: ${error}`);
      return false;
    }
  }

  /**
   * プラグインを登録
   * @param plugin プラグインインスタンス
   */
  public async registerPlugin(plugin: PluginInterface): Promise<boolean> {
    if (!this.initialized && this.config.autoInitialize) {
      await this.initialize();
    }

    try {
      const info = plugin.getInfo();
      
      // 重複チェック
      if (this.plugins.has(info.id)) {
        this.context.logger.warn(`Plugin with ID ${info.id} is already registered`);
        return false;
      }
      
      // 依存関係の抽出
      const dependencies = this.extractDependencies(plugin);
      
      // 登録情報を作成
      const registration: PluginRegistration = {
        info,
        instance: plugin,
        state: PluginState.UNLOADED,
        dependencies
      };
      
      // プラグインを登録
      this.plugins.set(info.id, registration);
      
      // イベント発行
      this.emit('plugin:registered', { 
        pluginId: info.id, 
        info 
      });
      
      this.context.logger.info(`Plugin registered: ${info.id} v${info.version}`);
      
      // 自動初期化
      if (this.config.autoInitialize) {
        await this.initializePlugin(info.id);
      }
      
      // 自動有効化
      if (this.config.activateOnLoad || 
          (this.config.autoActivatePlugins && 
           this.config.autoActivatePlugins.includes(info.id))) {
        await this.activatePlugin(info.id);
      }
      
      return true;
    } catch (error) {
      this.context.logger.error(`Error registering plugin: ${error}`);
      return false;
    }
  }

  /**
   * 依存関係を抽出
   * @param plugin プラグインインスタンス
   */
  private extractDependencies(plugin: PluginInterface): string[] {
    // 将来的に依存関係の抽出ロジックを実装
    // 現在は未実装のため空配列を返す
    return [];
  }

  /**
   * プラグインを初期化
   * @param pluginId プラグインID
   */
  public async initializePlugin(pluginId: string): Promise<boolean> {
    // プラグインの状態をチェック
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      this.context.logger.error(`Plugin ${pluginId} not found`);
      return false;
    }
    
    // 既に初期化済みの場合は何もしない
    if (registration.state !== PluginState.UNLOADED) {
      if (registration.state === PluginState.ERROR) {
        this.context.logger.error(`Cannot initialize plugin ${pluginId} due to previous error: ${registration.error}`);
        return false;
      }
      return true;
    }
    
    // 依存関係の確認と初期化
    for (const depId of registration.dependencies) {
      if (!await this.initializePlugin(depId)) {
        registration.state = PluginState.ERROR;
        registration.error = `Dependency ${depId} initialization failed`;
        
        this.emit('plugin:error', {
          pluginId,
          error: registration.error,
          operation: 'initialize'
        });
        
        return false;
      }
    }
    
    try {
      // プラグイン設定を読み込む
      const pluginConfig = this.configManager.getPluginConfig(pluginId);
      
      // プラグインを初期化
      const success = await registration.instance.initialize(this.context);
      
      if (success) {
        // 状態を更新
        registration.state = PluginState.INITIALIZED;
        registration.error = undefined;
        
        // イベント発行
        this.emit('plugin:initialized', { pluginId });
        
        this.context.logger.info(`Plugin initialized: ${pluginId}`);
        return true;
      } else {
        registration.state = PluginState.ERROR;
        registration.error = 'Initialization failed';
        
        this.emit('plugin:error', {
          pluginId,
          error: registration.error,
          operation: 'initialize'
        });
        
        this.context.logger.error(`Plugin ${pluginId} initialization failed`);
        return false;
      }
    } catch (error) {
      registration.state = PluginState.ERROR;
      registration.error = `Initialization error: ${error}`;
      
      this.emit('plugin:error', {
        pluginId,
        error: registration.error,
        operation: 'initialize'
      });
      
      this.context.logger.error(`Error initializing plugin ${pluginId}: ${error}`);
      return false;
    }
  }

  /**
   * プラグインを有効化
   * @param pluginId プラグインID
   */
  public async activatePlugin(pluginId: string): Promise<boolean> {
    // プラグインの状態をチェック
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      this.context.logger.error(`Plugin ${pluginId} not found`);
      return false;
    }
    
    // 既に有効化されている場合は何もしない
    if (registration.state === PluginState.ACTIVE) {
      return true;
    }
    
    // エラー状態の場合は有効化できない
    if (registration.state === PluginState.ERROR) {
      this.context.logger.error(`Cannot activate plugin ${pluginId} due to error: ${registration.error}`);
      return false;
    }
    
    // 初期化されていない場合は初期化
    if (registration.state === PluginState.UNLOADED) {
      const initialized = await this.initializePlugin(pluginId);
      if (!initialized) {
        return false;
      }
    }
    
    try {
      // プラグインを有効化
      const success = await registration.instance.activate();
      
      if (success) {
        // 状態を更新
        registration.state = PluginState.ACTIVE;
        registration.error = undefined;
        
        // イベント発行
        this.emit('plugin:activated', { pluginId });
        
        this.context.logger.info(`Plugin activated: ${pluginId}`);
        return true;
      } else {
        registration.state = PluginState.ERROR;
        registration.error = 'Activation failed';
        
        this.emit('plugin:error', {
          pluginId,
          error: registration.error,
          operation: 'activate'
        });
        
        this.context.logger.error(`Plugin ${pluginId} activation failed`);
        return false;
      }
    } catch (error) {
      registration.state = PluginState.ERROR;
      registration.error = `Activation error: ${error}`;
      
      this.emit('plugin:error', {
        pluginId,
        error: registration.error,
        operation: 'activate'
      });
      
      this.context.logger.error(`Error activating plugin ${pluginId}: ${error}`);
      return false;
    }
  }

  /**
   * プラグインを無効化
   * @param pluginId プラグインID
   */
  public async deactivatePlugin(pluginId: string): Promise<boolean> {
    // プラグインの状態をチェック
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
      // プラグインを無効化
      const success = await registration.instance.deactivate();
      
      if (success) {
        // 状態を更新
        registration.state = PluginState.INACTIVE;
        registration.error = undefined;
        
        // イベント発行
        this.emit('plugin:deactivated', { pluginId });
        
        this.context.logger.info(`Plugin deactivated: ${pluginId}`);
        return true;
      } else {
        registration.error = 'Deactivation failed';
        
        this.emit('plugin:error', {
          pluginId,
          error: registration.error,
          operation: 'deactivate'
        });
        
        this.context.logger.error(`Plugin ${pluginId} deactivation failed`);
        return false;
      }
    } catch (error) {
      registration.error = `Deactivation error: ${error}`;
      
      this.emit('plugin:error', {
        pluginId,
        error: registration.error,
        operation: 'deactivate'
      });
      
      this.context.logger.error(`Error deactivating plugin ${pluginId}: ${error}`);
      return false;
    }
  }

  /**
   * プラグインの登録を解除
   * @param pluginId プラグインID
   */
  public async unregisterPlugin(pluginId: string): Promise<boolean> {
    // プラグインの状態をチェック
    const registration = this.plugins.get(pluginId);
    if (!registration) {
      return false;
    }
    
    // 有効化されている場合は先に無効化
    if (registration.state === PluginState.ACTIVE) {
      const deactivated = await this.deactivatePlugin(pluginId);
      if (!deactivated) {
        this.context.logger.error(`Failed to deactivate plugin ${pluginId} before unregistering`);
        return false;
      }
    }
    
    // 登録解除
    this.plugins.delete(pluginId);
    
    // イベント発行
    this.emit('plugin:unregistered', { pluginId });
    
    this.context.logger.info(`Plugin unregistered: ${pluginId}`);
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
    return Array.from(this.plugins.values()).map(reg => reg.info);
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
    return Array.from(this.plugins.values())
      .filter(reg => reg.state === PluginState.ACTIVE)
      .map(reg => reg.info);
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
   * イベントを発行
   * @param event イベント名
   * @param data イベントデータ
   */
  private emit(event: string, data: any): void {
    this.eventSystem.publish(event, data);
  }

  /**
   * イベントリスナーを登録
   * @param event イベント名
   * @param listener リスナー関数
   */
  public on<T = any>(event: string, listener: (data: T) => void): () => void {
    return this.eventSystem.subscribe<T>(event, listener);
  }

  /**
   * イベントリスナーを解除
   * @param event イベント名
   * @param listener リスナー関数
   */
  public off<T = any>(event: string, listener: (data: T) => void): void {
    this.eventSystem.unsubscribe(event, listener);
  }

  /**
   * 外部ソースからプラグインをロード
   * @param source プラグインソース情報
   */
  public async loadExternalPlugin(source: string): Promise<boolean> {
    try {
      // バックエンドからプラグインをロード
      const pluginData = await invoke<any>('load_external_plugin', { source });
      
      // この部分は実際のプラグインのロード方法によって異なる
      // 現在はプロトタイプとして提供
      this.context.logger.info(`External plugin loaded from ${source}`);
      
      return true;
    } catch (error) {
      this.context.logger.error(`Failed to load external plugin: ${error}`);
      return false;
    }
  }

  /**
   * 指定した状態のプラグインIDを取得
   * @param state プラグインの状態
   */
  public getPluginsByState(state: PluginState): string[] {
    return Array.from(this.plugins.entries())
      .filter(([_, reg]) => reg.state === state)
      .map(([id, _]) => id);
  }

  /**
   * プラグインの設定を取得
   * @param pluginId プラグインID
   */
  public getPluginConfig(pluginId: string): PluginConfig | null {
    try {
      return this.configManager.getPluginConfig(pluginId);
    } catch (error) {
      this.context.logger.error(`Failed to get plugin config for ${pluginId}: ${error}`);
      return null;
    }
  }

  /**
   * プラグインの設定を更新
   * @param config プラグイン設定
   */
  public async updatePluginConfig(config: PluginConfig): Promise<boolean> {
    try {
      // プラグインが存在するか確認
      const plugin = this.getPlugin(config.pluginId);
      if (!plugin) {
        this.context.logger.error(`Plugin ${config.pluginId} not found`);
        return false;
      }
      
      // プラグインに設定を更新
      const success = await plugin.updateConfig(config);
      if (!success) {
        this.context.logger.error(`Plugin ${config.pluginId} rejected config update`);
        return false;
      }
      
      // 設定マネージャーに保存
      await this.configManager.savePluginConfig(config);
      
      // イベント発行
      this.emit('plugin:configUpdated', {
        pluginId: config.pluginId,
        config: config.data
      });
      
      return true;
    } catch (error) {
      this.context.logger.error(`Failed to update plugin config: ${error}`);
      return false;
    }
  }

  /**
   * すべてのプラグインをロード（例：起動時）
   */
  public async loadAllPlugins(): Promise<boolean> {
    if (!this.initialized) {
      await this.initialize();
    }
    
    try {
      // Tauriバックエンドからプラグイン情報を取得
      const pluginPaths = await invoke<string[]>('get_plugin_paths');
      
      for (const path of pluginPaths) {
        try {
          await this.loadExternalPlugin(path);
        } catch (error) {
          this.context.logger.error(`Failed to load plugin from ${path}: ${error}`);
        }
      }
      
      return true;
    } catch (error) {
      this.context.logger.error(`Failed to load plugins: ${error}`);
      return false;
    }
  }

  /**
   * デバッグ情報を取得
   */
  public getDebugInfo(): any {
    const pluginInfo = Array.from(this.plugins.entries()).map(([id, reg]) => ({
      id,
      name: reg.info.name,
      version: reg.info.version,
      state: reg.state,
      error: reg.error,
      dependencies: reg.dependencies
    }));
    
    return {
      initialized: this.initialized,
      pluginCount: this.plugins.size,
      activePlugins: this.getPluginsByState(PluginState.ACTIVE).length,
      errorPlugins: this.getPluginsByState(PluginState.ERROR).length,
      config: this.config,
      plugins: pluginInfo
    };
  }
}

// シングルトンインスタンスをエクスポート
export default PluginRegistry.getInstance();