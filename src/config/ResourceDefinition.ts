import { invoke } from "@tauri-apps/api/core";
import { exists, writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { appConfigDir, join } from '@tauri-apps/api/path';
import { mkdir } from '@tauri-apps/plugin-fs';

// リソースフィルターのインターフェース
export interface ResourceFilter {
  include: string[];
  exclude: string[];
}

// リソース設定のインターフェース
export interface ResourceConfig {
  id: string;
  name: string;
  filters: ResourceFilter;
}

// リソース定義管理クラス
export class ResourceDefinitionManager {
  // // 設定ファイルのデフォルト保存先
  // private static DEFAULT_CONFIG_PATH = 'resources.json';

  /**
   * リソース設定をJSONファイルに保存
   * @param config リソース設定
   * @param path オプションの保存パス（未指定の場合はデフォルトパス）
   */
  public static async saveResourceConfig(
    config: ResourceConfig, 
    path?: string
  ): Promise<void> {
    try {
      // アプリ固有の設定ディレクトリを取得
      const configDir = await appConfigDir();
      
      // 設定ディレクトリが存在することを確認
      await mkdir(configDir, { recursive: true });
      
      // 保存パスを決定（デフォルトはアプリ設定ディレクトリ内）
      const savePath = path || await join(configDir, 'resources.json');
      
      const jsonContent = JSON.stringify(config, null, 2);
      
      await writeTextFile(savePath, jsonContent);
    } catch (error) {
      console.error('Failed to save resource config:', error);
      throw new Error(`リソース設定の保存に失敗しました: ${error}`);
    }
  }

  /**
   * JSONファイルからリソース設定を読み込む
   * @param path オプションの読み込みパス（未指定の場合はデフォルトパス）
   * @returns リソース設定
   */
  public static async loadResourceConfig(
      path?: string
  ): Promise<ResourceConfig> {
      try {
          // アプリ固有の設定ディレクトリを取得
          const configDir = await appConfigDir();
          
          // 保存パスを決定（デフォルトはアプリ設定ディレクトリ内）
          const loadPath = path || await join(configDir, 'resources.json');
          
          // ファイルの存在を確認（オプション）
          const fileExists = await exists(loadPath);
          if (!fileExists) {
              // デフォルトの設定を返すか、エラーをスローする
              return this.createDefaultResourceConfig();
          }
          
          // ファイルを読み込む
          const jsonContent = await readTextFile(loadPath);
          
          // JSONをパース
          const config: ResourceConfig = JSON.parse(jsonContent);
          
          // バリデーション
          this.validateResourceConfig(config);
          
          return config;
      } catch (error) {
          console.error('Failed to load resource config:', error);
          
          // エラーハンドリング
          if (error instanceof Error && error.message.includes('forbidden path')) {
              // パス制限エラーの場合、デフォルト設定を返す
              return this.createDefaultResourceConfig();
          }
          
          throw new Error(`リソース設定の読み込みに失敗しました: ${error}`);
      }
  }
  
  // デフォルトのリソース設定を作成するヘルパーメソッド
  private static createDefaultResourceConfig(): ResourceConfig {
      return {
          id: 'default-resource-config',
          name: 'デフォルト画像リソース',
          filters: {
              include: [
                  '~/Pictures',  // ユーザーの画像フォルダ
                  '~/Documents/Images'  // ドキュメント内の画像フォルダ
              ],
              exclude: [
                  '~/Pictures/Private',  // プライベートフォルダを除外
                  '~/Documents/Images/Temp'  // 一時フォルダを除外
              ]
          }
      };
  }

  /**
   * リソース設定のバリデーション
   * @param config バリデーションするリソース設定
   */
  private static validateResourceConfig(config: ResourceConfig): void {
    // IDのチェック
    if (!config.id || config.id.trim() === '') {
      throw new Error('リソース設定のIDが無効です');
    }

    // 名前のチェック
    if (!config.name || config.name.trim() === '') {
      throw new Error('リソース設定の名前が無効です');
    }

    // フィルターのバリデーション
    if (!config.filters) {
      throw new Error('リソースフィルターが定義されていません');
    }

    // includeパスの検証
    if (!config.filters.include || config.filters.include.length === 0) {
      throw new Error('少なくとも1つのインクルードパスが必要です');
    }

    // パスの形式チェック（オプション）
    config.filters.include.forEach(path => {
      if (!path || path.trim() === '') {
        throw new Error('無効なインクルードパスが含まれています');
      }
    });

    // excludeパスの検証（オプション）
    if (config.filters.exclude) {
      config.filters.exclude.forEach(path => {
        if (!path || path.trim() === '') {
          throw new Error('無効な除外パスが含まれています');
        }
      });
    }
  }

  /**
   * バックエンドのリソース解決APIを呼び出す
   * @param config リソース設定
   * @returns 解決されたパスのリスト
   */
  public static async resolveResources(
    config: ResourceConfig
  ): Promise<string[]> {
    try {
      const result: { paths: string[] } = await invoke('resolve_resources', { config });
      return result.paths || [];
    } catch (error) {
      console.error('Failed to resolve resources:', error);
      throw new Error(`リソースの解決に失敗しました: ${error}`);
    }
  }

  /**
   * 新しいリソース設定を作成
   * @param params リソース設定のパラメータ
   * @returns 作成されたリソース設定
   */
  public static createResourceConfig(params: {
    id?: string;
    name?: string;
    includePaths: string[];
    excludePaths?: string[];
  }): ResourceConfig {
    // IDとタイトルのデフォルト生成
    const defaultId = `resource_${Date.now()}`;
    const defaultName = `リソース設定 ${new Date().toLocaleDateString()}`;

    return {
      id: params.id || defaultId,
      name: params.name || defaultName,
      filters: {
        include: params.includePaths,
        exclude: params.excludePaths || []
      }
    };
  }

  /**
   * 既存の設定を更新
   * @param existingConfig 既存のリソース設定
   * @param updates 更新するプロパティ
   * @returns 更新されたリソース設定
   */
  public static updateResourceConfig(
    existingConfig: ResourceConfig, 
    updates: Partial<ResourceConfig>
  ): ResourceConfig {
    const updatedConfig = { ...existingConfig, ...updates };
    
    // バリデーションを実行
    this.validateResourceConfig(updatedConfig);
    
    return updatedConfig;
  }

  /**
   * リソース設定のリストを管理するユーティリティ
   */
  private static resourceConfigList: ResourceConfig[] = [];

  /**
   * リソース設定リストに新しい設定を追加
   * @param config 追加するリソース設定
   */
  public static addResourceConfigToList(config: ResourceConfig): void {
    // 重複チェック
    const existingIndex = this.resourceConfigList.findIndex(
      existing => existing.id === config.id
    );

    if (existingIndex !== -1) {
      // 既存の設定を更新
      this.resourceConfigList[existingIndex] = config;
    } else {
      // 新規追加
      this.resourceConfigList.push(config);
    }
  }

  /**
   * リソース設定リストから設定を削除
   * @param configId 削除する設定のID
   */
  public static removeResourceConfigFromList(configId: string): void {
    const index = this.resourceConfigList.findIndex(
      config => config.id === configId
    );

    if (index !== -1) {
      this.resourceConfigList.splice(index, 1);
    }
  }

  /**
   * リソース設定リストを取得
   * @returns リソース設定のリスト
   */
  public static getResourceConfigList(): ResourceConfig[] {
    return [...this.resourceConfigList];
  }
}

// グローバルなリソース管理のためのシングルトンインスタンス
export default ResourceDefinitionManager;