import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import getEventSystem from "./core/EventSystem";

// アプリケーション全体で共有する初期化処理
async function initializeApp() {
  // イベントシステムの初期化（デバッグモード有効）
  const eventSystem = getEventSystem(true);
  
  // アプリケーション初期化イベントを発行
  eventSystem.publish('app:initialized', {
    timestamp: Date.now(),
    environment: import.meta.env.MODE
  });
  
  // グローバルエラーハンドリング
  window.addEventListener('error', (event) => {
    eventSystem.publish('app:error', {
      message: event.message,
      source: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      error: event.error?.toString()
    });
  });
  
  // Reactアプリケーションのレンダリング
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

// アプリケーションの起動
initializeApp().catch(error => {
  console.error("Failed to initialize application:", error);
  
  // 初期化失敗時のフォールバックUI
  const rootElement = document.getElementById("root");
  if (rootElement) {
    rootElement.innerHTML = `
      <div style="padding: 20px; text-align: center;">
        <h1>アプリケーションの初期化に失敗しました</h1>
        <p>申し訳ありませんが、アプリケーションの起動中にエラーが発生しました。</p>
        <p>エラー: ${error?.message || error}</p>
        <button onclick="window.location.reload()">再読み込み</button>
      </div>
    `;
  }
});