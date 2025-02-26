// src/types/tauri.d.ts
import { ImageData } from '../core/ImageManager';

declare module '@tauri-apps/api/core' {
  interface InvokeCommands {
    'resolve_resources': {
      args: { config: any };
      return: { paths: string[]; count: number; };
    },
    'load_images_from_paths': {
      args: { paths: string[] };
      return: { metadataList: ImageData[]; };
    }
    // 他のコマンドも同様に定義
  }
}