// findme/ui.rs - FindMeプラグインのUI関連コード

/// フロントエンドコードを生成
/// この関数は、Rustバックエンドからフロントエンドに渡される
/// UI関連のJavaScriptコードを返します。
pub fn get_frontend_code() -> String {
    // Hello FindMeメッセージを表示するシンプルなコード
    let code = r#"
    // FindMeプラグインのUIコンポーネント初期化関数
    function initializeFindMeUI(container) {
        // コンテナの初期化
        container.innerHTML = '';
        
        // タイトル要素の作成
        const title = document.createElement('h1');
        title.textContent = 'Hello FindMe';
        title.style.textAlign = 'center';
        title.style.color = '#4a90e2';
        title.style.marginTop = '50px';
        
        // サブタイトル要素の作成
        const subtitle = document.createElement('p');
        subtitle.textContent = 'Welcome to the FindMe Image Game!';
        subtitle.style.textAlign = 'center';
        subtitle.style.color = '#666';
        
        // コンテナに要素を追加
        container.appendChild(title);
        container.appendChild(subtitle);
        
        // スタイルの追加
        container.style.fontFamily = 'Arial, sans-serif';
        container.style.backgroundColor = '#f4f4f4';
        container.style.height = '100%';
        container.style.display = 'flex';
        container.style.flexDirection = 'column';
        container.style.justifyContent = 'center';
        container.style.alignItems = 'center';
    }

    export default {
        initialize: function(container) {
            initializeFindMeUI(container);
        }
    };
    "#;
    
    code.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frontend_code() {
        let code = get_frontend_code();
        assert!(!code.is_empty());
        assert!(code.contains("Hello FindMe"));
    }
}
