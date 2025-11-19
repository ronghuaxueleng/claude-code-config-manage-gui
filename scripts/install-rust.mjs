#!/usr/bin/env node

/**
 * Rust 安装脚本 - 自动配置国内镜像
 * 支持 Windows / Linux / macOS
 */

import { execSync, spawn } from 'child_process'
import { existsSync, mkdirSync, writeFileSync, appendFileSync, readFileSync, statSync } from 'fs'
import { homedir, platform } from 'os'
import { join } from 'path'
import { createInterface } from 'readline'

// 国内镜像源配置（按推荐顺序排列）
const MIRRORS = {
  rsproxy: {
    name: '字节跳动 RsProxy（推荐）',
    rustup_dist: 'https://rsproxy.cn',
    rustup_update: 'https://rsproxy.cn/rustup',
    cargo_registry: 'sparse+https://rsproxy.cn/index/',
  },
  ustc: {
    name: '中国科学技术大学',
    rustup_dist: 'https://mirrors.ustc.edu.cn/rust-static',
    rustup_update: 'https://mirrors.ustc.edu.cn/rust-static/rustup',
    cargo_registry: 'sparse+https://mirrors.ustc.edu.cn/crates.io-index/',
  },
  tuna: {
    name: '清华大学 TUNA',
    rustup_dist: 'https://mirrors.tuna.tsinghua.edu.cn/rustup',
    rustup_update: 'https://mirrors.tuna.tsinghua.edu.cn/rustup',
    cargo_registry: 'sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/',
  },
  sjtu: {
    name: '上海交通大学',
    rustup_dist: 'https://mirrors.sjtug.sjtu.edu.cn/rust-static',
    rustup_update: 'https://mirrors.sjtug.sjtu.edu.cn/rust-static/rustup',
    cargo_registry: 'sparse+https://mirrors.sjtug.sjtu.edu.cn/crates.io-index/',
  },
}

const isWindows = platform() === 'win32'
const isMac = platform() === 'darwin'
const home = homedir()

/**
 * 创建 readline 接口用于用户交互
 */
function createReadlineInterface() {
  return createInterface({
    input: process.stdin,
    output: process.stdout,
  })
}

/**
 * 提示用户输入
 */
async function prompt(rl, question) {
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      resolve(answer.trim())
    })
  })
}

/**
 * 打印彩色日志
 */
function log(message, type = 'info') {
  const colors = {
    info: '\x1b[36m',    // 青色
    success: '\x1b[32m', // 绿色
    warn: '\x1b[33m',    // 黄色
    error: '\x1b[31m',   // 红色
    reset: '\x1b[0m',
  }
  const prefix = {
    info: 'ℹ',
    success: '✔',
    warn: '⚠',
    error: '✖',
  }
  console.log(`${colors[type]}${prefix[type]} ${message}${colors.reset}`)
}

/**
 * 检查 Rust 是否已安装
 */
function checkRustInstalled() {
  try {
    const version = execSync('rustc --version', { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'pipe'] })
    return version.trim()
  } catch {
    return null
  }
}

/**
 * 配置 Cargo 镜像
 */
function configureCargoMirror(mirror) {
  const cargoDir = join(home, '.cargo')
  const configPath = join(cargoDir, 'config.toml')

  // 确保 .cargo 目录存在
  if (!existsSync(cargoDir)) {
    mkdirSync(cargoDir, { recursive: true })
    log(`创建目录: ${cargoDir}`, 'info')
  }

  const mirrorName = Object.keys(MIRRORS).find(key => MIRRORS[key] === mirror)

  const configContent = `# Cargo 镜像配置 - 由 install-rust.mjs 生成
# 镜像源: ${mirror.name}

[source.crates-io]
replace-with = '${mirrorName}'

[source.${mirrorName}]
registry = "${mirror.cargo_registry}"

[net]
git-fetch-with-cli = true

[http]
check-revoke = false
`

  writeFileSync(configPath, configContent, 'utf-8')
  log(`Cargo 镜像配置已写入: ${configPath}`, 'success')
}

