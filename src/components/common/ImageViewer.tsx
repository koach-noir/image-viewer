import React, { useState, useEffect, useCallback, useRef } from 'react';
import { ImageData } from '../../core/ImageManager';

// CSSスタイル
const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    width: '100%',
    height: '100%',
    position: 'relative' as const,
  },
  imageContainer: {
    position: 'relative' as const,
    width: '100%',
    height: '100%',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    overflow: 'hidden',
  },
  image: {
    maxWidth: '100%',
    maxHeight: '100%',
    objectFit: 'contain' as const,
    userSelect: 'none' as const,
    transition: 'transform 0.1s ease-out',
  },
  zoomedImage: {
    cursor: 'grab',
  },
  loadingOverlay: {
    position: 'absolute' as const,
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    color: 'white',
    zIndex: 10,
  },
  errorOverlay: {
    position: 'absolute' as const,
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: 'rgba(255, 0, 0, 0.3)',
    color: 'white',
    zIndex: 10,
    padding: '1rem',
  },
  infoBar: {
    display: 'flex',
    justifyContent: 'space-between',
    width: '100%',
    padding: '0.5rem',
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    color: 'white',
    zIndex: 5,
  },
  controls: {
    position: 'absolute' as const,
    bottom: '1rem',
    left: '50%',
    transform: 'translateX(-50%)',
    display: 'flex',
    gap: '1rem',
    padding: '0.5rem 1rem',
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    borderRadius: '4px',
    zIndex: 5,
  },
  controlButton: {
    backgroundColor: 'transparent',
    border: 'none',
    color: 'white',
    cursor: 'pointer',
    fontSize: '1.5rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '2rem',
    height: '2rem',
    borderRadius: '50%',
  },
  navigationButton: {
    position: 'absolute' as const,
    top: '50%',
    transform: 'translateY(-50%)',
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    border: 'none',
    color: 'white',
    cursor: 'pointer',
    fontSize: '2rem',
    height: '3rem',
    width: '3rem',
    borderRadius: '50%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 5,
  },
  prevButton: {
    left: '1rem',
  },
  nextButton: {
    right: '1rem',
  },
};

export interface ImageViewerProps {
  image?: ImageData;
  isLoading?: boolean;
  error?: string | null;
  showControls?: boolean;
  showNavigation?: boolean;
  showInfo?: boolean;
  onPrevious?: () => void;
  onNext?: () => void;
  onClose?: () => void;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * 画像ビューワーコンポーネント
 * 画像の表示、ズーム、パン機能を提供
 */
const ImageViewer: React.FC<ImageViewerProps> = ({
  image,
  isLoading = false,
  error = null,
  showControls = true,
  showNavigation = false,
  showInfo = true,
  onPrevious,
  onNext,
  onClose,
  className = '',
  style = {},
}) => {
  const [zoom, setZoom] = useState(1);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const imageRef = useRef<HTMLImageElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // ズームリセット
  const resetZoom = useCallback(() => {
    setZoom(1);
    setPosition({ x: 0, y: 0 });
  }, []);

  // ズームイン
  const zoomIn = useCallback(() => {
    setZoom((prev) => Math.min(prev + 0.25, 5));
  }, []);

  // ズームアウト
  const zoomOut = useCallback(() => {
    setZoom((prev) => {
      const newZoom = Math.max(prev - 0.25, 0.5);
      if (newZoom === 1) {
        setPosition({ x: 0, y: 0 });
      }
      return newZoom;
    });
  }, []);

  // マウスホイールでのズーム
  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (e.deltaY < 0) {
      zoomIn();
    } else {
      zoomOut();
    }
    e.preventDefault();
  }, [zoomIn, zoomOut]);

  // ドラッグ開始
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (zoom > 1) {
      setIsDragging(true);
      setDragStart({
        x: e.clientX - position.x,
        y: e.clientY - position.y,
      });
    }
  }, [zoom, position]);

  // ドラッグ中
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isDragging) {
      setPosition({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      });
    }
  }, [isDragging, dragStart]);

  // ドラッグ終了
  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // ドラッグ中にマウスがコンテナ外に出た場合
  const handleMouseLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  // キーボードイベント
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowLeft':
          if (onPrevious) onPrevious();
          break;
        case 'ArrowRight':
          if (onNext) onNext();
          break;
        case 'Escape':
          if (onClose) onClose();
          break;
        case '+':
        case '=':
          zoomIn();
          break;
        case '-':
          zoomOut();
          break;
        case '0':
          resetZoom();
          break;
        default:
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [onPrevious, onNext, onClose, zoomIn, zoomOut, resetZoom]);

  // 画像が変わったらズームリセット
  useEffect(() => {
    resetZoom();
  }, [image, resetZoom]);

  return (
    <div
      ref={containerRef}
      className={`image-viewer ${className}`}
      style={{ ...styles.container, ...style }}
    >
      <div
        className="image-viewer-container"
        style={styles.imageContainer}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseLeave}
      >
        {image && (
          <img
            ref={imageRef}
            src={`data:image/png;base64,${image.base64}`}
            alt={image.fileName}
            style={{
              ...styles.image,
              ...(zoom > 1 ? styles.zoomedImage : {}),
              transform: `translate(${position.x}px, ${position.y}px) scale(${zoom})`,
            }}
            draggable={false}
          />
        )}

        {isLoading && (
          <div style={styles.loadingOverlay}>
            <p>読み込み中...</p>
          </div>
        )}

        {error && (
          <div style={styles.errorOverlay}>
            <p>{error}</p>
          </div>
        )}

        {showNavigation && (
          <>
            {onPrevious && (
              <button
                style={{ ...styles.navigationButton, ...styles.prevButton }}
                onClick={onPrevious}
                aria-label="前の画像"
              >
                &#9664;
              </button>
            )}
            {onNext && (
              <button
                style={{ ...styles.navigationButton, ...styles.nextButton }}
                onClick={onNext}
                aria-label="次の画像"
              >
                &#9654;
              </button>
            )}
          </>
        )}
      </div>

      {showInfo && image && (
        <div className="image-viewer-info" style={styles.infoBar}>
          <span>{image.fileName}</span>
          {image.metadata.dimensions && (
            <span>
              {image.metadata.dimensions.width} x {image.metadata.dimensions.height}
            </span>
          )}
        </div>
      )}

      {showControls && (
        <div className="image-viewer-controls" style={styles.controls}>
          <button
            style={styles.controlButton}
            onClick={zoomOut}
            aria-label="ズームアウト"
          >
            &#8722;
          </button>
          <button
            style={styles.controlButton}
            onClick={resetZoom}
            aria-label="ズームリセット"
          >
            &#8634;
          </button>
          <button
            style={styles.controlButton}
            onClick={zoomIn}
            aria-label="ズームイン"
          >
            &#43;
          </button>
        </div>
      )}
    </div>
  );
};

export default ImageViewer;
