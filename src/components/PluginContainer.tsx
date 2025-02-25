import React, { useState, useEffect, useCallback, useRef } from 'react';
import { PluginLoader, PluginState } from '../plugins/PluginLoader';
import { PluginInterface, PluginInfo } from '../plugins/PluginInterface';

// スタイル定義
const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    width: '100%',
    height: '100%',
    overflow: 'hidden',
  },
  header: {
    padding: '8px 16px',
    borderBottom: '1px solid #ddd',
    backgroundColor: '#f5f5f5',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  title: {
    margin: 0,
    fontSize: '1.2rem',
    fontWeight: 500,
  },
  controls: {
    display: 'flex',
    gap: '8px',
  },
  button: {
    padding: '4px 8px',
    backgroundColor: '#f0f0f0',
    border: '1px solid #ccc',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.9rem',
  },
  content: {
    flex: 1,
    overflow: 'auto',
    position: 'relative' as const,
  },
  pluginSelector: {
    padding: '12px',
    borderBottom: '1px solid #eee',
    backgroundColor: '#f9f9f9',
  },
  select: {
    width: '100%',
    padding: '8px',
    borderRadius: '4px',
    border: '1px solid #ccc',
  },
  pluginInfo: {
    padding: '12px',
    backgroundColor: '#fff',
    borderBottom: '1px solid #eee',
    fontSize: '0.9rem',
  },
  infoItem: {
    margin: '4px 0',
  },
  error: {
    padding: '16px',
    backgroundColor: '#ffebee',
    color: '#d32f2f',
    margin: '16px',
    borderRadius: '4px',
    border: '1px solid #ffcdd2',
  },
  empty: {
    padding: '32px',
    textAlign: 'center' as const,
    color: '#666',
  },
  loading: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '32px',
    color: '#666',
  },
};

// ダークモードのスタイル
const darkStyles = {
  header: {
    backgroundColor: '#2a2a2a',
    borderBottom: '1px solid #444',
  },
  title: {
    color: '#eee',
  },
  button: {
    backgroundColor: '#333',
    border: '1px solid #555',
    color: '#eee',
  },
  pluginSelector: {
    backgroundColor: '#2d2d2d',
    borderBottom: '1px solid #444',
  },
  select: {
    backgroundColor: '#333',
    border: '1px solid #555',
    color: '#eee',
  },
  pluginInfo: {
    backgroundColor: '#222',
    borderBottom: '1px solid #444',
    color: '#ddd',
  },
  error: {
    backgroundColor: '#442726',
    color: '#ff6b6b',
    border: '1px solid #663838',
  },
  empty: {
    color: '#aaa',
  },
  loading: {
    color: '#aaa',
  },
};

