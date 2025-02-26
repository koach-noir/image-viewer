// import React, { useState, useEffect, useCallback } from 'react';
import { useState, useEffect, useCallback } from 'react';
import './App.css';
import PluginContainer from './components/PluginContainer';
import { PluginLoader } from './plugins/PluginLoader';
// import { BasePlugin, PluginContext, PluginInfo } from './plugins/PluginInterface';
import { PluginContext, PluginInfo } from './plugins/PluginInterface';
import ImageManager from './core/ImageManager';
import getEventSystem from './core/EventSystem';
import ConfigManager from './config/ConfigManager';
import ResourceDefinitionManager from './config/ResourceDefinition';

// プラグインのダイナミックロード用のインポート関数
async function dynamicImportPlugin(pluginName: string) {
  try {
    switch (pluginName) {
      case 'allviewer':
        const { default: AllViewingPlugin } = await import('./plugins/allviewer/AllViewerPlugin');
        return AllViewingPlugin;
      case 'findme':
        const { default: FindMePlugin } = await import('./plugins/findme/FindMePlugin');
        return FindMePlugin;
      default:
        throw new Error(`Unknown plugin: ${pluginName}`);
    }
  } catch (error) {
    console.error(`Failed to import plugin ${pluginName}:`, error);
    throw error;
  }
}

function App() {
  const [pluginLoader, setPluginLoader] = useState<PluginLoader | null>(null);
  const [activePluginId, setActivePluginId] = useState<string | undefined>(undefined);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [availablePlugins, setAvailablePlugins] = useState<PluginInfo[]>([]);

  // プラグインシステムの初期化
  const initializePluginSystem = useCallback(async () => {
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
      
      // 利用可能なプラグインをダイナミックにロード
      const pluginNames = ['allviewer', 'findme']; // 注: 将来的には検出機能で動的に取得
      
      for (const pluginName of pluginNames) {
        try {
          const PluginClass = await dynamicImportPlugin(pluginName);
          const plugin = new PluginClass();
          await loader.registerPlugin(plugin);
        } catch (pluginError) {
          console.error(`Failed to load plugin ${pluginName}:`, pluginError);
          // プラグインの読み込みに失敗しても続行
        }
      }
      
      // 利用可能なプラグイン情報を取得
      const plugins = loader.getAllPluginInfo();
      setAvailablePlugins(plugins);
      
      // 最初のプラグインをアクティブに設定
      if (plugins.length > 0) {
        setActivePluginId(plugins[0].id);
      }
      
      setPluginLoader(loader);
      setIsLoading(false);
    } catch (initError) {
      console.error('Failed to initialize plugin system:', initError);
      setError(`プラグインシステムの初期化に失敗しました: ${initError}`);
      setIsLoading(false);
    }
  }, []);

  // 初期化処理の実行
  useEffect(() => {
    initializePluginSystem();
  }, [initializePluginSystem]);

  // プラグイン切り替えハンドラ
  const handlePluginChange = useCallback(async (pluginId: string) => {
    if (!pluginLoader) return;

    try {
      // 現在のアクティブなプラグインを無効化
      if (activePluginId) {
        await pluginLoader.deactivatePlugin(activePluginId);
      }

      // 新しいプラグインを有効化
      await pluginLoader.activatePlugin(pluginId);
      
      // アクティブなプラグインIDを更新
      setActivePluginId(pluginId);
    } catch (switchError) {
      console.error('Failed to switch plugin:', switchError);
      setError(`プラグインの切り替えに失敗しました: ${switchError}`);
    }
  }, [pluginLoader, activePluginId]);

  // リソース定義の初期化サンプル
  const initializeResourceDefinitions = useCallback(async () => {
    try {
      // デフォルトのリソース設定を作成
      const defaultResourceConfig = ResourceDefinitionManager.createResourceConfig({
        includePaths: [
          '~/Pictures', 
          '~/Documents/Images'
        ],
        excludePaths: [
          '~/Pictures/Private',
          '~/Documents/Images/Temp'
        ]
      });

      // リソース設定を保存
      await ResourceDefinitionManager.saveResourceConfig(defaultResourceConfig);
      
      // リソース設定リストに追加
      ResourceDefinitionManager.addResourceConfigToList(defaultResourceConfig);
    } catch (resError) {
      console.error('Failed to initialize resource definitions:', resError);
    }
  }, []);

  // リソース定義の初期化
  useEffect(() => {
    initializeResourceDefinitions();
  }, [initializeResourceDefinitions]);

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
            showPluginInfo={true}
            className="main-plugin-container"
            // プラグイン切り替えハンドラを追加
            onPluginChange={handlePluginChange}
            // 利用可能なプラグインリストを渡す
            availablePlugins={availablePlugins}
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
        <div className="plugin-status">
          {availablePlugins.length > 0 && (
            <span>
              プラグイン: {availablePlugins.length}個 
              (アクティブ: {availablePlugins.filter(p => 
                pluginLoader?.getPluginState(p.id) === 'active'
              ).length})
            </span>
          )}
        </div>
      </footer>
    </div>
  );
}

export default App;
