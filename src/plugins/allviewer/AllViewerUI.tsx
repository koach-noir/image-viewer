import React, { useState, useEffect, useCallback } from 'react';
import getEventSystem from '../../core/EventSystem';
import { ImageData } from '../../core/ImageManager';
import AllViewerPlugin from './AllViewerPlugin';
import ImageViewer from '../../components/common/ImageViewer';
import ThumbnailGrid from '../../components/common/ThumbnailGrid';

// スタイル定義
const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    width: '100%',
    height: '100%',
  },
  toolbar: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px',
    backgroundColor: '#f5f5f5',
    borderBottom: '1px solid #ddd',
  },
  viewModeSelect: {
    marginRight: '16px',
  },
  gridContainer: {
    flex: 1,
    overflow: 'auto',
  },
  imageViewerContainer: {
    flex: 1,
    maxHeight: 'calc(100vh - 200px)', // ツールバーとステータスバーを考慮
  },
};

const AllViewerUI: React.FC = () => {
  const [images, setImages] = useState<ImageData[]>([]);
  const [selectedImageIndex, setSelectedImageIndex] = useState<number>(-1);
  const [viewMode, setViewMode] = useState<'grid' | 'viewer'>('grid');
  const [thumbnailSize, setThumbnailSize] = useState<number>(150);
  const [showLabels, setShowLabels] = useState<boolean>(true);

  // プラグインの設定を初期化
  useEffect(() => {
    const config = AllViewerPlugin.getCurrentConfig();
    setThumbnailSize(config.thumbnailSize);
    setShowLabels(config.showLabels);
  }, []);

  // 画像の読み込み
  useEffect(() => {
    const loadImages = async () => {
      const loadedImages = AllViewerPlugin.getCurrentImages();
      setImages(loadedImages);
      if (loadedImages.length > 0) {
        setSelectedImageIndex(0);
      }
    };

    loadImages();
  }, []);

  useEffect(() => {
    // 画像読み込みイベントを購読
    const unsubscribe = getEventSystem().subscribe('allviewer:images_loaded', (data) => {
      if (data && data.images) {
        setImages(data.images);
        if (data.images.length > 0) {
          setSelectedImageIndex(0);
        }
      }
    });
    
    return () => {
      // クリーンアップ時にイベント購読を解除
      unsubscribe();
    };
  }, []);

  // ビューモードの切り替え
  const handleViewModeChange = useCallback((mode: 'grid' | 'viewer') => {
    setViewMode(mode);
  }, []);

  // サムネイルクリック時のハンドラ
  const handleThumbnailClick = useCallback((index: number) => {
    setSelectedImageIndex(index);
    setViewMode('viewer');
  }, []);

  // サムネイルサイズ変更
  const handleThumbnailSizeChange = useCallback((size: number) => {
    setThumbnailSize(size);
    AllViewerPlugin.getApiHandlers()
      .find(handler => handler.name === 'set_thumbnail_size')
      ?.handler({ size });
  }, []);

  // ラベル表示切り替え
  const handleToggleLabels = useCallback(() => {
    const newShowLabels = !showLabels;
    setShowLabels(newShowLabels);
    AllViewerPlugin.getApiHandlers()
      .find(handler => handler.name === 'set_view_mode')
      ?.handler({ mode: newShowLabels ? 'grid' : 'list' });
  }, [showLabels]);

  // ビューワーからグリッドに戻る
  const handleBackToGrid = useCallback(() => {
    setViewMode('grid');
  }, []);

  return (
    <div style={styles.container}>
      {/* ツールバー */}
      <div style={styles.toolbar}>
        <div>
          <select
            style={styles.viewModeSelect}
            value={viewMode}
            onChange={(e) => handleViewModeChange(e.target.value as 'grid' | 'viewer')}
          >
            <option value="grid">サムネイル</option>
            <option value="viewer">ビューワー</option>
          </select>
          
          <label>
            サムネイルサイズ:
            <input
              type="range"
              min="50"
              max="300"
              value={thumbnailSize}
              onChange={(e) => handleThumbnailSizeChange(Number(e.target.value))}
            />
          </label>
          
          <label>
            <input
              type="checkbox"
              checked={showLabels}
              onChange={handleToggleLabels}
            />
            ラベル表示
          </label>
        </div>
      </div>

      {/* メインコンテンツ */}
      {viewMode === 'grid' ? (
        <div style={styles.gridContainer}>
          <ThumbnailGrid
            images={images}
            selectedIndex={selectedImageIndex}
            imageSize={thumbnailSize}
            showLabels={showLabels}
            onImageClick={handleThumbnailClick}
          />
        </div>
      ) : (
        <div style={styles.imageViewerContainer}>
          {selectedImageIndex !== -1 && (
            <ImageViewer
              image={images[selectedImageIndex]}
              showNavigation
              showControls
              showInfo
              onPrevious={() => setSelectedImageIndex(prev => Math.max(0, prev - 1))}
              onNext={() => setSelectedImageIndex(prev => Math.min(images.length - 1, prev + 1))}
              onClose={handleBackToGrid}
            />
          )}
        </div>
      )}
    </div>
  );
};

export default AllViewerUI;
