import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface ImageData {
  base64: string;
  file_name: string;
}

function App() {
  const [imageData, setImageData] = useState<ImageData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadImage = async () => {
      try {
        // WSL環境内でのパスに修正
        const path = "/home/wsluser/temp-image/plugin-viewers-image/LeavingColorfulColorado.png";
        const data = await invoke<ImageData>("load_image", { path });
        setImageData(data);
        setError(null);
      } catch (err) {
        setError(err as string);
        console.error("Failed to load image:", err);
      }
    };

    loadImage();
  }, []);

  return (
    <div className="container">
      <h1>Image Viewer</h1>
      {error && <div style={{ color: "red" }}>{error}</div>}
      {imageData && (
        <div>
          <h2>{imageData.file_name}</h2>
          <img 
            src={`data:image/png;base64,${imageData.base64}`}
            alt={imageData.file_name}
            style={{ maxWidth: "100%", height: "auto" }}
          />
        </div>
      )}
    </div>
  );
}

export default App;