/**
 * 配置 Shell 环境变量 (Linux/macOS)
 */
function configureShellEnv(mirror) {
  const envLines = `
# Rust 镜像配置 - 由 install-rust.mjs 添加
export RUSTUP_DIST_SERVER="${mirror.rustup_dist}"
export RUSTUP_UPDATE_ROOT="${mirror.rustup_update}"
`

  // 检测使用的 shell
  const shell = process.env.SHELL || '/bin/bash'
  let rcFile

  if (shell.includes('zsh')) {
    rcFile = join(home, '.zshrc')
  } else if (shell.includes('fish')) {
    rcFile = join(home, '.config', 'fish', 'config.fish')
  } else {
    rcFile = join(home, '.bashrc')
  }

  // 检查是否已经配置过
  if (existsSync(rcFile)) {
    const content = readFileSync(rcFile, 'utf-8')
    if (content.includes('RUSTUP_DIST_SERVER')) {
      log(`环境变量已在 ${rcFile} 中配置，跳过`, 'warn')
      return
    }
  }

  appendFileSync(rcFile, envLines, 'utf-8')
  log(`环境变量已添加到: ${rcFile}`, 'success')
  log(`请运行 'source ${rcFile}' 或重新打开终端使配置生效`, 'info')
}

/**
 * 配置 Windows 环境变量
 */
function configureWindowsEnv(mirror) {
  try {
    // 使用 setx 设置用户环境变量
    execSync(`setx RUSTUP_DIST_SERVER "${mirror.rustup_dist}"`, { stdio: 'pipe' })
    execSync(`setx RUSTUP_UPDATE_ROOT "${mirror.rustup_update}"`, { stdio: 'pipe' })
    log('Windows 用户环境变量已设置', 'success')
    log('请重新打开终端使环境变量生效', 'info')
  } catch (error) {
    log(`设置环境变量失败: ${error.message}`, 'error')
    log('请手动设置环境变量:', 'info')
    console.log(`  RUSTUP_DIST_SERVER = ${mirror.rustup_dist}`)
    console.log(`  RUSTUP_UPDATE_ROOT = ${mirror.rustup_update}`)
  }
}

/**
 * 使用 Node.js 原生 https 模块下载文件
 */
