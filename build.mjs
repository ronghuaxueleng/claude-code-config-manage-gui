#!/usr/bin/env node

/**
 * Claude Config Manager 构建脚本
 * 支持多平台构建和配置镜像源
 */

import { spawn, execSync } from 'child_process';
import { existsSync, mkdirSync, writeFileSync, readFileSync } from 'fs';
import { join } from 'path';
import { homedir, platform } from 'os';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// ANSI 颜色代码
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

// 日志辅助函数
const log = {
  info: (msg) => console.log(`${colors.blue}ℹ${colors.reset} ${msg}`),
  success: (msg) => console.log(`${colors.green}✓${colors.reset} ${msg}`),
  error: (msg) => console.log(`${colors.red}✗${colors.reset} ${msg}`),
  warning: (msg) => console.log(`${colors.yellow}⚠${colors.reset} ${msg}`),
  step: (step, total, msg) => console.log(`\n${colors.cyan}[${step}/${total}]${colors.reset} ${colors.bright}${msg}${colors.reset}`),
  header: (msg) => {
    console.log(`\n${colors.blue}${'='.repeat(50)}${colors.reset}`);
    console.log(`${colors.bright}${colors.blue}${msg.padStart((50 + msg.length) / 2)}${colors.reset}`);
    console.log(`${colors.blue}${'='.repeat(50)}${colors.reset}\n`);
  },
};

// 解析命令行参数
const args = process.argv.slice(2);
const options = {
  debug: args.includes('--debug'),
  target: args.find(arg => arg.startsWith('--target='))?.split('=')[1],
  useMirror: !args.includes('--no-mirror'),
  clean: args.includes('--clean'),
  help: args.includes('--help') || args.includes('-h'),
};

// 显示帮助信息
if (options.help) {
  console.log(`
${colors.bright}Claude Config Manager 构建脚本${colors.reset}

用法: node build.mjs [选项]

选项:
  --debug              构建 debug 版本
  --target=<target>    指定构建目标 (如: nsis, msi, deb, appimage)
  --no-mirror          不使用国内镜像源
  --clean              清理构建缓存后再构建
  -h, --help           显示帮助信息

示例:
  node build.mjs                    # 构建 release 版本
  node build.mjs --debug            # 构建 debug 版本
  node build.mjs --target=msi       # 只构建 MSI 安装包
  node build.mjs --clean            # 清理后构建
  `);
  process.exit(0);
}

// 执行命令的辅助函数
function runCommand(command, args = [], options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: 'inherit',
      shell: true,
      ...options,
    });

    child.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`命令执行失败，退出码: ${code}`));
      }
    });

    child.on('error', (err) => {
      reject(err);
    });
  });
}

