// allviewer/ui.rs - AllViewerプラグインのUI関連コード

/// フロントエンドコードを生成
/// この関数は、Rustバックエンドからフロントエンドに渡される
/// UI関連のJavaScriptコードを返します。
pub fn get_frontend_code() -> String {
    // 注: 実際のアプリケーションでは、この部分はフロントエンド側に実装を
    // 移動し、Rustからのデータ提供に特化することを推奨します。
    // ここでは例としてシンプルな構造を示します。
    
    // 基本的なビューワーコンポーネントを定義
    let code = r#"
    // AllViewer UI コンポーネント
    // 注: このコードはフロントエンドのバンドラーでビルドされるべきですが、
    // ここではシンプルな実装例を示します。
    
    class AllViewerUI {
        constructor(container, config) {
            this.container = container;
            this.config = config || {
                viewMode: 'grid',
                thumbnailSize: 150,
                showLabels: true
            };
            
            this.images = [];
            this.selectedIndex = -1;
            
            this.initialize();
        }
        
        initialize() {
            // コンテナの初期化
            this.container.innerHTML = '';
            this.container.className = 'allviewer-container';
            
            // ツールバーの作成
            this.createToolbar();
            
            // メインビューの作成
            this.mainView = document.createElement('div');
            this.mainView.className = 'allviewer-main-view';
            this.container.appendChild(this.mainView);
            
            // ステータスバーの作成
            this.createStatusBar();
            
            // 初期ビューモードの適用
            this.setViewMode(this.config.viewMode);
        }
        
        createToolbar() {
            const toolbar = document.createElement('div');
            toolbar.className = 'allviewer-toolbar';
            
            // ビューモード切り替えボタン
            const viewModeBtn = document.createElement('button');
            viewModeBtn.textContent = 'View Mode';
            viewModeBtn.onclick = () => {
                const modes = ['grid', 'list', 'detail'];
                const currentIndex = modes.indexOf(this.config.viewMode);
                const nextIndex = (currentIndex + 1) % modes.length;
                this.setViewMode(modes[nextIndex]);
            };
            toolbar.appendChild(viewModeBtn);
            
            // サムネイルサイズスライダー
            const sizeLabel = document.createElement('span');
            sizeLabel.textContent = 'Size:';
            toolbar.appendChild(sizeLabel);
            
            const sizeSlider = document.createElement('input');
            sizeSlider.type = 'range';
            sizeSlider.min = '50';
            sizeSlider.max = '300';
            sizeSlider.value = this.config.thumbnailSize;
            sizeSlider.oninput = (e) => {
                this.setThumbnailSize(parseInt(e.target.value, 10));
            };
            toolbar.appendChild(sizeSlider);
            
            // ラベル表示トグル
            const labelToggle = document.createElement('input');
            labelToggle.type = 'checkbox';
            labelToggle.checked = this.config.showLabels;
            labelToggle.onchange = (e) => {
                this.setShowLabels(e.target.checked);
            };
            toolbar.appendChild(labelToggle);
            
            const labelText = document.createElement('span');
            labelText.textContent = 'Show Labels';
            toolbar.appendChild(labelText);
            
            this.container.appendChild(toolbar);
        }
        
        createStatusBar() {
            const statusBar = document.createElement('div');
            statusBar.className = 'allviewer-status-bar';
            
            // 画像カウンター
            this.imageCounter = document.createElement('span');
            this.updateImageCounter();
            statusBar.appendChild(this.imageCounter);
            
            this.container.appendChild(statusBar);
        }
        
        updateImageCounter() {
            if (!this.imageCounter) return;
            
            const total = this.images.length;
            const current = this.selectedIndex >= 0 ? this.selectedIndex + 1 : 0;
            this.imageCounter.textContent = `${current} / ${total} images`;
        }
        
        setViewMode(mode) {
            this.config.viewMode = mode;
            this.mainView.className = `allviewer-main-view mode-${mode}`;
            this.renderImages();
            
            // バックエンドに通知
            if (window.invoke) {
                window.invoke('plugin:allviewer:set_view_mode', { mode });
            }
        }
        
        setThumbnailSize(size) {
            this.config.thumbnailSize = size;
            
            // サムネイルサイズを更新
            const thumbnails = this.mainView.querySelectorAll('.image-thumbnail');
            thumbnails.forEach(thumb => {
                thumb.style.width = `${size}px`;
                thumb.style.height = `${size}px`;
            });
            
            // バックエンドに通知
            if (window.invoke) {
                window.invoke('plugin:allviewer:set_thumbnail_size', { size });
            }
        }
        
        setShowLabels(show) {
            this.config.showLabels = show;
            
            // ラベル表示を更新
            const labels = this.mainView.querySelectorAll('.image-label');
            labels.forEach(label => {
                label.style.display = show ? 'block' : 'none';
            });
            
            // バックエンドに通知
            if (window.invoke) {
                window.invoke('plugin:allviewer:toggle_labels');
            }
        }
        
        setImages(images) {
            this.images = images || [];
            this.selectedIndex = this.images.length > 0 ? 0 : -1;
            this.renderImages();
            this.updateImageCounter();
        }
        
        renderImages() {
            this.mainView.innerHTML = '';
            
            if (this.images.length === 0) {
                const emptyMsg = document.createElement('div');
                emptyMsg.className = 'empty-message';
                emptyMsg.textContent = 'No images to display';
                this.mainView.appendChild(emptyMsg);
                return;
            }
            
            if (this.config.viewMode === 'grid') {
                this.renderGridView();
            } else if (this.config.viewMode === 'list') {
                this.renderListView();
            } else {
                this.renderDetailView();
            }
        }
        
        renderGridView() {
            const grid = document.createElement('div');
            grid.className = 'image-grid';
            
            this.images.forEach((image, index) => {
                const thumbnail = this.createThumbnail(image, index);
                grid.appendChild(thumbnail);
            });
            
            this.mainView.appendChild(grid);
        }
        
        renderListView() {
            const list = document.createElement('div');
            list.className = 'image-list';
            
            this.images.forEach((image, index) => {
                const item = document.createElement('div');
                item.className = 'image-list-item';
                if (index === this.selectedIndex) {
                    item.classList.add('selected');
                }
                
                const thumbnail = this.createThumbnail(image, index, true);
                thumbnail.style.width = '50px';
                thumbnail.style.height = '50px';
                
                const details = document.createElement('div');
                details.className = 'image-details';
                details.innerHTML = `
                    <div class="image-name">${image.fileName}</div>
                    <div class="image-info">${image.width}x${image.height} - ${this.formatSize(image.fileSize)}</div>
                `;
                
                item.appendChild(thumbnail);
                item.appendChild(details);
                item.onclick = () => this.selectImage(index);
                
                list.appendChild(item);
            });
            
            this.mainView.appendChild(list);
        }
        
        renderDetailView() {
            if (this.selectedIndex < 0 || !this.images[this.selectedIndex]) {
                return;
            }
            
            const image = this.images[this.selectedIndex];
            
            const detailView = document.createElement('div');
            detailView.className = 'image-detail-view';
            
            // 画像表示
            const imgElement = document.createElement('img');
            imgElement.src = image.url || '';
            imgElement.alt = image.fileName;
            imgElement.className = 'detail-image';
            
            // メタデータ表示
            const metadata = document.createElement('div');
            metadata.className = 'image-metadata';
            metadata.innerHTML = `
                <table>
                    <tr><th>File name:</th><td>${image.fileName}</td></tr>
                    <tr><th>Dimensions:</th><td>${image.width}x${image.height}</td></tr>
                    <tr><th>File size:</th><td>${this.formatSize(image.fileSize)}</td></tr>
                    <tr><th>Created:</th><td>${image.dateCreated || 'Unknown'}</td></tr>
                    <tr><th>Modified:</th><td>${image.dateModified || 'Unknown'}</td></tr>
                </table>
            `;
            
            // ナビゲーションボタン
            const nav = document.createElement('div');
            nav.className = 'detail-navigation';
            
            const prevBtn = document.createElement('button');
            prevBtn.textContent = 'Previous';
            prevBtn.onclick = () => this.navigateImages(-1);
            prevBtn.disabled = this.selectedIndex <= 0;
            
            const nextBtn = document.createElement('button');
            nextBtn.textContent = 'Next';
            nextBtn.onclick = () => this.navigateImages(1);
            nextBtn.disabled = this.selectedIndex >= this.images.length - 1;
            
            nav.appendChild(prevBtn);
            nav.appendChild(nextBtn);
            
            detailView.appendChild(imgElement);
            detailView.appendChild(metadata);
            detailView.appendChild(nav);
            
            this.mainView.appendChild(detailView);
        }
        
        createThumbnail(image, index, simple = false) {
            const thumb = document.createElement('div');
            thumb.className = 'image-thumbnail';
            thumb.style.width = `${this.config.thumbnailSize}px`;
            thumb.style.height = `${this.config.thumbnailSize}px`;
            
            if (index === this.selectedIndex) {
                thumb.classList.add('selected');
            }
            
            const img = document.createElement('img');
            img.src = image.thumbnailUrl || image.url || '';
            img.alt = image.fileName;
            img.className = 'thumbnail-img';
            
            thumb.appendChild(img);
            
            if (!simple && this.config.showLabels) {
                const label = document.createElement('div');
                label.className = 'image-label';
                label.textContent = image.fileName;
                thumb.appendChild(label);
            }
            
            thumb.onclick = () => this.selectImage(index);
            
            return thumb;
        }
        
        selectImage(index) {
            if (index < 0 || index >= this.images.length) return;
            
            this.selectedIndex = index;
            this.updateImageCounter();
            
            // デフォルトビューは変更しない、詳細ビューの場合は再レンダリング
            if (this.config.viewMode === 'detail') {
                this.renderImages();
            } else {
                // 選択状態の更新
                const thumbnails = this.mainView.querySelectorAll('.image-thumbnail, .image-list-item');
                thumbnails.forEach((thumb, idx) => {
                    if (idx === index) {
                        thumb.classList.add('selected');
                    } else {
                        thumb.classList.remove('selected');
                    }
                });
            }
            
            // 選択イベントを発行
            if (window.invoke) {
                window.invoke('plugin:allviewer:select_image', { index });
            }
        }
        
        navigateImages(direction) {
            const newIndex = this.selectedIndex + direction;
            if (newIndex >= 0 && newIndex < this.images.length) {
                this.selectImage(newIndex);
            }
        }
        
        formatSize(bytes) {
            if (bytes < 1024) return `${bytes} B`;
            if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
            if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
            return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
        }
    }
    
    // モジュールとしてエクスポート
    export default {
        initialize: function(container, config) {
            return new AllViewerUI(container, config);
        }
    };
    "#;
    
    code.to_string()
}

