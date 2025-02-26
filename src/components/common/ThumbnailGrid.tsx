import React, { useState, useEffect, useRef, useCallback } from 'react';
import { ImageData, ImageMetadata } from '../../core/ImageManager';
import imageManager from '../../core/ImageManager';

// CSSスタイル
const styles = {
  container: {
    width: '100%',
    height: '100%',
    overflow: 'auto',
    padding: '8px',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))',
    gap: '8px',
  },
  item: {
    position: 'relative' as const,
    aspectRatio: '1',
    borderRadius: '4px',
    overflow: 'hidden',
    boxShadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
    cursor: 'pointer',
    transition: 'transform 0.2s ease-in-out, box-shadow 0.2s ease-in-out',
  },
  itemHover: {
    transform: 'scale(1.02)',
    boxShadow: '0 4px 8px rgba(0, 0, 0, 0.2)',
  },
  itemSelected: {
    boxShadow: '0 0 0 3px #4a90e2, 0 4px 8px rgba(0, 0, 0, 0.2)',
  },
  thumbnail: {
    width: '100%',
    height: '100%',
    objectFit: 'cover' as const,
    backgroundColor: '#f0f0f0',
  },
  loadingPlaceholder: {
    width: '100%',
    height: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#f0f0f0',
    color: '#888',
    fontSize: '12px',
  },
  errorPlaceholder: {
    width: '100%',
    height: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#ffebee',
    color: '#d32f2f',
    fontSize: '12px',
  },
  label: {
    position: 'absolute' as const,
    bottom: 0,
    left: 0,
    right: 0,
    padding: '4px 8px',
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    color: 'white',
    fontSize: '12px',
    whiteSpace: 'nowrap' as const,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  loaderContainer: {
    width: '100%',
    display: 'flex',
    justifyContent: 'center',
    padding: '16px 0',
  },
  emptyMessage: {
    width: '100%',
    textAlign: 'center' as const,
    padding: '32px 0',
    color: '#888',
  },
};

export interface ThumbnailGridProps {
  images: (ImageData | ImageMetadata)[];
  selectedIndex?: number;
  imageSize?: number;
  showLabels?: boolean;
  loadOnScroll?: boolean;
  showPlaceholdersBeforeLoad?: boolean;
  onImageClick?: (index: number, image: ImageData | ImageMetadata) => void;
  onImageLoad?: (index: number, image: ImageData) => void;
  onLoadMore?: () => void;
  hasMoreImages?: boolean;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * サムネイルグリッドコンポーネント
 * 画像のサムネイルをグリッド表示する
 */
const ThumbnailGrid: React.FC<ThumbnailGridProps> = ({
  images,
  selectedIndex = -1,
  imageSize = 150,
  showLabels = true,
  loadOnScroll = true,
  showPlaceholdersBeforeLoad = true,
  onImageClick,
  onImageLoad,
  onLoadMore,
  hasMoreImages = false,
  className = '',
  style = {},
}) => {
  const [loadedImages, setLoadedImages] = useState<Record<number, ImageData | null>>({});
  const [loadingIndices, setLoadingIndices] = useState<Set<number>>(new Set());
  const [errorIndices, setErrorIndices] = useState<Set<number>>(new Set());
  const [hoverIndex, setHoverIndex] = useState<number>(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);
  const loaderRef = useRef<HTMLDivElement>(null);

  // 画像をロードする
  const loadImage = useCallback(async (index: number) => {
    // 既にロード中または成功・エラー済みの場合はスキップ
    if (loadingIndices.has(index) || loadedImages[index] !== undefined) {
      return;
    }

    // ロード状態を更新
    setLoadingIndices(prev => {
      const newSet = new Set(prev);
      newSet.add(index);
      return newSet;
    });

    try {
      const image = images[index];
      
      // 既にImageDataの場合はそのまま使用
      if ('base64' in image && image.base64) {
        setLoadedImages(prev => ({ ...prev, [index]: image as ImageData }));
        if (onImageLoad) onImageLoad(index, image as ImageData);
      } 
      // メタデータのみの場合はロード処理
      else {
        const path = 'path' in image ? image.path : image.metadata.path;
        const loadedImage = await imageManager.loadImage(path);
        setLoadedImages(prev => ({ ...prev, [index]: loadedImage }));
        if (onImageLoad) onImageLoad(index, loadedImage);
      }
    } catch (error) {
      console.error(`Failed to load image at index ${index}:`, error);
      setErrorIndices(prev => {
        const newSet = new Set(prev);
        newSet.add(index);
        return newSet;
      });
      setLoadedImages(prev => ({ ...prev, [index]: null }));
    } finally {
      setLoadingIndices(prev => {
        const newSet = new Set(prev);
        newSet.delete(index);
        return newSet;
      });
    }
  }, [images, loadedImages, loadingIndices, onImageLoad]);

  // 可視領域内の画像をロード
  const loadVisibleImages = useCallback(() => {
    if (!containerRef.current) return;

    const container = containerRef.current;
    const containerRect = container.getBoundingClientRect();
    const thumbnails = container.querySelectorAll('.thumbnail-item');

    // 可視領域に入っている各サムネイルをロード
    thumbnails.forEach((thumbnail, index) => {
      const rect = thumbnail.getBoundingClientRect();
      
      // 要素が可視領域内にあるか確認
      const isVisible = 
        rect.top <= containerRect.bottom &&
        rect.bottom >= containerRect.top &&
        rect.left <= containerRect.right &&
        rect.right >= containerRect.left;
      
      if (isVisible) {
        loadImage(index);
      }
    });
  }, [loadImage]);

  // スクロールイベント
  const handleScroll = useCallback(() => {
    if (loadOnScroll) {
      loadVisibleImages();
    }
  }, [loadOnScroll, loadVisibleImages]);

  // 「もっと読み込む」の Intersection Observer 設定
  useEffect(() => {
    if (!loaderRef.current || !onLoadMore || !hasMoreImages) return;

    const options = {
      root: containerRef.current,
      rootMargin: '100px',
      threshold: 0.1,
    };

    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        onLoadMore();
      }
    }, options);