interface PluginContainerProps {
  pluginLoader: PluginLoader;
  defaultPluginId?: string;
  showHeader?: boolean;
  showPluginSelector?: boolean;
  showPluginInfo?: boolean;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * プラグインコンテナコンポーネント
 * プラグインのUIを表示・管理するコンテナ
 */
const PluginContainer: React.FC<PluginContainerProps> = ({
  pluginLoader,
  defaultPluginId,
  showHeader = true,
  showPluginSelector = true,
  showPluginInfo = true,
  className = '',
  style = {},
}) => {
  // 状態管理
  const [activePluginId, setActivePluginId] = useState<string | null>(defaultPluginId || null);
  const [availablePlugins, setAvailablePlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [isDarkMode, setIsDarkMode] = useState<boolean>(false);
  
  // プラグインUIコンポーネント参照
  const pluginComponentRef = useRef<React.ReactNode | null>(null);
  
  // ダークモード検出
  useEffect(() => {
    const darkModeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    setIsDarkMode(darkModeMediaQuery.matches);
    
    const handler = (e: MediaQueryListEvent) => setIsDarkMode(e.matches);
    darkModeMediaQuery.addEventListener('change', handler);
    
    return () => {
      darkModeMediaQuery.removeEventListener('change', handler);
    };
  }, []);
  
  // マージされたスタイル
  const mergedStyles = isDarkMode
    ? Object.fromEntries(
        Object.entries(styles).map(([key, value]) => [
          key,
          { ...value, ...(darkStyles[key as keyof typeof darkStyles] || {}) }
        ])
      )
    : styles;
  
  // プラグインリストの読み込み
  const loadPlugins = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      
      // すべてのプラグイン情報を取得
      const plugins = pluginLoader.getAllPluginInfo();
      setAvailablePlugins(plugins);
      
      // デフォルトプラグインがなく、プラグインが存在する場合は最初のプラグインをアクティブに
      if (!activePluginId && plugins.length > 0) {
        setActivePluginId(plugins[0].id);
      }
      
      setLoading(false);
    } catch (err) {
      setLoading(false);
      setError(`Failed to load plugins: ${err}`);
      console.error('Failed to load plugins:', err);
    }
  }, [pluginLoader, activePluginId]);
  
  // プラグインの有効化
  const activatePlugin = useCallback(async (pluginId: string) => {
    try {
      setError(null);
      
      // 現在のプラグインを無効化
      if (activePluginId) {
        await pluginLoader.deactivatePlugin(activePluginId);
      }
      
      // 新しいプラグインを有効化
      const success = await pluginLoader.activatePlugin(pluginId);
      
      if (success) {
        setActivePluginId(pluginId);
      } else {
        const registration = pluginLoader.getPluginRegistration(pluginId);
        setError(`Failed to activate plugin: ${registration?.error || 'Unknown error'}`);
      }
    } catch (err) {
      setError(`Error activating plugin: ${err}`);
      console.error('Error activating plugin:', err);
    }
  }, [pluginLoader, activePluginId]);
  
  // プラグインの選択変更ハンドラ
  const handlePluginChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const pluginId = e.target.value;
    activatePlugin(pluginId);
  }, [activatePlugin]);
  
  // 初期ロード
  useEffect(() => {
    loadPlugins();
    
    // プラグインイベントの購読
    const onPluginRegistered = () => loadPlugins();
    const onPluginUnregistered = () => loadPlugins();
    
    pluginLoader.on('plugin:registered', onPluginRegistered);
    pluginLoader.on('plugin:unregistered', onPluginUnregistered);
    
    return () => {
      pluginLoader.off('plugin:registered', onPluginRegistered);
      pluginLoader.off('plugin:unregistered', onPluginUnregistered);
    };
  }, [pluginLoader, loadPlugins]);
  
  // 選択されたプラグインの有効化
  useEffect(() => {
    if (activePluginId) {
      const pluginState = pluginLoader.getPluginState(activePluginId);
      if (pluginState !== PluginState.ACTIVE) {
        activatePlugin(activePluginId);
      }
    }
  }, [pluginLoader, activePluginId, activatePlugin]);
  
  // 現在のプラグインコンポーネントを取得
  const renderActivePlugin = useCallback(() => {
    if (!activePluginId) return null;
    
    const plugin = pluginLoader.getPlugin(activePluginId);
    if (!plugin) return null;
    
    const pluginState = pluginLoader.getPluginState(activePluginId);
    if (pluginState !== PluginState.ACTIVE) return null;
    
    try {
      const PluginComponent = plugin.getUIComponent();
      pluginComponentRef.current = <PluginComponent />;
      return pluginComponentRef.current;
    } catch (err) {
      console.error('Error rendering plugin component:', err);
      return (
        <div style={mergedStyles.error}>
          Error rendering plugin component: {String(err)}
        </div>
      );
    }
  }, [pluginLoader, activePluginId, mergedStyles.error]);
  
  // 現在のプラグイン情報を取得
  const getActivePluginInfo = useCallback(() => {
    if (!activePluginId) return null;
    
    const plugin = pluginLoader.getPlugin(activePluginId);
    return plugin ? plugin.getInfo() : null;
  }, [pluginLoader, activePluginId]);
  
  // 現在のプラグイン情報
  const activePluginInfo = getActivePluginInfo();
  
  // プラグイン情報表示
  const renderPluginInfo = () => {
    if (!activePluginInfo) return null;
    
    return (
      <div style={mergedStyles.pluginInfo}>
        <div style={mergedStyles.infoItem}>
          <strong>Name:</strong> {activePluginInfo.name}
        </div>
        <div style={mergedStyles.infoItem}>
          <strong>Version:</strong> {activePluginInfo.version}
        </div>
        <div style={mergedStyles.infoItem}>
          <strong>Author:</strong> {activePluginInfo.author}
        </div>
        <div style={mergedStyles.infoItem}>
          <strong>Description:</strong> {activePluginInfo.description}
        </div>
      </div>
    );
  };

  return (
    <div 
      className={`plugin-container ${className}`}
      style={{ ...mergedStyles.container, ...style }}
    >
      {/* ヘッダー */}
      {showHeader && (
        <div style={mergedStyles.header}>
          <h2 style={mergedStyles.title}>
            {activePluginInfo ? activePluginInfo.name : 'Plugin Viewer'}
          </h2>
          <div style={mergedStyles.controls}>
            <button
              style={mergedStyles.button}
              onClick={loadPlugins}
              disabled={loading}
            >
              Reload Plugins
            </button>
          </div>
        </div>
      )}
      
      {/* プラグイン選択 */}
      {showPluginSelector && (
        <div style={mergedStyles.pluginSelector}>
          <select
            style={mergedStyles.select}
            value={activePluginId || ''}
            onChange={handlePluginChange}
            disabled={loading || availablePlugins.length === 0}
          >
            {availablePlugins.length === 0 ? (
              <option value="">No plugins available</option>
            ) : (
              availablePlugins.map(plugin => (
                <option key={plugin.id} value={plugin.id}>
                  {plugin.name}
                </option>
              ))
            )}
          </select>
        </div>
      )}
      
      {/* プラグイン情報表示 */}
      {showPluginInfo && activePluginInfo && renderPluginInfo()}
      
      {/* エラー表示 */}
      {error && (
        <div style={mergedStyles.error}>
          {error}
        </div>
      )}
      
      {/* プラグインコンテンツ */}
      <div style={mergedStyles.content}>
        {loading ? (
          <div style={mergedStyles.loading}>Loading plugins...</div>
        ) : availablePlugins.length === 0 ? (
          <div style={mergedStyles.empty}>No plugins installed</div>
        ) : !activePluginId ? (
          <div style={mergedStyles.empty}>No plugin selected</div>
        ) : (
          renderActivePlugin()
        )}
      </div>
    </div>
  );
};

export default PluginContainer;
