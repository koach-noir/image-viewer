/**
 * プラグイン基本情報のインターフェース
 * プラグインの識別や表示に必要な情報を定義
 */
export interface PluginInfo {
    /** プラグインの一意識別子 */
    id: string;
    /** プラグインの表示名 */
    name: string;
    /** プラグインのバージョン */
    version: string;
    /** プラグインの説明 */
    description: string;
    /** 作者情報 */
    author: string;
    /** アイコン画像のパス（オプション） */
    iconPath?: string;
  }
  
  /**
   * メニュー項目のインターフェース
   * プラグインがメインメニューに追加するメニュー項目を定義
   */
  export interface MenuItem {
    /** メニュー項目の識別子 */
    id: string;
    /** メニュー項目の表示ラベル */
    label: string;
    /** メニュー項目のアイコン（オプション） */
    icon?: string;
    /** ショートカットキー（オプション） */
    shortcut?: string;
    /** クリック時のコールバック関数 */
    onClick: () => void;
    /** サブメニュー項目（オプション） */
    submenu?: MenuItem[];
    /** メニュー項目の有効/無効状態 */
    enabled?: boolean;
  }
  
  /**
   * キーバインディングのインターフェース
   * プラグインがサポートするキーボードショートカットを定義
   */
  export interface KeyBinding {
    /** キーバインディングの識別子 */
    id: string;
    /** キーコンビネーション（例: "Ctrl+S"） */
    key: string;
    /** キー押下時に実行されるコールバック関数 */
    onPress: () => void;
    /** キーバインディングの説明 */
    description?: string;
  }
  
  /**
   * プラグインコンテキストのインターフェース
   * コアシステムがプラグインに提供するサービスと機能を定義
   */
  export interface PluginContext {
    /** リソースマネージャーへのアクセス */
    resourceManager: any; // 後でリソースマネージャーインターフェースに置き換え
    /** イベントバスへのアクセス */
    eventBus: any; // 後でイベントバスインターフェースに置き換え
    /** 設定マネージャーへのアクセス */
    configManager: any; // 後で設定マネージャーインターフェースに置き換え
    /** ロガーへのアクセス */
    logger: {
      debug: (message: string) => void;
      info: (message: string) => void;
      warn: (message: string) => void;
      error: (message: string) => void;
    };
  }
  
  /**
   * プラグイン設定のインターフェース
   * プラグイン固有の設定を定義
   */
  export interface PluginConfig {
    /** プラグインの一意識別子 */
    pluginId: string;
    /** 設定データ（任意の形式） */
    data: Record<string, any>;
    /** プラグイン設定のバージョン */
    version?: string;
  }
  
  /**
   * プラグインインターフェース
   * すべてのプラグインが実装すべきインターフェース
   */
  export interface PluginInterface {
    /**
     * プラグイン情報を取得
     * @returns プラグイン基本情報
     */
    getInfo(): PluginInfo;
    
    /**
     * プラグインを初期化（アプリ起動時に呼び出される）
     * @param context プラグインコンテキスト
     * @returns 初期化が成功したかどうか
     */
    initialize(context: PluginContext): Promise<boolean>;
    
    /**
     * プラグインを有効化（ユーザーが選択時に呼び出される）
     * @returns 有効化が成功したかどうか
     */
    activate(): Promise<boolean>;
    
    /**
     * プラグインを無効化
     * @returns 無効化が成功したかどうか
     */
    deactivate(): Promise<boolean>;
    
    /**
     * プラグインのUIコンポーネントを取得
     * @returns Reactコンポーネント
     */
    getUIComponent(): React.ComponentType<any>;
    
    /**
     * プラグインが提供するメニュー項目を取得
     * @returns メニュー項目の配列（なければ空配列）
     */
    getMenuItems(): MenuItem[];
    
    /**
     * プラグインが定義するキーバインディングを取得
     * @returns キーバインディングの配列（なければ空配列）
     */
    getKeyBindings(): KeyBinding[];
    
    /**
     * プラグイン設定を取得
     * @returns プラグイン設定
     */
    getConfig(): PluginConfig;
    
    /**
     * プラグイン設定を更新
     * @param config 新しいプラグイン設定
     * @returns 更新が成功したかどうか
     */
    updateConfig(config: PluginConfig): Promise<boolean>;
  }
  
  /**
   * 基本的なプラグイン実装の抽象クラス
   * プラグイン開発を簡素化するためのベースクラス
   */
  export abstract class BasePlugin implements PluginInterface {
    protected context: PluginContext | null = null;
    protected config: PluginConfig | null = null;
    
    /**
     * プラグイン情報を取得（サブクラスで実装する必要がある）
     */
    abstract getInfo(): PluginInfo;
    
    /**
     * UIコンポーネントを取得（サブクラスで実装する必要がある）
     */
    abstract getUIComponent(): React.ComponentType<any>;
    
    /**
     * プラグインを初期化
     * @param context プラグインコンテキスト
     */
    async initialize(context: PluginContext): Promise<boolean> {
      this.context = context;
      
      try {
        // 設定を読み込む
        const pluginInfo = this.getInfo();
        if (context.configManager) {
          this.config = await context.configManager.getPluginConfig(pluginInfo.id);
        }
        
        return true;
      } catch (error) {
        context.logger.error(`Failed to initialize plugin ${this.getInfo().id}: ${error}`);
        return false;
      }
    }
    
    /**
     * プラグインを有効化（基本実装はそのまま成功を返す）
     */
    async activate(): Promise<boolean> {
      return true;
    }
    
    /**
     * プラグインを無効化（基本実装はそのまま成功を返す）
     */
    async deactivate(): Promise<boolean> {
      return true;
    }
    
    /**
     * メニュー項目を取得（基本実装は空配列を返す）
     */
    getMenuItems(): MenuItem[] {
      return [];
    }
    
    /**
     * キーバインディングを取得（基本実装は空配列を返す）
     */
    getKeyBindings(): KeyBinding[] {
      return [];
    }
    
    /**
     * プラグイン設定を取得
     */
    getConfig(): PluginConfig {
      if (!this.config) {
        const info = this.getInfo();
        this.config = {
          pluginId: info.id,
          data: {},
        };
      }
      return this.config;
    }
    
    /**
     * プラグイン設定を更新
     * @param config 新しい設定
     */
    async updateConfig(config: PluginConfig): Promise<boolean> {
      try {
        // 設定を検証
        if (config.pluginId !== this.getInfo().id) {
          throw new Error('Plugin ID mismatch');
        }
        
        this.config = { ...config };
        
        // 設定をストレージに保存
        if (this.context?.configManager) {
          await this.context.configManager.savePluginConfig(config);
        }
        
        return true;
      } catch (error) {
        if (this.context?.logger) {
          this.context.logger.error(`Failed to update plugin config: ${error}`);
        }
        return false;
      }
    }
  }