    observer.observe(loaderRef.current);
    observerRef.current = observer;

    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect();
      }
    };
  }, [onLoadMore, hasMoreImages]);

  // 初期ロード
  useEffect(() => {
    loadVisibleImages();
  }, [images, loadVisibleImages]);

  // グリッドスタイルを計算
  const gridStyle = {
    ...styles.grid,
    gridTemplateColumns: `repeat(auto-fill, minmax(${imageSize}px, 1fr))`,
  };

  // サムネイルクリックハンドラ
  const handleImageClick = (index: number) => {
    if (onImageClick) {
      onImageClick(index, images[index]);
    }
  };

  return (
    <div
      ref={containerRef}
      className={`thumbnail-grid ${className}`}
      style={{ ...styles.container, ...style }}
      onScroll={handleScroll}
    >
      {images.length === 0 ? (
        <div style={styles.emptyMessage}>
          <p>画像がありません</p>
        </div>
      ) : (
        <div className="thumbnail-grid-content" style={gridStyle}>
          {images.map((image, index) => {
            const isLoaded = loadedImages[index] !== undefined;
            const isLoading = loadingIndices.has(index);
            const isError = errorIndices.has(index);
            const isSelected = index === selectedIndex;
            const isHovered = index === hoverIndex;
            
            // ファイル名の取得
            const fileName = 'fileName' in image 
              ? image.fileName 
              // : image.file_name || 'image';
              : 'image';
            
            return (
              <div
                key={`thumbnail-${index}`}
                className="thumbnail-item"
                style={{
                  ...styles.item,
                  ...(isSelected ? styles.itemSelected : {}),
                  ...(isHovered ? styles.itemHover : {}),
                }}
                onClick={() => handleImageClick(index)}
                onMouseEnter={() => setHoverIndex(index)}
                onMouseLeave={() => setHoverIndex(-1)}
              >
                {/* サムネイル画像 */}
                {isLoaded && loadedImages[index] ? (
                  <img
                    src={`data:image/png;base64,${loadedImages[index]?.base64}`}
                    alt={fileName}
                    style={styles.thumbnail}
                  />
                ) : isLoading ? (
                  <div style={styles.loadingPlaceholder}>
                    読み込み中...
                  </div>
                ) : isError ? (
                  <div style={styles.errorPlaceholder}>
                    読み込みエラー
                  </div>
                ) : showPlaceholdersBeforeLoad ? (
                  <div 
                    style={styles.loadingPlaceholder}
                    onClick={() => loadImage(index)}
                  >
                    クリックで読み込み
                  </div>
                ) : null}
                
                {/* ラベル表示 */}
                {showLabels && (
                  <div style={styles.label} title={fileName}>
                    {fileName}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
      
      {/* 「もっと読み込む」ローダー */}
      {hasMoreImages && onLoadMore && (
        <div ref={loaderRef} style={styles.loaderContainer}>
          読み込み中...
        </div>
      )}
    </div>
  );
};

export default ThumbnailGrid;
