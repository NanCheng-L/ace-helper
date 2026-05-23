/**
 * 生成 Tauri 更新所需的 latest.json 文件
 * 用法: node scripts/generate-update-json.js <版本号> <安装包路径>
 * 示例: node scripts/generate-update-json.js 0.2.0 ./src-tauri/target/release/bundle/nsis/ACE小助手_0.2.0_x64-setup.exe
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// 从 Cargo.toml 读取公钥
function getPublicKey() {
  const cargoToml = fs.readFileSync(path.join(__dirname, '../src-tauri/Cargo.toml'), 'utf8');
  // 公钥在 tauri.conf.json 中
  const tauriConf = JSON.parse(fs.readFileSync(path.join(__dirname, '../src-tauri/tauri.conf.json'), 'utf8'));
  return tauriConf.plugins.updater.pubkey;
}

// 生成签名
function generateSignature(filePath, privateKey) {
  const fileBuffer = fs.readFileSync(filePath);
  const signature = crypto.createSign('SHA256').update(fileBuffer).sign(privateKey, 'base64');
  return signature;
}

// 主函数
function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.log('用法: node scripts/generate-update-json.js <版本号> <安装包路径>');
    console.log('示例: node scripts/generate-update-json.js 0.2.0 ./src-tauri/target/release/bundle/nsis/ACE小助手_0.2.0_x64-setup.exe');
    process.exit(1);
  }
  
  const version = args[0];
  const installerPath = args[1];
  
  if (!fs.existsSync(installerPath)) {
    console.error('错误: 安装包不存在:', installerPath);
    process.exit(1);
  }
  
  // 获取文件大小
  const stats = fs.statSync(installerPath);
  
  // 生成 latest.json 结构
  const updateJson = {
    version: version,
    notes: "新版本更新",
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature: "需要私钥签名后填入",
        url: `https://github.com/NanCheng-L/ace-helper/releases/download/v${version}/ACE小助手_${version}_x64-setup.exe`
      }
    }
  };
  
  // 输出到文件
  const outputPath = path.join(__dirname, `../latest.json`);
  fs.writeFileSync(outputPath, JSON.stringify(updateJson, null, 2));
  
  console.log('✅ latest.json 已生成:', outputPath);
  console.log('\n文件信息:');
  console.log('  版本:', version);
  console.log('  文件大小:', (stats.size / 1024 / 1024).toFixed(2), 'MB');
  console.log('  文件路径:', installerPath);
  console.log('\n⚠️  注意: 你需要使用私钥对安装包签名，并将签名填入 signature 字段');
  console.log('公钥:', getPublicKey().substring(0, 50) + '...');
}

main();