// 実際のCSSスタイルを提供するための関数
pub fn get_css_styles() -> String {
    r#"
    /* AllViewer CSS スタイル */
    .allviewer-container {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
        overflow: hidden;
    }
    
    .allviewer-toolbar {
        padding: 8px;
        display: flex;
        align-items: center;
        background-color: #f5f5f5;
        border-bottom: 1px solid #ddd;
    }
    
    .allviewer-toolbar button {
        margin-right: 8px;
        padding: 4px 8px;
        background-color: #fff;
        border: 1px solid #ccc;
        border-radius: 4px;
        cursor: pointer;
    }
    
    .allviewer-toolbar input[type="range"] {
        margin: 0 8px;
    }
    
    .allviewer-toolbar span {
        margin-right: 8px;
    }
    
    .allviewer-main-view {
        flex: 1;
        overflow: auto;
        padding: 8px;
    }
    
    .allviewer-status-bar {
        padding: 4px 8px;
        background-color: #f5f5f5;
        border-top: 1px solid #ddd;
        font-size: 0.8rem;
        color: #666;
    }
    
    /* グリッドビュー */
    .image-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
    }
    
    .image-thumbnail {
        position: relative;
        border: 2px solid transparent;
        cursor: pointer;
        border-radius: 4px;
        overflow: hidden;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }
    
    .image-thumbnail.selected {
        border-color: #4a90e2;
    }
    
    .thumbnail-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    
    .image-label {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        padding: 4px;
        background-color: rgba(0, 0, 0, 0.7);
        color: white;
        font-size: 0.8rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    
    /* リストビュー */
    .image-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    
    .image-list-item {
        display: flex;
        align-items: center;
        padding: 4px;
        border-radius: 4px;
        cursor: pointer;
    }
    
    .image-list-item:hover {
        background-color: #f0f0f0;
    }
    
    .image-list-item.selected {
        background-color: #e3f2fd;
    }
    
    .image-details {
        margin-left: 8px;
    }
    
    .image-name {
        font-weight: bold;
    }
    
    .image-info {
        font-size: 0.8rem;
        color: #666;
    }
    
    /* 詳細ビュー */
    .image-detail-view {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 16px;
    }
    
    .detail-image {
        max-width: 100%;
        max-height: 70vh;
        object-fit: contain;
        margin-bottom: 16px;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    }
    
    .image-metadata {
        width: 100%;
        max-width: 600px;
        margin-bottom: 16px;
    }
    
    .image-metadata table {
        width: 100%;
        border-collapse: collapse;
    }
    
    .image-metadata th {
        text-align: right;
        padding: 4px 8px;
        width: 30%;
        color: #666;
    }
    
    .image-metadata td {
        padding: 4px 8px;
    }
    
    .detail-navigation {
        display: flex;
        gap: 16px;
    }
    
    .detail-navigation button {
        padding: 8px 16px;
        background-color: #4a90e2;
        border: none;
        border-radius: 4px;
        color: white;
        cursor: pointer;
    }
    
    .detail-navigation button:hover {
        background-color: #3a7cc7;
    }
    
    .detail-navigation button:disabled {
        background-color: #cccccc;
        cursor: not-allowed;
    }
    
    .empty-message {
        display: flex;
        height: 100%;
        justify-content: center;
        align-items: center;
        font-size: 1.2rem;
        color: #666;
    }
    
    /* ダークモード対応 */
    @media (prefers-color-scheme: dark) {
        .allviewer-toolbar,
        .allviewer-status-bar {
            background-color: #333;
            border-color: #666;
        }
        
        .allviewer-toolbar button {
            background-color: #444;
            border-color: #666;
            color: #fff;
        }
        
        .image-list-item:hover {
            background-color: #444;
        }
        
        .image-list-item.selected {
            background-color: #2c3e50;
        }
        
        .image-info,
        .image-metadata th {
            color: #aaa;
        }
        
        .empty-message {
            color: #aaa;
        }
    }
    "#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frontend_code() {
        let code = get_frontend_code();
        assert!(!code.is_empty());
        assert!(code.contains("AllViewerUI"));
        assert!(code.contains("renderImages"));
    }
    
    #[test]
    fn test_css_styles() {
        let styles = get_css_styles();
        assert!(!styles.is_empty());
        assert!(styles.contains(".allviewer-container"));
        assert!(styles.contains("@media (prefers-color-scheme: dark)"));
    }
}