// 检查命令是否存在
function commandExists(command) {
  try {
    execSync(platform() === 'win32' ? `where ${command}` : `which ${command}`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

// 检查构建环境
async function checkEnvironment() {
  log.step(1, 6, '检查构建环境...');

  // 检查 Node.js
  if (!commandExists('node')) {
    log.error('未找到 Node.js，请先安装 Node.js');
    log.info('下载地址: https://nodejs.org/');
    process.exit(1);
  }
  log.success(`Node.js 版本: ${process.version}`);

  // 检查 Rust/Cargo
  if (!commandExists('cargo')) {
    log.error('未找到 Rust/Cargo，请先安装 Rust');
    log.info('下载地址: https://www.rust-lang.org/zh-CN/tools/install');
    if (options.useMirror) {
      log.info('国内用户推荐使用: https://rsproxy.cn/');
    }
    process.exit(1);
  }

  try {
    const rustVersion = execSync('rustc --version', { encoding: 'utf-8' }).trim();
    log.success(`Rust 版本: ${rustVersion}`);
  } catch (err) {
    log.error('无法获取 Rust 版本信息');
  }

  // 检查 pnpm (可选)
  if (commandExists('pnpm')) {
    const pnpmVersion = execSync('pnpm --version', { encoding: 'utf-8' }).trim();
    log.info(`检测到 pnpm 版本: ${pnpmVersion}`);
  }
}

// 配置 Rust 镜像源
function configureRustMirror() {
  if (!options.useMirror) {
    log.info('跳过镜像源配置 (--no-mirror)');
    return;
  }

  log.step(2, 6, '配置 Rust 镜像源...');

  const cargoHome = process.env.CARGO_HOME || join(homedir(), '.cargo');
  const configPath = join(cargoHome, 'config.toml');

  if (existsSync(configPath)) {
    log.success('Rust 镜像源配置已存在');
    return;
  }

  mkdirSync(cargoHome, { recursive: true });

  const rustMirrorConfig = `[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
retry = 2
git-fetch-with-cli = true

[http]
timeout = 60
`;

  writeFileSync(configPath, rustMirrorConfig, 'utf-8');
  log.success('已配置 Rust 镜像源 (rsproxy.cn)');
}

// 配置 npm 镜像源
function configureNpmMirror() {
  if (!options.useMirror) {
    return;
  }

  log.step(3, 6, '配置 npm 镜像源...');

  try {
    const currentRegistry = execSync('npm config get registry', { encoding: 'utf-8' }).trim();

    if (currentRegistry.includes('npmmirror.com')) {
      log.success('npm 镜像源已配置');
      return;
    }

    execSync('npm config set registry https://registry.npmmirror.com/', { stdio: 'inherit' });
    log.success('已配置 npm 镜像源 (npmmirror.com)');
  } catch (err) {
    log.warning('配置 npm 镜像源时出现错误，将继续使用当前配置');
  }
}

// 安装依赖
async function installDependencies() {
  log.step(4, 6, '安装项目依赖...');

  if (existsSync(join(__dirname, 'node_modules')) && !options.clean) {
    log.success('依赖已存在，跳过安装');
    log.info('如需重新安装，请使用 --clean 选项');
    return;
  }

  try {
    log.info('正在安装 npm 依赖...');

    // 优先使用 pnpm，其次是 npm
    const packageManager = commandExists('pnpm') ? 'pnpm' : 'npm';
    log.info(`使用包管理器: ${packageManager}`);

    await runCommand(packageManager, ['install']);
    log.success('依赖安装完成');
  } catch (err) {
    log.error(`依赖安装失败: ${err.message}`);
    process.exit(1);
  }
}

// 清理构建缓存
async function cleanBuild() {
  if (!options.clean) {
    return;
  }

  log.info('清理构建缓存...');

  try {
    // 清理 Rust 构建缓存
    if (existsSync(join(__dirname, 'src-tauri', 'target'))) {
      log.info('清理 Rust 构建缓存...');
      await runCommand('cargo', ['clean'], { cwd: join(__dirname, 'src-tauri') });
    }

    // 清理 node_modules
    if (existsSync(join(__dirname, 'node_modules'))) {
      log.info('清理 node_modules...');
      const rimraf = await import('fs').then(fs => fs.promises.rm);
      await rimraf(join(__dirname, 'node_modules'), { recursive: true, force: true });
    }

    log.success('缓存清理完成');
  } catch (err) {
    log.warning(`清理缓存时出现错误: ${err.message}`);
  }
}

// 配置构建环境
function setupBuildEnvironment() {
  log.step(5, 6, '配置构建环境...');

  const env = { ...process.env };

  if (options.useMirror) {
    // Tauri Bundler 工具 GitHub 镜像（包括 NSIS、WiX 等）
    const githubMirror = 'https://gh-proxy.com/https://github.com';
    env.TAURI_BUNDLER_TOOLS_GITHUB_MIRROR = githubMirror;
    log.info(`Tauri Bundler 工具镜像: ${githubMirror}`);

    // WiX 工具镜像（用于 Windows MSI 打包）
    if (platform() === 'win32') {
      const wixMirror = 'https://gh-proxy.com/https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip';
      env.WIX_MIRROR = wixMirror;
      env.WIX3_DOWNLOAD_URL = wixMirror;
      env.TAURI_WIX3_DOWNLOAD_URL = wixMirror;
      env.TAURI_BUNDLE_WIX_DOWNLOAD_URL = wixMirror;
      env.WIX_DOWNLOAD_URL = wixMirror;
      log.info(`WiX 镜像: ${wixMirror}`);
    }
  }

  // Cargo 网络优化
  env.CARGO_HTTP_TIMEOUT = '120';
  env.CARGO_NET_RETRY = '3';
  env.CARGO_HTTP_MULTIPLEXING = 'false';

  log.success('构建环境配置完成');

  return env;
}

// 执行构建
async function build(env) {
  log.step(6, 6, '开始构建...');
  log.info('这可能需要几分钟时间，请耐心等待...\n');

  const buildArgs = ['run', 'tauri', 'build'];

  // 添加 debug 选项
  if (options.debug) {
    buildArgs.push('--', '--debug');
    log.info('构建模式: Debug');
  } else {
    log.info('构建模式: Release');
  }

  // 添加 target 选项
  if (options.target) {
    buildArgs.push('--', '--bundles', options.target);
    log.info(`构建目标: ${options.target}`);
  }

  try {
    await runCommand('npm', buildArgs, { env });
    return true;
  } catch (err) {
    log.error(`构建失败: ${err.message}`);
    return false;
  }
}

// 显示构建结果
function showBuildResult(success) {
  console.log('\n' + colors.blue + '='.repeat(50) + colors.reset);

  if (success) {
    console.log(colors.green + colors.bright + '✓ 构建成功！' + colors.reset);
    console.log(colors.blue + '='.repeat(50) + colors.reset + '\n');

    console.log(colors.bright + '📦 构建产物位置:' + colors.reset);

    const releaseDir = options.debug ? 'debug' : 'release';
    const bundleDir = join(__dirname, 'src-tauri', 'target', releaseDir, 'bundle');

    if (platform() === 'win32') {
      console.log(`  NSIS: ${colors.cyan}${join(bundleDir, 'nsis')}${colors.reset}`);
      console.log(`  MSI:  ${colors.cyan}${join(bundleDir, 'msi')}${colors.reset}`);
    } else if (platform() === 'darwin') {
      console.log(`  DMG:  ${colors.cyan}${join(bundleDir, 'dmg')}${colors.reset}`);
      console.log(`  App:  ${colors.cyan}${join(bundleDir, 'macos')}${colors.reset}`);
    } else {
      console.log(`  DEB:       ${colors.cyan}${join(bundleDir, 'deb')}${colors.reset}`);
      console.log(`  AppImage:  ${colors.cyan}${join(bundleDir, 'appimage')}${colors.reset}`);
    }

    console.log(`\n${colors.green}🎉 可以在以上目录找到安装程序${colors.reset}`);
  } else {
    console.log(colors.red + colors.bright + '✗ 构建失败！' + colors.reset);
    console.log(colors.blue + '='.repeat(50) + colors.reset + '\n');

    console.log(colors.bright + '🔧 故障排除建议:' + colors.reset);
    console.log('  1. 检查网络连接');
    console.log('  2. 清理缓存后重试: node build.mjs --clean');
    console.log('  3. 更新 Rust 工具链: rustup update');
    console.log('  4. 查看上面的错误信息获取详细原因\n');
  }
}

// 主函数
async function main() {
  log.header('Claude Config Manager 构建脚本');

  const startTime = Date.now();

  try {
    // 1. 检查环境
    await checkEnvironment();

    // 2. 配置 Rust 镜像
    configureRustMirror();

    // 3. 配置 npm 镜像
    configureNpmMirror();

    // 清理构建缓存（如果需要）
    await cleanBuild();

    // 4. 安装依赖
    await installDependencies();

    // 5. 配置构建环境
    const env = setupBuildEnvironment();

    // 6. 执行构建
    const success = await build(env);

    // 显示构建结果
    showBuildResult(success);

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);
    console.log(`\n${colors.cyan}⏱  总用时: ${elapsed}秒${colors.reset}\n`);

    process.exit(success ? 0 : 1);
  } catch (err) {
    log.error(`构建过程中发生错误: ${err.message}`);
    console.error(err);
    process.exit(1);
  }
}

// 运行主函数
main();
