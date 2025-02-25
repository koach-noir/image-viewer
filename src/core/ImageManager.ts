import { invoke } from '@tauri-apps/api/core';

/**
 * 画像メタデータのインターフェース
 */
export interface ImageMetadata {
  path: string;
  fileName: string;
  fileSize: number;
  dimensions?: {
    width: number;
    height: number;
  };
  dateCreated?: string;
  dateModified?: string;
}

/**
 * 画像データのインターフェース
 */
export interface ImageData {
  base64: string;
  fileName: string;
  metadata: ImageMetadata;
}

/**
 * 画像コレクションダイジェスト情報
 */
export interface ImageCollectionDigest {
  totalImages: number;
  totalSizeBytes: number;
}

/**
 * リソースフィルタのインターフェース
 */
export interface ResourceFilter {
  include: string[];
  exclude: string[];
}

/**
 * リソース設定のインターフェース
 */
export interface ResourceConfig {
  id: string;
  name: string;
  filters: ResourceFilter;
}

/**
 * パス解決結果のインターフェース
 */
export interface PathResolutionResult {
  paths: string[];
  count: number;
}

/**
 * 画像マネージャークラス
 * バックエンドの画像管理機能と連携するフロントエンド側のインターフェース
 */
export class ImageManager {
  private static instance: ImageManager;
  private cachedCollections: Map<string, ImageData[]> = new Map();

  /**
   * シングルトンインスタンスを取得
   */
  public static getInstance(): ImageManager {
    if (!ImageManager.instance) {
      ImageManager.instance = new ImageManager();
    }
    return ImageManager.instance;
  }

  /**
   * プライベートコンストラクタ（シングルトンパターン用）
   */
  private constructor() {
    // 初期化処理があればここに実装
  }

  /**
   * 設定に基づいてリソースを解決
   * @param config リソース設定
   * @returns パス解決結果
   */
  public async resolveResources(config: ResourceConfig): Promise<PathResolutionResult> {
    try {
      return await invoke<PathResolutionResult>('resolve_resources', { config });
    } catch (error) {
      console.error('Failed to resolve resources:', error);
      throw new Error(`リソース解決に失敗しました: ${error}`);
    }
  }

  /**
   * パスリストから画像を読み込む
   * @param paths 画像パスのリスト
   * @returns 画像コレクション
   */
  public async loadImagesFromPaths(paths: string[]): Promise<ImageData[]> {
    try {
      const collection = await invoke<{
        metadataList: ImageMetadata[];
      }>('load_images_from_paths', { paths });
      
      // 実際の画像データはまだロードされていないので、メタデータのみ返す
      return collection.metadataList.map(metadata => ({
        base64: '', // 初期値は空（実際の画像は必要に応じて後から読み込む）
        fileName: metadata.fileName,
        metadata
      }));
    } catch (error) {
      console.error('Failed to load images from paths:', error);
      throw new Error(`画像の読み込みに失敗しました: ${error}`);
    }
  }

  /**
   * 設定IDに基づいて画像コレクションを直接ロード
   * @param configId 設定ID
   * @returns 画像コレクション
   */
  public async loadImagesFromConfig(configId: string): Promise<ImageData[]> {
    try {
      const collection = await invoke<{
        metadataList: ImageMetadata[];
      }>('load_images_from_config', { configId });
      
      // 実際の画像データはまだロードされていないので、メタデータのみ返す
      return collection.metadataList.map(metadata => ({
        base64: '', // 初期値は空（実際の画像は必要に応じて後から読み込む）
        fileName: metadata.fileName,
        metadata
      }));
    } catch (error) {
      console.error(`Failed to load images from config ${configId}:`, error);
      throw new Error(`設定からの画像読み込みに失敗しました: ${error}`);
    }
  }

  /**
   * 特定のパスの画像を読み込む
   * @param path 画像のパス
   * @returns 画像データ
   */
  public async loadImage(path: string): Promise<ImageData> {
    try {
      return await invoke<ImageData>('load_image', { path });
    } catch (error) {
      console.error('Failed to load image:', error);
      throw new Error(`画像の読み込みに失敗しました: ${error}`);
    }
  }

  /**
   * インデックスで指定したコレクション内の画像を読み込む
   * @param collectionId コレクションID
   * @param index インデックス
   * @returns 画像データ
   */
  public async loadImageAtIndex(collectionId: string, index: number): Promise<ImageData> {
    try {
      return await invoke<ImageData>('load_image_at', { collectionId, index });
    } catch (error) {
      console.error(`Failed to load image at index ${index}:`, error);
      throw new Error(`インデックス ${index} の画像読み込みに失敗しました: ${error}`);
    }
  }

  /**
   * JSON設定ファイルを読み込む
   * @param path 設定ファイルのパス
   * @returns リソース設定
   */
  public async loadConfig(path: string): Promise<ResourceConfig> {
    try {
      return await invoke<ResourceConfig>('load_config', { path });
    } catch (error) {
      console.error('Failed to load config:', error);
      throw new Error(`設定ファイルの読み込みに失敗しました: ${error}`);
    }
  }

  /**
   * 設定をJSONファイルに保存
   * @param config リソース設定
   * @param path 保存先パス
   */
  public async saveConfig(config: ResourceConfig, path: string): Promise<void> {
    try {
      await invoke<void>('save_config', { config, path });
    } catch (error) {
      console.error('Failed to save config:', error);
      throw new Error(`設定ファイルの保存に失敗しました: ${error}`);
    }
  }

  /**
   * キャッシュをクリア
   */
  public async clearCache(): Promise<void> {
    try {
      await invoke<void>('clear_cache');
      this.cachedCollections.clear();
    } catch (error) {
      console.error('Failed to clear cache:', error);
      throw new Error(`キャッシュのクリアに失敗しました: ${error}`);
    }
  }

  /**
   * 特定の設定IDのキャッシュをクリア
   * @param configId 設定ID
   */
  public async clearConfigCache(configId: string): Promise<void> {
    try {
      await invoke<void>('clear_config_cache', { configId });
      this.cachedCollections.delete(configId);
    } catch (error) {
      console.error(`Failed to clear cache for config ${configId}:`, error);
      throw new Error(`設定 ${configId} のキャッシュクリアに失敗しました: ${error}`);
    }
  }

  /**
   * 指定した数のランダムな画像を取得
   * @param collectionId コレクションID
   * @param count 取得する画像数
   * @returns ランダムな画像のリスト
   */
  public async getRandomImages(collectionId: string, count: number): Promise<ImageData[]> {
    try {
      return await invoke<ImageData[]>('get_random_images', { collectionId, count });
    } catch (error) {
      console.error('Failed to get random images:', error);
      throw new Error(`ランダム画像の取得に失敗しました: ${error}`);
    }
  }
}

// デフォルトエクスポートをシングルトンインスタンスに設定
export default ImageManager.getInstance();
