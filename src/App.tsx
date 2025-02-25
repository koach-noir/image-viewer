import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface ImageData {
  base64: string;
  file_name: string;
}

interface DirectoryContent {
  images: string[];
  current_index: number;
}

function App() {
  const [imageData, setImageData] = useState<ImageData | null>(null);
  const [directoryPath, setDirectoryPath] = useState<string>("");
  const [directoryContent, setDirectoryContent] = useState<DirectoryContent | null>(null);
  const [currentIndex, setCurrentIndex] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // 画像のロード関数
  const loadImage = useCallback(async (path: string) => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await invoke<ImageData>("load_image", { path });
      setImageData(data);
    } catch (err) {
      setError(`Failed to load image: ${err}`);
      console.error("Failed to load image:", err);
      setImageData(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // ディレクトリ内の画像を取得する関数
  const loadDirectoryImages = async (dirPath: string) => {
    if (!dirPath.trim()) {
      setError("Please enter a directory path");
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      const content = await invoke<DirectoryContent>("get_directory_images", { dirPath });
      
      if (content.images.length === 0) {
        setError("No images found in the directory");
        setDirectoryContent(null);
        setImageData(null);
        return;
      }
      
      setDirectoryContent(content);
      setCurrentIndex(0);
      
      // 最初の画像を読み込む
      await loadImage(content.images[0]);
    } catch (err) {
      setError(`Failed to read directory: ${err}`);
      console.error("Failed to read directory:", err);
      setDirectoryContent(null);
      setImageData(null);
    } finally {
      setIsLoading(false);
    }
  };

  // 前の画像に移動
  const goToPreviousImage = useCallback(() => {
    if (!directoryContent || directoryContent.images.length === 0) return;
    
    const newIndex = currentIndex > 0 
      ? currentIndex - 1 
      : directoryContent.images.length - 1;
    
    setCurrentIndex(newIndex);
    loadImage(directoryContent.images[newIndex]);
  }, [directoryContent, currentIndex, loadImage]);

  // 次の画像に移動
  const goToNextImage = useCallback(() => {
    if (!directoryContent || directoryContent.images.length === 0) return;
    
    const newIndex = currentIndex < directoryContent.images.length - 1 
      ? currentIndex + 1 
      : 0;
    
    setCurrentIndex(newIndex);
    loadImage(directoryContent.images[newIndex]);
  }, [directoryContent, currentIndex, loadImage]);

  // キーボード操作のイベントリスナー
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "ArrowLeft") {
        goToPreviousImage();
      } else if (event.key === "ArrowRight") {
        goToNextImage();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [goToPreviousImage, goToNextImage]);

  // フォームの送信ハンドラ
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    loadDirectoryImages(directoryPath);
  };

  return (
    <div className="container">
      <h1>Image Viewer</h1>
      
      <form onSubmit={handleSubmit} className="directory-form">
        <div className="input-group">
          <input
            type="text"
            value={directoryPath}
            onChange={(e) => setDirectoryPath(e.target.value)}
            placeholder="Enter directory path"
            className="directory-input"
          />
          <button type="submit" disabled={isLoading}>
            {isLoading ? "Loading..." : "Load Images"}
          </button>
        </div>
      </form>

      {error && <div className="error-message">{error}</div>}

      <div className="image-container">
        {directoryContent && directoryContent.images.length > 0 && (
          <div className="image-navigation">
            <button onClick={goToPreviousImage} disabled={isLoading}>
              ← Previous
            </button>
            <span className="image-counter">
              {currentIndex + 1} / {directoryContent.images.length}
            </span>
            <button onClick={goToNextImage} disabled={isLoading}>
              Next →
            </button>
          </div>
        )}

        {imageData && (
          <div className="image-view">
            <h2>{imageData.file_name}</h2>
            <img
              src={`data:image/png;base64,${imageData.base64}`}
              alt={imageData.file_name}
              className="image-display"
            />
          </div>
        )}

        {isLoading && <div className="loading">Loading...</div>}
      </div>
    </div>
  );
}

export default App;
