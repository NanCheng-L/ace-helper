/**
 * 本地更新测试服务器
 * 用于在本地测试 Tauri 更新检测功能，无需发布到 GitHub
 * 
 * 用法:
 * 1. node scripts/local-update-server.js
 * 2. 修改 tauri.conf.json 中的 endpoints 为: ["http://localhost:3000/latest.json"]
 * 3. 运行应用测试更新检测
 */

import http from 'http';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PORT = 3000;
const UPDATE_DIR = path.join(__dirname, '../test-updates');

// 确保测试更新目录存在
if (!fs.existsSync(UPDATE_DIR)) {
  fs.mkdirSync(UPDATE_DIR, { recursive: true });
}

// 创建测试用的 latest.json
function createTestLatestJson() {
  const latestJson = {
    version: "0.99.0",
    notes: "这是一个本地测试更新\n- 测试更新检测功能\n- 无需发布到 GitHub",
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        "signature": "dW1teS1zaWduYXR1cmUtZm9yLXRlc3Rpbmc=",
        "url": `http://localhost:${PORT}/test-update.exe`
      }
    }
  };
  
  fs.writeFileSync(path.join(UPDATE_DIR, 'latest.json'), JSON.stringify(latestJson, null, 2));
  console.log('✅ 已创建测试用的 latest.json');
  return latestJson;
}

// 创建虚拟的更新文件（用于测试下载）
function createDummyUpdateFile() {
  const dummyContent = Buffer.alloc(1024 * 1024); // 1MB 虚拟文件
  fs.writeFileSync(path.join(UPDATE_DIR, 'test-update.exe'), dummyContent);
  console.log('✅ 已创建虚拟更新文件 (1MB)');
}

// 启动服务器
const server = http.createServer((req, res) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  
  // 设置 CORS 头，允许 Tauri 应用访问
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  
  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }
  
  const url = req.url;
  
  if (url === '/latest.json') {
    // 返回 latest.json
    const filePath = path.join(UPDATE_DIR, 'latest.json');
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath);
      res.setHeader('Content-Type', 'application/json');
      res.writeHead(200);
      res.end(content);
    } else {
      res.writeHead(404);
      res.end(JSON.stringify({ error: 'latest.json not found' }));
    }
  } else if (url === '/test-update.exe') {
    // 返回虚拟更新文件
    const filePath = path.join(UPDATE_DIR, 'test-update.exe');
    if (fs.existsSync(filePath)) {
      const stat = fs.statSync(filePath);
      res.setHeader('Content-Type', 'application/octet-stream');
      res.setHeader('Content-Length', stat.size);
      res.setHeader('Content-Disposition', 'attachment; filename="test-update.exe"');
      res.writeHead(200);
      const stream = fs.createReadStream(filePath);
      stream.pipe(res);
    } else {
      res.writeHead(404);
      res.end('File not found');
    }
  } else {
    res.writeHead(404);
    res.end('Not found');
  }
});

server.listen(PORT, () => {
  console.log('\n🚀 本地更新测试服务器已启动');
  console.log(`📡 地址: http://localhost:${PORT}`);
  console.log('\n📋 测试步骤:');
  console.log('1. 确认 tauri.conf.json 中 endpoints 指向本地服务器');
  console.log('2. 运行应用: npm run tauri:dev');
  console.log('3. 进入"关于"页面，点击"检查更新"');
  console.log('4. 应该检测到版本 0.99.0');
  console.log('\n⚠️  本服务器仅用于测试"检查更新"功能');
  console.log('⚠️  "下载更新"会因签名验证失败（虚拟下载文件无有效签名）');
  console.log('💡 如需测试完整流程（含下载），请运行: node scripts/test-update-full.js');
  console.log('\n按 Ctrl+C 停止服务器\n');
  
  // 创建测试文件
  createTestLatestJson();
  createDummyUpdateFile();
});

// 优雅退出
process.on('SIGINT', () => {
  console.log('\n\n👋 正在关闭服务器...');
  server.close(() => {
    console.log('✅ 服务器已关闭');
    process.exit(0);
  });
});
