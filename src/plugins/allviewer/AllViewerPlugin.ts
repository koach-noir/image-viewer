// import { PluginInfo, PluginConfig, BasePlugin } from '../PluginInterface';
import { PluginInfo, BasePlugin } from '../PluginInterface';
import AllViewerUI from './AllViewerUI';
// import { invoke } from '@tauri-apps/api/core';
import ImageManager, { ImageData } from '../../core/ImageManager';
import ConfigManager from '../../config/ConfigManager';
import getEventSystem from '../../core/EventSystem';
import ResourceDefinitionManager, { ResourceConfig } from '../../config/ResourceDefinition';


// プラグインの設定型定義
interface AllViewerConfig {
  viewMode: 'grid' | 'list' | 'detail';
  thumbnailSize: number;
  showLabels: boolean;
  currentDirectory?: string;
}

// // リソース解決のレスポンス型を明示的に定義
// interface ResourceResolveResponse {
//   paths?: string[];
//   count?: number;
// }

// APIハンドラのタイプ定義
interface ApiHandler {
  name: string;
  handler: (args: Record<string, unknown>) => Promise<{
    success: boolean;
    error?: string;
    images?: ImageData[];
  }>;
}

class AllViewerPlugin extends BasePlugin {
  private eventSystem = getEventSystem();
  private currentImages: ImageData[] = [];
  private currentConfig: AllViewerConfig = {
    viewMode: 'grid',
    thumbnailSize: 150,
    showLabels: true,
  };

  getInfo(): PluginInfo {
    return {
      id: 'allviewer',
      name: 'ALL Viewing',
      version: '0.1.0',
      description: 'フレキシブルサムネイル＆ポップアップビューワー',
      author: 'Your Name',
    };
  }

  getUIComponent() {
    return AllViewerUI;
  }

  async initialize() {
    try {
      // プラグイン固有の初期設定をロード
      const savedConfig = await this.loadConfig();
      if (savedConfig) {
        this.currentConfig = savedConfig;
      }

      // イベント購読
      this.subscribeToEvents();

      return true;
    } catch (error) {
      console.error('AllViewer plugin initialization failed:', error);
      return false;
    }
  }

  async activate() {
    try {
        // リソース設定をデフォルトパスから読み込む
        const resourceConfig: ResourceConfig = await ResourceDefinitionManager.loadResourceConfig();

        // リソースからパスを解決
        const resolvedResources = await ResourceDefinitionManager.resolveResources(resourceConfig);

        // 解決されたパスから画像をロード
        const images = await ImageManager.loadImagesFromPaths(resolvedResources);
        
        // 読み込んだ画像をプラグインの状態に設定
        this.currentImages = images;

        // イベントを発行して画像が読み込まれたことを通知
        this.eventSystem.publish('allviewer:images_loaded', {
            count: images.length,
            resources: resolvedResources
        });

        return true;
    } catch (error) {
        console.error('Failed to activate AllViewer plugin:', error);
        return false;
    }
  }

  async deactivate() {
    // プラグインが非アクティブになった際の処理
    this.currentImages = [];
    return true;
  }

  // 設定をロード
  private async loadConfig(): Promise<AllViewerConfig | null> {
    try {
      const config = await ConfigManager.get('plugins.allviewer');
      return config as AllViewerConfig | null;
    } catch (error) {
      console.warn('Failed to load AllViewer config:', error);
      return null;
    }
  }

  // 設定を保存
  private async saveConfig(config: AllViewerConfig) {
    try {
      await ConfigManager.set('plugins.allviewer', config);
    } catch (error) {
      console.error('Failed to save AllViewer config:', error);
    }
  }

  // ディレクトリから画像をロード
  private async loadImagesFromDirectory(directory: string) {
    try {
      const result = await ImageManager.loadImagesFromConfig(directory);
      this.currentImages = result;
      
      // イベント発行
      this.eventSystem.publish('allviewer:images_loaded', {
        count: result.length,
        directory
      });
    } catch (error) {
      console.error('Failed to load images:', error);
    }
  }

  // イベント購読
  private subscribeToEvents() {
    // サムネイルサイズ変更イベント
    this.eventSystem.subscribe('allviewer:set_thumbnail_size', async (data: { size: number }) => {
      if (data.size) {
        this.currentConfig.thumbnailSize = data.size;
        await this.saveConfig(this.currentConfig);
      }
    });

    // ビューモード変更イベント
    this.eventSystem.subscribe('allviewer:set_view_mode', async (data: { mode: 'grid' | 'list' | 'detail' }) => {
      if (data.mode) {
        this.currentConfig.viewMode = data.mode;
        await this.saveConfig(this.currentConfig);
      }
    });
  }

  // プラグインAPIハンドラ
  getApiHandlers(): ApiHandler[] {
    return [
      {
        name: 'set_thumbnail_size',
        handler: async (args: Record<string, unknown>) => {
          const size = typeof args.size === 'number' ? args.size : null;
          if (size) {
            this.currentConfig.thumbnailSize = size;
            await this.saveConfig(this.currentConfig);
            return { success: true };
          }
          return { success: false, error: 'Invalid size' };
        }
      },
      {
        name: 'set_view_mode',
        handler: async (args: Record<string, unknown>) => {
          const mode = args.mode;
          if (mode === 'grid' || mode === 'list' || mode === 'detail') {
            this.currentConfig.viewMode = mode;
            await this.saveConfig(this.currentConfig);
            return { success: true };
          }
          return { success: false, error: 'Invalid mode' };
        }
      },
      {
        name: 'load_directory',
        handler: async (args: Record<string, unknown>) => {
          const directory = typeof args.directory === 'string' ? args.directory : null;
          if (directory) {
            try {
              this.currentConfig.currentDirectory = directory;
              await this.saveConfig(this.currentConfig);
              await this.loadImagesFromDirectory(directory);
              return { success: true, images: this.currentImages };
            } catch (error) {
              return { success: false, error: String(error) };
            }
          }
          return { success: false, error: 'Invalid directory' };
        }
      }
    ];
  }

  // データ取得メソッド
  getCurrentImages(): ImageData[] {
    return this.currentImages;
  }

  // 設定取得メソッド
  getCurrentConfig(): AllViewerConfig {
    return this.currentConfig;
  }
}

export default new AllViewerPlugin();
