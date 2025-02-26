// import { useState, useEffect, useCallback } from "react";
import { useState, useEffect } from "react";
// import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import PluginContainer from "./components/PluginContainer";
import { PluginLoader } from "./plugins/PluginLoader";
import { BasePlugin, PluginContext, PluginInfo } from "./plugins/PluginInterface";
import ImageManager from "./core/ImageManager";
import getEventSystem from "./core/EventSystem";
import ConfigManager from "./config/ConfigManager";

// サンプルプラグイン：AllViewing（実際のプロジェクトでは別ファイルに分離することをお勧めします）
class AllViewingPlugin extends BasePlugin {
  private imageViewer: React.ComponentType<any> | null = null;

  getInfo(): PluginInfo {
    return {
      id: "all-viewing",
      name: "ALL Viewing",
      version: "0.1.0",
      description: "フレキシブルサムネイル＆ポップアップビューワー",
      author: "Your Name",
    };
  }

  async initialize(context: PluginContext): Promise<boolean> {
    await super.initialize(context);
    
    // ここで必要な初期化処理を行う
    this.context?.logger.info("AllViewing plugin initialized");
    
    // 遅延インポートでUIコンポーネントを読み込む
    const { default: AllViewingUI } = await import("./components/common/ImageViewer");
    this.imageViewer = AllViewingUI;
    
    return true;
  }

  getUIComponent(): React.ComponentType<any> {
    if (!this.imageViewer) {
      // フォールバックUIコンポーネント
      return () => (
        <div className="plugin-loading">
          <p>Loading AllViewing plugin...</p>
        </div>
      );
    }
    
    return this.imageViewer;
  }
}

function App() {
  const [pluginLoader, setPluginLoader] = useState<PluginLoader | null>(null);
  // ここが修正ポイント: string | null から string | undefined に変更
  const [activePluginId, setActivePluginId] = useState<string | undefined>(undefined);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // プラグインシステムの初期化
  useEffect(() => {
    const initializePluginSystem = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        // 設定マネージャーの初期化
        await ConfigManager.initialize();
        
        // イベントシステムの取得
        const eventSystem = getEventSystem();
        
        // プラグインローダーのコンテキスト作成
        const context: PluginContext = {
          resourceManager: ImageManager,
          eventBus: eventSystem,
          configManager: ConfigManager,
          logger: {
            debug: (message: string) => console.debug(`[Plugin] ${message}`),
            info: (message: string) => console.info(`[Plugin] ${message}`),
            warn: (message: string) => console.warn(`[Plugin] ${message}`),
            error: (message: string) => console.error(`[Plugin] ${message}`),
          }
        };
        
        // プラグインローダーの初期化
        const loader = PluginLoader.getInstance(context, {
          autoActivatePlugins: ConfigManager.get('app.autoActivatePlugins', []),
          activateOnLoad: true,
        });
        
        // サンプルプラグインの登録
        await loader.registerPlugin(new AllViewingPlugin());
        
        // TODO: 外部プラグインのディスカバリーと読み込み
        // プロトタイプ段階ではここでサンプルプラグインを登録
        
        setPluginLoader(loader);
        
        // プラグイン情報の取得
        const plugins = loader.getAllPluginInfo();
        if (plugins.length > 0) {
          setActivePluginId(plugins[0].id);
        }
        
        setIsLoading(false);
      } catch (error) {
        console.error("Failed to initialize plugin system:", error);
        setError(`プラグインシステムの初期化に失敗しました: ${error}`);
        setIsLoading(false);
      }
    };
    
    initializePluginSystem();
  }, []);

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>画像ビューワー</h1>
      </header>
      
      <main className="app-main">
        {isLoading ? (
          <div className="loading-overlay">
            <p>読み込み中...</p>
          </div>
        ) : error ? (
          <div className="error-message">
            <p>{error}</p>
            <button onClick={() => setError(null)}>閉じる</button>
          </div>
        ) : pluginLoader ? (
          <PluginContainer
            pluginLoader={pluginLoader}
            defaultPluginId={activePluginId}
            showHeader={true}
            showPluginSelector={true}
            className="main-plugin-container"
          />
        ) : (
          <div className="welcome-message">
            <p>プラグインシステムが読み込まれていません。</p>
            <p>ページを再読み込みするか、管理者に連絡してください。</p>
          </div>
        )}
      </main>
      
      <footer className="app-footer">
        <p>© 2025 画像ビューワープラグインシステム</p>
      </footer>
    </div>
  );
}

export default App;
