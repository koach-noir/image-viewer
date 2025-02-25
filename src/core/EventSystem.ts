/**
 * イベントシステム
 * アプリケーション内のコンポーネント間通信を管理するイベントバス
 */

// イベントハンドラーの型定義
export type EventHandler<T = any> = (data: T) => void;

// イベント購読の返り値（購読解除用関数）
export type Unsubscribe = () => void;

// イベントペイロードのインターフェース
export interface EventPayload<T = any> {
  // イベントタイプ/名前
  eventType: string;
  // イベントデータ
  data: T;
  // イベントの送信元（オプション）
  source?: string;
  // イベントの送信先（オプション、指定がなければブロードキャスト）
  target?: string;
  // タイムスタンプ
  timestamp: number;
}

/**
 * イベントバスクラス
 * イベントの発行と購読を管理
 */
export class EventBus {
  // イベントタイプごとのハンドラーマップ
  private handlers: Map<string, Set<EventHandler>> = new Map();
  // コンポーネントごとのハンドラーマップ
  private componentHandlers: Map<string, Map<string, Set<EventHandler>>> = new Map();
  // デバッグモードフラグ
  private debugMode: boolean = false;

  /**
   * コンストラクタ
   * @param debug デバッグモードを有効にするかどうか
   */
  constructor(debug: boolean = false) {
    this.debugMode = debug;
  }

  /**
   * デバッグモードを設定
   * @param enabled デバッグモードを有効にするかどうか
   */
  setDebugMode(enabled: boolean): void {
    this.debugMode = enabled;
  }

  /**
   * イベントを購読
   * @param eventType イベントタイプ
   * @param handler イベントハンドラー
   * @returns 購読解除用関数
   */
  subscribe<T = any>(eventType: string, handler: EventHandler<T>): Unsubscribe {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }

    this.handlers.get(eventType)!.add(handler as EventHandler);

    if (this.debugMode) {
      console.log(`[EventBus] Subscribed to event: ${eventType}`);
    }

