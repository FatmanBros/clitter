# Windows ビルド手順

## 前提条件

### 1. Rust のインストール
https://rustup.rs/ から rustup-init.exe をダウンロードして実行

```powershell
# または winget を使用
winget install Rustlang.Rustup
```

### 2. Node.js のインストール
https://nodejs.org/ から LTS版をダウンロード

```powershell
# または winget を使用
winget install OpenJS.NodeJS.LTS
```

### 3. Visual Studio Build Tools のインストール
https://visualstudio.microsoft.com/visual-cpp-build-tools/

「C++ によるデスクトップ開発」ワークロードを選択してインストール

## ビルド手順

### 開発モード（デバッグ）
```powershell
cd clitter
npm install
npm run tauri dev
```

### リリースビルド（本番用 .exe）
```powershell
cd clitter
npm install
npm run tauri build
```

## 出力ファイル

ビルド完了後、以下の場所にファイルが生成されます：

```
src-tauri/target/release/
├── clitter.exe                      # 単体実行ファイル

src-tauri/target/release/bundle/
├── msi/
│   └── Clitter_0.1.0_x64_en-US.msi  # MSI インストーラー
└── nsis/
    └── Clitter_0.1.0_x64-setup.exe  # NSIS インストーラー（推奨）
```

## インストール

### 方法A: インストーラーを使用（推奨）
`Clitter_0.1.0_x64-setup.exe` を実行

### 方法B: ポータブル版
`clitter.exe` を任意のフォルダにコピーして直接実行

## 使い方

1. アプリを起動（タスクトレイに常駐）
2. `Alt + V` でウィンドウを表示
3. 数字キー `1`〜`5` で直近の履歴をコピー
4. 矢印キーでカテゴリ切替
   - `←` 画像
   - `↓` 文字
   - `→` 数字
5. `↑` でホワイトボード表示
6. ホワイトボードでショートカット入力 → `Enter` でコピー

## トラブルシューティング

### Rust のコンパイルエラー
```powershell
rustup update
```

### WebView2 関連のエラー
Windows 10/11 には通常プリインストールされていますが、
ない場合は https://developer.microsoft.com/en-us/microsoft-edge/webview2/ からインストール
