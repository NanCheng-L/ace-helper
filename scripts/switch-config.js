/**
 * 切换 Tauri 配置文件（生产环境/开发测试环境）
 *
 * 用法:
 * node scripts/switch-config.js dev    # 切换到开发测试配置（本地更新服务器）
 * node scripts/switch-config.js prod   # 切换到生产配置（GitHub）
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const CONFIG_DIR = path.join(__dirname, '../src-tauri');
const PROD_CONFIG = 'tauri.conf.json';
const DEV_CONFIG = 'tauri.conf.dev.json';
const BACKUP_CONFIG = 'tauri.conf.json.backup';

function switchToDev() {
  console.log('🔧 切换到开发测试配置...');

  // 备份生产配置
  if (fs.existsSync(path.join(CONFIG_DIR, PROD_CONFIG))) {
    fs.copyFileSync(
      path.join(CONFIG_DIR, PROD_CONFIG),
      path.join(CONFIG_DIR, BACKUP_CONFIG)
    );
    console.log('✅ 已备份生产配置');
  }

  // 复制开发配置
  if (fs.existsSync(path.join(CONFIG_DIR, DEV_CONFIG))) {
    fs.copyFileSync(
      path.join(CONFIG_DIR, DEV_CONFIG),
      path.join(CONFIG_DIR, PROD_CONFIG)
    );
    console.log('✅ 已切换到开发测试配置');
    console.log('\n📡 更新端点: http://localhost:3000/latest.json');
    console.log('💡 请先运行: node scripts/local-update-server.js');
  } else {
    console.error('❌ 开发配置文件不存在:', DEV_CONFIG);
    process.exit(1);
  }
}

function switchToProd() {
  console.log('🚀 切换到生产配置...');

  // 从备份恢复
  if (fs.existsSync(path.join(CONFIG_DIR, BACKUP_CONFIG))) {
    fs.copyFileSync(
      path.join(CONFIG_DIR, BACKUP_CONFIG),
      path.join(CONFIG_DIR, PROD_CONFIG)
    );
    fs.unlinkSync(path.join(CONFIG_DIR, BACKUP_CONFIG));
    console.log('✅ 已恢复生产配置');
    console.log('\n📡 更新端点: GitHub Releases');
  } else {
    console.error('❌ 备份文件不存在，无法恢复生产配置');
    console.log('💡 请手动检查 tauri.conf.json 文件');
    process.exit(1);
  }
}

function showCurrent() {
  const configPath = path.join(CONFIG_DIR, PROD_CONFIG);
  if (fs.existsSync(configPath)) {
    const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
    const endpoint = config.plugins?.updater?.endpoints?.[0] || '未配置';
    console.log('📋 当前配置:');
    console.log('   更新端点:', endpoint);
    console.log('   模式:', endpoint.includes('localhost') ? '开发测试' : '生产环境');
  }
}

// 主函数
function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command || command === 'status') {
    showCurrent();
  } else if (command === 'dev') {
    switchToDev();
  } else if (command === 'prod') {
    switchToProd();
  } else {
    console.log('用法: node scripts/switch-config.js [dev|prod|status]');
    console.log('  dev    - 切换到开发测试配置（本地更新服务器）');
    console.log('  prod   - 切换到生产配置（GitHub）');
    console.log('  status - 显示当前配置');
    process.exit(1);
  }
}

main();
