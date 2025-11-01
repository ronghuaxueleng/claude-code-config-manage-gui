#!/usr/bin/env node

/**
 * Claude Config CLI 构建脚本
 * 纯 Rust 项目构建工具
 */

import { spawn, execSync } from 'child_process';
import { existsSync, mkdirSync, writeFileSync, readFileSync, copyFileSync, readdirSync, statSync } from 'fs';
import { join, basename } from 'path';
import { homedir, platform, arch } from 'os';
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
  magenta: '\x1b[35m',
};

// 日志辅助函数
const log = {
  info: (msg) => console.log(`${colors.cyan}ℹ${colors.reset} ${msg}`),
  success: (msg) => console.log(`${colors.green}✓${colors.reset} ${msg}`),
  error: (msg) => console.log(`${colors.red}✗${colors.reset} ${msg}`),
  warning: (msg) => console.log(`${colors.yellow}⚠${colors.reset} ${msg}`),
  step: (step, total, msg) => console.log(`\n${colors.blue}[${step}/${total}]${colors.reset} ${colors.bright}${msg}${colors.reset}`),
  header: (msg) => {
    console.log(`\n${colors.cyan}${'='.repeat(60)}${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}${msg.padStart((60 + msg.length) / 2)}${colors.reset}`);
    console.log(`${colors.cyan}${'='.repeat(60)}${colors.reset}\n`);
  },
};

// 解析命令行参数
const args = process.argv.slice(2);
const options = {
  release: !args.includes('--debug'),
  debug: args.includes('--debug'),
  target: args.find(arg => arg.startsWith('--target='))?.split('=')[1],
  useMirror: !args.includes('--no-mirror'),
  clean: args.includes('--clean'),
  install: args.includes('--install'),
  strip: args.includes('--strip'),
  help: args.includes('--help') || args.includes('-h'),
};