async function downloadFile(url, destPath) {
  const https = await import('https')
  const http = await import('http')
  const { createWriteStream } = await import('fs')
  const { parse } = await import('url')

  return new Promise((resolve, reject) => {
    const file = createWriteStream(destPath)
    const parsedUrl = parse(url)
    const isHttps = parsedUrl.protocol === 'https:'
    const client = isHttps ? https : http

    const options = {
      hostname: parsedUrl.hostname,
      port: parsedUrl.port || (isHttps ? 443 : 80),
      path: parsedUrl.path,
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Accept': '*/*',
        'Accept-Encoding': 'identity',
        'Connection': 'keep-alive',
      },
    }

    const request = client.request(options, (response) => {
      // 处理重定向
      if (response.statusCode === 301 || response.statusCode === 302 || response.statusCode === 307 || response.statusCode === 308) {
        file.close()
        const redirectUrl = response.headers.location
        return downloadFile(redirectUrl, destPath)
          .then(resolve)
          .catch(reject)
      }

      if (response.statusCode !== 200) {
        file.close()
        return reject(new Error(`下载失败: HTTP ${response.statusCode}`))
      }

      const totalSize = parseInt(response.headers['content-length'], 10)
      let downloadedSize = 0

      response.on('data', (chunk) => {
        downloadedSize += chunk.length
        const percent = totalSize ? ((downloadedSize / totalSize) * 100).toFixed(1) : '?'
        process.stdout.write(`\r   下载进度: ${percent}% (${(downloadedSize / 1024 / 1024).toFixed(2)} MB)`)
      })

      response.pipe(file)

      file.on('finish', () => {
        file.close()
        console.log('\n')
        resolve()
      })

      file.on('error', (err) => {
        file.close()
        reject(err)
      })
    })

    request.on('error', (err) => {
      file.close()
      reject(err)
    })

    request.end()
  })
}

/**
 * 下载并运行 rustup 安装程序
 */
async function installRust(mirror) {
  // 设置当前进程的环境变量
  process.env.RUSTUP_DIST_SERVER = mirror.rustup_dist
  process.env.RUSTUP_UPDATE_ROOT = mirror.rustup_update

  log('开始下载 Rust 安装程序...', 'info')

  if (isWindows) {
    // Windows: 下载并运行 rustup-init.exe
    const installerUrl = `${mirror.rustup_dist}/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe`
    const installerPath = join(process.env.TEMP || 'C:\\Windows\\Temp', 'rustup-init.exe')

    try {
      log(`下载地址: ${installerUrl}`, 'info')

      // Windows 优先使用 curl (Windows 10+ 自带，最稳定可靠)
      let downloadSuccess = false

      try {
        log('使用 curl 下载...', 'info')
        execSync(
          `curl -fSL --progress-bar -o "${installerPath}" "${installerUrl}"`,
          { stdio: 'inherit' }
        )
        downloadSuccess = true
        log('下载完成', 'success')
      } catch (curlError) {
        log(`curl 下载失败，尝试其他方式`, 'warn')

        // 回退到 PowerShell 下载
        try {
          log('使用 PowerShell 下载...', 'info')
          execSync(
            `powershell -Command "$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '${installerUrl}' -OutFile '${installerPath}'"`,
            { stdio: 'pipe' }
          )
          downloadSuccess = true
          log('下载完成', 'success')
        } catch (psError) {
          log(`PowerShell 下载失败，尝试 Node.js`, 'warn')

          // 最后尝试 Node.js 原生下载
          try {
            log('使用 Node.js 下载...', 'info')
            await downloadFile(installerUrl, installerPath)
            downloadSuccess = true
            log('下载完成', 'success')
          } catch (nodeError) {
            throw new Error(`所有下载方式均失败。\n\n请手动下载安装:\n1. 访问: ${installerUrl}\n2. 保存到: ${installerPath}\n3. 运行安装程序`)
          }
        }
      }

      // 检查文件是否下载成功
      if (!existsSync(installerPath)) {
        throw new Error('安装程序下载失败，文件不存在')
      }

      // 检查文件大小（rustup-init.exe 通常大于 5MB）
      const fileSize = statSync(installerPath).size
      if (fileSize < 1024 * 1024 * 2) { // 小于 2MB 可能是错误页面
        throw new Error(`下载的文件大小异常 (${(fileSize / 1024).toFixed(2)} KB)，可能下载失败`)
      }

      log(`文件大小: ${(fileSize / 1024 / 1024).toFixed(2)} MB`, 'info')
      log('运行安装程序...', 'info')
      log('请在安装程序中按照提示完成安装', 'info')

      // 使用 cmd.exe 运行安装程序，更可靠
      const installer = spawn('cmd.exe', ['/c', installerPath], {
        stdio: 'inherit',
        env: {
          ...process.env,
          RUSTUP_DIST_SERVER: mirror.rustup_dist,
          RUSTUP_UPDATE_ROOT: mirror.rustup_update,
        },
      })

      return new Promise((resolve, reject) => {
        installer.on('close', (code) => {
          if (code === 0) {
            resolve()
          } else {
            reject(new Error(`安装程序退出码: ${code}`))
          }
        })
        installer.on('error', (error) => {
          reject(new Error(`运行安装程序失败: ${error.message}`))
        })
      })
    } catch (error) {
      throw new Error(`安装失败: ${error.message}`)
    }
  } else {
    // Linux/macOS: 使用 curl 运行安装脚本
    const installerUrl = `${mirror.rustup_dist}/rustup-init.sh`

    try {
      log(`下载地址: ${installerUrl}`, 'info')

      const installer = spawn('sh', ['-c', `curl --proto '=https' --tlsv1.2 -sSf ${installerUrl} | sh`], {
        stdio: 'inherit',
        env: {
          ...process.env,
          RUSTUP_DIST_SERVER: mirror.rustup_dist,
          RUSTUP_UPDATE_ROOT: mirror.rustup_update,
        },
      })

      return new Promise((resolve, reject) => {
        installer.on('close', (code) => {
          if (code === 0) {
            resolve()
          } else {
            reject(new Error(`安装脚本退出码: ${code}`))
          }
        })
        installer.on('error', reject)
      })
    } catch (error) {
      throw new Error(`运行安装脚本失败: ${error.message}`)
    }
  }
}

/**
 * 主函数
 */
async function main() {
  console.log('\n🦀 Rust 安装脚本 - 国内镜像加速\n')
  console.log(`平台: ${platform()}`)
  console.log(`用户目录: ${home}\n`)

  // 检查是否已安装
  const existingRust = checkRustInstalled()
  if (existingRust) {
    log(`检测到已安装 Rust: ${existingRust}`, 'warn')
  }

  const rl = createReadlineInterface()

  try {
    // 选择镜像源
    console.log('请选择镜像源:\n')
    const mirrorKeys = Object.keys(MIRRORS)
    mirrorKeys.forEach((key, index) => {
      console.log(`  ${index + 1}. ${MIRRORS[key].name} (${key})`)
    })
    console.log()

    const mirrorChoice = await prompt(rl, '请输入序号 (默认 1 - 字节跳动): ')
    const mirrorIndex = parseInt(mirrorChoice || '1', 10) - 1

    if (mirrorIndex < 0 || mirrorIndex >= mirrorKeys.length) {
      throw new Error('无效的选择')
    }

    const selectedMirrorKey = mirrorKeys[mirrorIndex]
    const selectedMirror = MIRRORS[selectedMirrorKey]

    log(`已选择: ${selectedMirror.name}`, 'success')
    console.log()

    // 配置 Cargo 镜像
    const configCargo = await prompt(rl, '是否配置 Cargo 镜像? (Y/n): ')
    if (configCargo.toLowerCase() !== 'n') {
      configureCargoMirror(selectedMirror)
    }

    // 配置环境变量
    const configEnv = await prompt(rl, '是否配置环境变量? (Y/n): ')
    if (configEnv.toLowerCase() !== 'n') {
      if (isWindows) {
        configureWindowsEnv(selectedMirror)
      } else {
        await configureShellEnv(selectedMirror)
      }
    }

    // 安装 Rust
    if (existingRust) {
      const reinstall = await prompt(rl, '是否重新安装/更新 Rust? (y/N): ')
      if (reinstall.toLowerCase() !== 'y') {
        log('跳过 Rust 安装', 'info')
        rl.close()
        return
      }
    }

    const doInstall = await prompt(rl, '是否开始安装 Rust? (Y/n): ')
    if (doInstall.toLowerCase() !== 'n') {
      rl.close()
      await installRust(selectedMirror)
      log('Rust 安装完成!', 'success')

      // 验证安装
      console.log('\n验证安装:')
      try {
        if (isWindows) {
          console.log('请重新打开终端后运行: rustc --version')
        } else {
          execSync('source $HOME/.cargo/env && rustc --version', {
            stdio: 'inherit',
            shell: true
          })
        }
      } catch {
        log('请重新打开终端后验证安装', 'info')
      }
    } else {
      rl.close()
      log('已取消安装', 'info')
    }

    console.log('\n配置信息:')
    console.log(`  镜像源: ${selectedMirror.name}`)
    console.log(`  RUSTUP_DIST_SERVER: ${selectedMirror.rustup_dist}`)
    console.log(`  RUSTUP_UPDATE_ROOT: ${selectedMirror.rustup_update}`)
    console.log(`  Cargo Registry: ${selectedMirror.cargo_registry}`)
    console.log()

  } catch (error) {
    rl.close()
    log(error.message, 'error')
    process.exit(1)
  }
}

// 运行主函数
main().catch((error) => {
  log(`发生错误: ${error.message}`, 'error')
  process.exit(1)
})
