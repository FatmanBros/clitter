#!/usr/bin/env node

/**
 * アイコン生成スクリプト
 *
 * 使用方法:
 *   node scripts/generate-icons.js
 *
 * 前提条件:
 *   npm install sharp
 */

const fs = require('fs');
const path = require('path');

async function generateIcons() {
  let sharp;
  try {
    sharp = require('sharp');
  } catch (e) {
    console.log('sharpがインストールされていません。インストールします...');
    const { execSync } = require('child_process');
    execSync('npm install sharp', { stdio: 'inherit' });
    sharp = require('sharp');
  }

  const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
  const svgPath = path.join(iconsDir, 'icon.svg');

  if (!fs.existsSync(svgPath)) {
    console.error('icon.svg が見つかりません:', svgPath);
    process.exit(1);
  }

  const sizes = [
    { name: '32x32.png', size: 32 },
    { name: '128x128.png', size: 128 },
    { name: '128x128@2x.png', size: 256 },
    { name: 'icon.png', size: 512 },
  ];

  console.log('アイコンを生成中...');

  for (const { name, size } of sizes) {
    const outputPath = path.join(iconsDir, name);
    await sharp(svgPath)
      .resize(size, size)
      .png()
      .toFile(outputPath);
    console.log(`  ✓ ${name} (${size}x${size})`);
  }

  // ICO for Windows (multi-size)
  const icoPath = path.join(iconsDir, 'icon.ico');
  const pngBuffer = await sharp(svgPath).resize(256, 256).png().toBuffer();

  // Simple ICO generation (single size, works for most cases)
  await sharp(pngBuffer).toFile(icoPath);
  console.log('  ✓ icon.ico (256x256)');

  console.log('\n✓ アイコン生成完了！');
  console.log('\n注意: icon.icns (macOS用) は macOS でのみ生成可能です。');
  console.log('macOS でビルドする場合は、以下を実行してください:');
  console.log('  npm run tauri icon src-tauri/icons/icon.svg');
}

generateIcons().catch(console.error);