// 显示帮助信息
if (options.help) {
  console.log(`
${colors.bright}Claude Config CLI 构建脚本${colors.reset}

用法: node build.mjs [选项]

选项:
  --debug              构建 debug 版本（默认构建 release）
  --target=<triple>    指定目标三元组（如: x86_64-pc-windows-gnu）
  --no-mirror          不使用国内镜像源
  --clean              清理构建缓存后再构建
  --install            构建完成后安装到系统
  --strip              剥离二进制文件的调试符号（减小体积）
  -h, --help           显示帮助信息

示例:
  node build.mjs                        # 构建 release 版本
  node build.mjs --debug                # 构建 debug 版本
  node build.mjs --clean                # 清理后构建
  node build.mjs --install              # 构建并安装
  node build.mjs --strip                # 构建并剥离调试符号
  node build.mjs --target=x86_64-unknown-linux-musl  # 交叉编译
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
  log.step(1, 8, '检查构建环境...');

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
    const cargoVersion = execSync('cargo --version', { encoding: 'utf-8' }).trim();
    log.success(`${rustVersion}`);
    log.success(`${cargoVersion}`);
  } catch (err) {
    log.error('无法获取 Rust 版本信息');
  }

  // 检查目标三元组
  if (options.target) {
    log.info(`构建目标: ${options.target}`);

    try {
      const installedTargets = execSync('rustup target list --installed', { encoding: 'utf-8' });
      if (!installedTargets.includes(options.target)) {
        log.warning(`目标 ${options.target} 未安装，正在安装...`);
        execSync(`rustup target add ${options.target}`, { stdio: 'inherit' });
        log.success(`目标 ${options.target} 安装完成`);
      }
    } catch (err) {
      log.warning(`无法检查目标是否已安装: ${err.message}`);
    }
  }

  log.info(`构建模式: ${options.release ? 'Release' : 'Debug'}`);
  log.info(`平台: ${platform()} (${arch()})`);
}

// 配置 Rust 镜像源
function configureRustMirror() {
  if (!options.useMirror) {
    log.info('跳过镜像源配置 (--no-mirror)');
    return;
  }

  log.step(2, 8, '配置 Rust 镜像源...');

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

// 清理构建缓存
async function cleanBuild() {
  if (!options.clean) {
    return;
  }

  log.step(3, 8, '清理构建缓存...');

  try {
    await runCommand('cargo', ['clean'], { cwd: __dirname });
    log.success('构建缓存清理完成');
  } catch (err) {
    log.warning(`清理缓存失败: ${err.message}`);
  }
}

// 同步版本号：从 Cargo.toml 读取版本号并更新到 i18n.rs
function syncVersionNumber() {
  log.step(4, 8, '同步版本号...');

  try {
    // 读取 Cargo.toml 获取版本号
    const cargoTomlPath = join(__dirname, 'Cargo.toml');
    const cargoToml = readFileSync(cargoTomlPath, 'utf-8');
    const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);

    if (!versionMatch) {
      log.warning('无法从 Cargo.toml 读取版本号，跳过同步');
      return;
    }

    const version = `v${versionMatch[1]}`;
    log.info(`检测到版本号: ${version}`);

    // 更新 i18n.rs 中的版本号
    const i18nPath = join(__dirname, 'src', 'i18n.rs');
    let i18nContent = readFileSync(i18nPath, 'utf-8');

    // 替换中文版本号
    const zhCnReplaced = i18nContent.replace(
      /zh_cn\.insert\("app\.version",\s*"v[^"]+"\)/,
      `zh_cn.insert("app.version", "${version}")`
    );

    // 替换英文版本号
    const enUsReplaced = zhCnReplaced.replace(
      /en_us\.insert\("app\.version",\s*"v[^"]+"\)/,
      `en_us.insert("app.version", "${version}")`
    );

    if (enUsReplaced !== i18nContent) {
      writeFileSync(i18nPath, enUsReplaced, 'utf-8');
      log.success(`版本号已同步到 i18n.rs: ${version}`);
    } else {
      log.success('i18n.rs 版本号已是最新');
    }
  } catch (err) {
    log.warning(`版本号同步失败: ${err.message}`);
    log.warning('将继续构建，但版本号可能不一致');
  }
}

// 检查并更新依赖
async function updateDependencies() {
  log.step(5, 8, '检查依赖...');

  if (existsSync(join(__dirname, 'Cargo.lock'))) {
    log.success('依赖已存在');
  } else {
    log.info('首次构建，将自动下载依赖...');
  }
}

// 配置构建环境
function setupBuildEnvironment() {
  log.step(6, 8, '配置构建环境...');

  const env = { ...process.env };

  // Cargo 网络优化
  env.CARGO_HTTP_TIMEOUT = '120';
  env.CARGO_NET_RETRY = '3';
  env.CARGO_HTTP_MULTIPLEXING = 'false';

  // 如果是 release 构建，启用优化
  if (options.release) {
    env.CARGO_PROFILE_RELEASE_LTO = 'true';
    env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '1';
    log.info('已启用 LTO 优化');
  }

  log.success('构建环境配置完成');

  return env;
}

// 执行构建
async function build(env) {
  log.step(7, 8, '开始构建...');
  log.info('这可能需要几分钟时间，请耐心等待...\n');

  const buildArgs = ['build'];

  // 添加 release 或 debug
  if (options.release) {
    buildArgs.push('--release');
  }

  // 添加 target
  if (options.target) {
    buildArgs.push('--target', options.target);
  }

  try {
    await runCommand('cargo', buildArgs, { env, cwd: __dirname });
    log.success('构建完成！');
    return true;
  } catch (err) {
    log.error(`构建失败: ${err.message}`);
    return false;
  }
}

// 剥离调试符号
async function stripBinary() {
  if (!options.strip || !options.release) {
    return;
  }

  log.info('正在剥离调试符号...');

  const binaryPath = getBinaryPath();
  if (!existsSync(binaryPath)) {
    log.warning('找不到二进制文件，跳过剥离');
    return;
  }

  try {
    if (commandExists('strip')) {
      await runCommand('strip', [binaryPath]);
      log.success('调试符号已剥离');
    } else {
      log.warning('未找到 strip 命令，跳过剥离');
    }
  } catch (err) {
    log.warning(`剥离失败: ${err.message}`);
  }
}

// 获取二进制文件路径
function getBinaryPath() {
  const buildMode = options.release ? 'release' : 'debug';
  const targetDir = options.target
    ? join(__dirname, 'target', options.target, buildMode)
    : join(__dirname, 'target', buildMode);

  const binaryName = platform() === 'win32' ? 'claude-config.exe' : 'claude-config';
  return join(targetDir, binaryName);
}

// 安装到系统
async function installToSystem() {
  if (!options.install) {
    return;
  }

  log.step(8, 8, '安装到系统...');

  const binaryPath = getBinaryPath();

  if (!existsSync(binaryPath)) {
    log.error('找不到构建的二进制文件');
    return false;
  }

  try {
    if (platform() === 'win32') {
      // Windows: 复制到 %USERPROFILE%\.cargo\bin
      const installDir = join(homedir(), '.cargo', 'bin');
      mkdirSync(installDir, { recursive: true });
      const destPath = join(installDir, 'claude-config.exe');
      copyFileSync(binaryPath, destPath);
      log.success(`已安装到: ${destPath}`);
      log.info('请确保 %USERPROFILE%\\.cargo\\bin 在 PATH 环境变量中');
    } else {
      // Linux/macOS: 使用 cargo install
      await runCommand('cargo', ['install', '--path', '.'], { cwd: __dirname });
      log.success('已安装到: ~/.cargo/bin/claude-config');
      log.info('请确保 ~/.cargo/bin 在 PATH 环境变量中');
    }

    return true;
  } catch (err) {
    log.error(`安装失败: ${err.message}`);
    return false;
  }
}

// 显示构建结果
function showBuildResult(success) {
  console.log('\n' + colors.cyan + '='.repeat(60) + colors.reset);

  if (success) {
    console.log(colors.green + colors.bright + '✓ 构建成功！' + colors.reset);
    console.log(colors.cyan + '='.repeat(60) + colors.reset + '\n');

    const binaryPath = getBinaryPath();

    console.log(colors.bright + '📦 构建产物:' + colors.reset);
    console.log(`  ${colors.cyan}${binaryPath}${colors.reset}`);

    // 显示文件大小
    if (existsSync(binaryPath)) {
      const stats = statSync(binaryPath);
      const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
      console.log(`  ${colors.magenta}大小: ${sizeMB} MB${colors.reset}`);
    }

    console.log(`\n${colors.bright}🚀 运行方式:${colors.reset}`);
    console.log(`  ${colors.cyan}${binaryPath}${colors.reset}`);

    if (options.install) {
      console.log(`\n${colors.bright}或直接运行:${colors.reset}`);
      console.log(`  ${colors.cyan}claude-config${colors.reset}`);
    } else {
      console.log(`\n${colors.bright}安装到系统:${colors.reset}`);
      console.log(`  ${colors.cyan}node build.mjs --install${colors.reset}`);
    }
  } else {
    console.log(colors.red + colors.bright + '✗ 构建失败！' + colors.reset);
    console.log(colors.cyan + '='.repeat(60) + colors.reset + '\n');

    console.log(colors.bright + '🔧 故障排除建议:' + colors.reset);
    console.log('  1. 检查网络连接');
    console.log('  2. 清理缓存后重试: node build.mjs --clean');
    console.log('  3. 更新 Rust 工具链: rustup update');
    console.log('  4. 查看上面的错误信息获取详细原因\n');
  }
}

// 主函数
async function main() {
  log.header('Claude Config CLI 构建脚本');

  const startTime = Date.now();

  try {
    // 1. 检查环境
    await checkEnvironment();

    // 2. 配置 Rust 镜像
    configureRustMirror();

    // 3. 清理构建缓存（如果需要）
    await cleanBuild();

    // 4. 同步版本号
    syncVersionNumber();

    // 5. 检查依赖
    await updateDependencies();

    // 6. 配置构建环境
    const env = setupBuildEnvironment();

    // 7. 执行构建
    const success = await build(env);

    if (!success) {
      showBuildResult(false);
      process.exit(1);
    }

    // 剥离调试符号（可选）
    await stripBinary();

    // 7. 安装到系统（可选）
    await installToSystem();

    // 显示构建结果
    showBuildResult(true);

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);
    console.log(`\n${colors.cyan}⏱  总用时: ${elapsed}秒${colors.reset}\n`);

    process.exit(0);
  } catch (err) {
    log.error(`构建过程中发生错误: ${err.message}`);
    console.error(err);
    process.exit(1);
  }
}

// 运行主函数
main();
