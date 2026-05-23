/**
 * 完整更新流程本地测试脚本
 * 构建真实安装包并启动本地更新服务器，可测试检查+下载完整流程
 *
 * 用法:
 * node scripts/test-update-full.js
 *
 * 前置条件:
 * - 已生成签名密钥对（tauri signer generate）
 * - tauri.key 文件在项目根目录
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';
import http from 'http';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.join(__dirname, '..');
const TAURI_DIR = path.join(ROOT, 'src-tauri');

const TEST_VERSION = '0.99.0';
const PORT = 3000;

function backupAndSetVersion() {
  const confPath = path.join(TAURI_DIR, 'tauri.conf.json');
  const cargoPath = path.join(TAURI_DIR, 'Cargo.toml');
  const pkgPath = path.join(ROOT, 'package.json');

  // Backup files
  fs.copyFileSync(confPath, confPath + '.local_test_bak');
  fs.copyFileSync(cargoPath, cargoPath + '.local_test_bak');
  fs.copyFileSync(pkgPath, pkgPath + '.local_test_bak');

  // Update version in tauri.conf.json
  const conf = JSON.parse(fs.readFileSync(confPath, 'utf8'));
  conf.version = TEST_VERSION;
  fs.writeFileSync(confPath, JSON.stringify(conf, null, 2));

  // Update version in Cargo.toml
  let cargo = fs.readFileSync(cargoPath, 'utf8');
  cargo = cargo.replace(/^version = ".*"/m, `version = "${TEST_VERSION}"`);
  fs.writeFileSync(cargoPath, cargo);

  // Update version in package.json
  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  pkg.version = TEST_VERSION;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');

  console.log(`✅ 版本号已临时修改为 ${TEST_VERSION}`);
}

function restoreVersion() {
  const confPath = path.join(TAURI_DIR, 'tauri.conf.json');
  const cargoPath = path.join(TAURI_DIR, 'Cargo.toml');
  const pkgPath = path.join(ROOT, 'package.json');

  for (const f of [confPath, cargoPath, pkgPath]) {
    const bak = f + '.local_test_bak';
    if (fs.existsSync(bak)) {
      fs.copyFileSync(bak, f);
      fs.unlinkSync(bak);
    }
  }
  console.log('✅ 版本号已恢复');
}

function buildInstaller() {
  console.log('\n🔨 开始构建安装包（这可能需要几分钟）...\n');
  try {
    execSync('npm run tauri:build', { cwd: ROOT, stdio: 'inherit' });
    console.log('\n✅ 构建完成');
  } catch (e) {
    restoreVersion();
    console.error('❌ 构建失败');
    process.exit(1);
  }
}

function findInstaller() {
  const nsisDir = path.join(TAURI_DIR, 'target/release/bundle/nsis');
  if (!fs.existsSync(nsisDir)) {
    return null;
  }
  const files = fs.readdirSync(nsisDir);
  const exe = files.find(f => f.endsWith('.exe') && f.includes('setup'));
  return exe ? path.join(nsisDir, exe) : null;
}

function signInstaller(installerPath) {
  const keyPath = path.join(ROOT, 'tauri.key');
  if (!fs.existsSync(keyPath)) {
    console.error('❌ 未找到签名私钥 tauri.key');
    console.log('💡 请先运行: tauri signer generate');
    return null;
  }

  console.log('\n🔏 签名安装包...');
  try {
    const result = execSync(
      `npx tauri signer sign --private-key "${keyPath}" --password "" "${installerPath}"`,
      { cwd: ROOT, encoding: 'utf8', stdio: 'pipe' }
    );
    // Parse signature from output
    const match = result.match(/Signature:?\s*(.+)/i);
    if (match) {
      console.log('✅ 签名成功');
      return match[1].trim();
    }
    // Try to read result directly
    console.log('✅ 签名结果:', result);
    return result.trim();
  } catch (e) {
    console.error('❌ 签名失败:', e.message);
    return null;
  }
}

function startServer(installerPath, signature) {
  const serverDir = path.join(ROOT, 'test-updates');
  if (!fs.existsSync(serverDir)) {
    fs.mkdirSync(serverDir, { recursive: true });
  }

  const installerName = path.basename(installerPath);
  const targetPath = path.join(serverDir, installerName);
  fs.copyFileSync(installerPath, targetPath);

  const latestJson = {
    version: TEST_VERSION,
    notes: "本地测试更新 - 完整流程\n\n这是通过本地构建的真实安装包",
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature: signature || "",
        url: `http://localhost:${PORT}/${installerName}`
      }
    }
  };
  fs.writeFileSync(path.join(serverDir, 'latest.json'), JSON.stringify(latestJson, null, 2));

  const server = http.createServer((req, res) => {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    if (req.method === 'OPTIONS') {
      res.writeHead(200);
      res.end();
      return;
    }

    const url = req.url === '/' ? '/latest.json' : req.url;

    if (url === '/latest.json') {
      const filePath = path.join(serverDir, 'latest.json');
      res.setHeader('Content-Type', 'application/json');
      res.end(fs.readFileSync(filePath));
    } else {
      const filePath = path.join(serverDir, url.slice(1));
      if (fs.existsSync(filePath)) {
        const stat = fs.statSync(filePath);
        res.setHeader('Content-Type', 'application/octet-stream');
        res.setHeader('Content-Length', stat.size);
        res.writeHead(200);
        fs.createReadStream(filePath).pipe(res);
      } else {
        res.writeHead(404);
        res.end('Not found');
      }
    }
  });

  server.listen(PORT, () => {
    console.log('\n' + '='.repeat(60));
    console.log('🚀 本地更新服务器已启动');
    console.log(`📡 地址: http://localhost:${PORT}`);
    console.log(`📦 安装包: ${installerName}`);
    console.log('='.repeat(60));
    console.log('\n📋 测试步骤:');
    console.log('1. 启动旧版本的应用 (0.1.0)');
    console.log('2. 进入"关于"页面 → 点击"检查更新"');
    console.log('3. 应该检测到 0.99.0 → 点击"立即更新"');
    console.log('4. 观察下载进度 → 下载完成后自动安装');
    console.log('\n按 Ctrl+C 停止服务器（版本号会自动恢复）\n');
  });
}

function cleanup() {
  restoreVersion();
}

process.on('SIGINT', () => {
  cleanup();
  process.exit(0);
});

// 主流程
function main() {
  console.log('🔧 Tauri 完整更新流程本地测试\n');

  // Step 1: 临时修改版本号
  backupAndSetVersion();

  // Step 2: 构建安装包
  buildInstaller();

  // Step 3: 找到安装包
  const installerPath = findInstaller();
  if (!installerPath) {
    console.error('❌ 找不到构建好的安装包');
    restoreVersion();
    process.exit(1);
  }
  console.log(`📦 安装包: ${installerPath}`);

  // Step 4: 签名
  const signature = signInstaller(installerPath);

  // Step 5: 恢复版本号
  restoreVersion();

  // Step 6: 启动服务器
  startServer(installerPath, signature);
}

main();
