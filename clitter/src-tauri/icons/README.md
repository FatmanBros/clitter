# アイコンファイルの生成

Tauriアプリのビルドには以下のアイコンファイルが必要です。

## 必要なファイル

- `32x32.png` - 32x32ピクセル
- `128x128.png` - 128x128ピクセル
- `128x128@2x.png` - 256x256ピクセル（Retina用）
- `icon.icns` - macOS用
- `icon.ico` - Windows用
- `icon.png` - Linux/トレイ用

## 生成方法

### 方法1: Tauri CLI を使用（推奨）

`icon.svg` または `icon.png`（1024x1024推奨）を用意して：

```bash
npm run tauri icon src-tauri/icons/icon.svg
```

### 方法2: オンラインツール

1. https://icon.kitchen/ でアイコンを作成
2. 各サイズをダウンロード
3. このフォルダに配置

### 方法3: ImageMagick を使用

```bash
# PNG各サイズ
convert icon.svg -resize 32x32 32x32.png
convert icon.svg -resize 128x128 128x128.png
convert icon.svg -resize 256x256 128x128@2x.png

# ICO (Windows)
convert icon.svg -resize 256x256 icon.ico

# ICNS (macOS)
# macOSでのみ生成可能
iconutil -c icns icon.iconset
```

## 注意

- `icon.svg` はサンプルとして含まれています
- ビルド前に必ず上記のファイルを生成してください