    // 購読解除関数を返す
    return () => {
      this.unsubscribe(eventType, handler);
    };
  }

  /**
   * イベントの購読を解除
   * @param eventType イベントタイプ
   * @param handler イベントハンドラー
   */
  unsubscribe<T = any>(eventType: string, handler: EventHandler<T>): void {
    const handlers = this.handlers.get(eventType);
    if (handlers) {
      handlers.delete(handler as EventHandler);
      if (handlers.size === 0) {
        this.handlers.delete(eventType);
      }

      if (this.debugMode) {
        console.log(`[EventBus] Unsubscribed from event: ${eventType}`);
      }
    }
  }

  /**
   * 特定のコンポーネントへのイベントを購読
   * @param componentId コンポーネントID
   * @param eventType イベントタイプ
   * @param handler イベントハンドラー
   * @returns 購読解除用関数
   */
  subscribeComponent<T = any>(
    componentId: string,
    eventType: string,
    handler: EventHandler<T>
  ): Unsubscribe {
    if (!this.componentHandlers.has(componentId)) {
      this.componentHandlers.set(componentId, new Map());
    }

    const componentMap = this.componentHandlers.get(componentId)!;
    if (!componentMap.has(eventType)) {
      componentMap.set(eventType, new Set());
    }

    componentMap.get(eventType)!.add(handler as EventHandler);

    if (this.debugMode) {
      console.log(`[EventBus] Component '${componentId}' subscribed to event: ${eventType}`);
    }

    // 購読解除関数を返す
    return () => {
      this.unsubscribeComponent(componentId, eventType, handler);
    };
  }

  /**
   * コンポーネントのイベント購読を解除
   * @param componentId コンポーネントID
   * @param eventType イベントタイプ
   * @param handler イベントハンドラー
   */
  unsubscribeComponent<T = any>(
    componentId: string,
    eventType: string,
    handler: EventHandler<T>
  ): void {
    const componentMap = this.componentHandlers.get(componentId);
    if (componentMap) {
      const handlers = componentMap.get(eventType);
      if (handlers) {
        handlers.delete(handler as EventHandler);
        if (handlers.size === 0) {
          componentMap.delete(eventType);
        }
        if (componentMap.size === 0) {
          this.componentHandlers.delete(componentId);
        }

        if (this.debugMode) {
          console.log(`[EventBus] Component '${componentId}' unsubscribed from event: ${eventType}`);
        }
      }
    }
  }

  /**
   * コンポーネントのすべてのイベント購読を解除
   * @param componentId コンポーネントID
   */
  unsubscribeAllComponentEvents(componentId: string): void {
    if (this.componentHandlers.has(componentId)) {
      this.componentHandlers.delete(componentId);

      if (this.debugMode) {
        console.log(`[EventBus] Unsubscribed all events for component: ${componentId}`);
      }
    }
  }

  /**
   * イベントを発行
   * @param eventType イベントタイプ
   * @param data イベントデータ
   * @returns 発行が成功したかどうか
   */
  publish<T = any>(eventType: string, data: T): boolean {
    const payload: EventPayload<T> = {
      eventType,
      data,
      timestamp: Date.now(),
    };

    return this.dispatchEvent(payload);
  }

  /**
   * 特定のコンポーネントからイベントを発行
   * @param sourceId 送信元コンポーネントID
   * @param eventType イベントタイプ
   * @param data イベントデータ
   * @returns 発行が成功したかどうか
   */
  publishFrom<T = any>(sourceId: string, eventType: string, data: T): boolean {
    const payload: EventPayload<T> = {
      eventType,
      data,
      source: sourceId,
      timestamp: Date.now(),
    };

    return this.dispatchEvent(payload);
  }

  /**
   * 特定のコンポーネントへイベントを発行
   * @param targetId 送信先コンポーネントID
   * @param eventType イベントタイプ
   * @param data イベントデータ
   * @returns 発行が成功したかどうか
   */
  publishTo<T = any>(targetId: string, eventType: string, data: T): boolean {
    const payload: EventPayload<T> = {
      eventType,
      data,
      target: targetId,
      timestamp: Date.now(),
    };

    return this.dispatchEvent(payload);
  }

  /**
   * コンポーネント間でイベントを発行
   * @param sourceId 送信元コンポーネントID
   * @param targetId 送信先コンポーネントID
   * @param eventType イベントタイプ
   * @param data イベントデータ
   * @returns 発行が成功したかどうか
   */
  publishBetween<T = any>(
    sourceId: string,
    targetId: string,
    eventType: string,
    data: T
  ): boolean {
    const payload: EventPayload<T> = {
      eventType,
      data,
      source: sourceId,
      target: targetId,
      timestamp: Date.now(),
    };

    return this.dispatchEvent(payload);
  }

  /**
   * イベントをディスパッチ
   * @param payload イベントペイロード
   * @returns 配信が成功したかどうか
   */
  private dispatchEvent<T = any>(payload: EventPayload<T>): boolean {
    const { eventType, target } = payload;
    let delivered = false;

    // デバッグログ
    if (this.debugMode) {
      console.log(`[EventBus] Dispatching event: ${eventType}`, payload);
    }

    // グローバルハンドラーに配信
    const handlers = this.handlers.get(eventType);
    if (handlers) {
      try {
        handlers.forEach(handler => {
          try {
            handler(payload);
            delivered = true;
          } catch (err) {
            console.error(`[EventBus] Error in event handler for ${eventType}:`, err);
          }
        });
      } catch (err) {
        console.error(`[EventBus] Error iterating handlers for ${eventType}:`, err);
      }
    }

    // 特定のターゲットがある場合、そのコンポーネントのハンドラーに配信
    if (target) {
      const componentMap = this.componentHandlers.get(target);
      if (componentMap) {
        const targetHandlers = componentMap.get(eventType);
        if (targetHandlers) {
          try {
            targetHandlers.forEach(handler => {
              try {
                handler(payload);
                delivered = true;
              } catch (err) {
                console.error(`[EventBus] Error in component handler for ${eventType}:`, err);
              }
            });
          } catch (err) {
            console.error(`[EventBus] Error iterating component handlers for ${eventType}:`, err);
          }
        }
      }
    }

    return delivered;
  }

  /**
   * すべてのイベント購読を解除
   */
  clear(): void {
    this.handlers.clear();
    this.componentHandlers.clear();

    if (this.debugMode) {
      console.log(`[EventBus] All event subscriptions cleared`);
    }
  }
}

// シングルトンインスタンス
let instance: EventBus | null = null;

/**
 * イベントシステムのシングルトンインスタンスを取得
 * @param debug デバッグモードを有効にするかどうか
 * @returns EventBusインスタンス
 */
export function getEventSystem(debug?: boolean): EventBus {
  if (!instance) {
    instance = new EventBus(debug);
  }
  return instance;
}

/**
 * イベントシステムのインスタンスをリセット（主にテスト用）
 */
export function resetEventSystem(): void {
  if (instance) {
    instance.clear();
    instance = null;
  }
}

// デフォルトエクスポート
export default getEventSystem;