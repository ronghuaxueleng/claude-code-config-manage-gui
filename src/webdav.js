// WebDAV 同步功能

const { invoke } = window.__TAURI__.core;

// 全局变量
let webdavConfigs = [];
let selectedWebdavConfig = null;

// 辅助函数：标准化错误消息处理
function getErrorMessage(error) {
    if (typeof error === 'string') {
        return error;
    } else if (error && error.message) {
        return error.message;
    } else if (error && typeof error.toString === 'function') {
        return error.toString();
    } else {
        return '未知错误';
    }
}

// Tauri 命令包装函数
async function tauriGetWebdavConfigs() {
    return await invoke('get_webdav_configs');
}

async function tauriGetActiveWebdavConfig() {
    return await invoke('get_active_webdav_config');
}

async function tauriCreateWebdavConfig(config) {
    return await invoke('create_webdav_config', {
        name: config.name,
        url: config.url,
        username: config.username,
        password: config.password,
        remotePath: config.remotePath || '/claude-config',
        autoSync: config.autoSync || false,
        syncInterval: config.syncInterval || 3600
    });
}

async function tauriUpdateWebdavConfig(id, config) {
    return await invoke('update_webdav_config', {
        id,
        name: config.name,
        url: config.url,
        username: config.username,
        password: config.password,
        remotePath: config.remotePath,
        autoSync: config.autoSync,
        syncInterval: config.syncInterval,
        isActive: config.isActive
    });
}

async function tauriDeleteWebdavConfig(id) {
    return await invoke('delete_webdav_config', { id });
}

async function tauriTestWebdavConnection(id) {
    return await invoke('test_webdav_connection', { id });
}

async function tauriUploadConfigToWebdav(configId, filename) {
    return await invoke('upload_config_to_webdav', {
        configId,
        filename
    });
}

async function tauriDownloadConfigFromWebdav(configId, filename) {
    return await invoke('download_config_from_webdav', {
        configId,
        filename
    });
}

async function tauriListWebdavFiles(configId) {
    return await invoke('list_webdav_files', { configId });
}

async function tauriGetSyncLogs(configId = null, limit = 50) {
    return await invoke('get_sync_logs', { configId, limit });
}

async function tauriDeleteRemoteFile(configId, filename) {
    return await invoke('delete_remote_file', { configId, filename });
}

// 加载 WebDAV 配置列表
async function loadWebdavConfigs() {
    try {
        webdavConfigs = await tauriGetWebdavConfigs();
        console.log('加载到的 WebDAV 配置:', webdavConfigs);
        const listElement = document.getElementById('webdavConfigsList');

        if (webdavConfigs.length === 0) {
            listElement.innerHTML = `
                <div class="text-center text-muted py-4">
                    <i class="fas fa-cloud-upload-alt fa-3x mb-3"></i>
                    <p>暂无 WebDAV 配置</p>
                    <p class="small">点击上方"添加配置"按钮开始</p>
                </div>
            `;
            return;
        }

        // 如果还没有选中的配置，自动选中活跃的配置
        if (!selectedWebdavConfig) {
            const activeConfig = webdavConfigs.find(c => c.is_active);
            if (activeConfig) {
                selectedWebdavConfig = activeConfig;
                console.log('自动选中活跃配置:', selectedWebdavConfig);
            }
        }

        listElement.innerHTML = webdavConfigs.map(config => {
            const selectedStyle = selectedWebdavConfig?.id === config.id ? 'border-primary border-2' : '';
            const activeBadge = config.is_active ? '<span class="badge bg-success ms-2">活跃</span>' : '';
            const autoSyncIcon = config.auto_sync ? ' | <i class="fas fa-sync-alt"></i> 自动同步' : '';

            return `
                <div class="list-group-item list-group-item-action ${selectedStyle}"
                     onclick="selectWebdavConfig(${config.id})">
                    <div class="d-flex w-100 justify-content-between align-items-center">
                        <h6 class="mb-1">
                            <i class="fas fa-cloud me-2"></i>${config.name}
                            ${activeBadge}
                        </h6>
                        <div>
                            <button class="btn btn-sm btn-outline-primary" onclick="event.stopPropagation(); editWebdavConfig(${config.id})">
                                <i class="fas fa-edit"></i>
                            </button>
                            <button class="btn btn-sm btn-outline-danger" onclick="event.stopPropagation(); deleteWebdavConfig(${config.id})">
                                <i class="fas fa-trash"></i>
                            </button>
                        </div>
                    </div>
                    <p class="mb-1 small">${config.url}</p>
                    <small class="text-muted">
                        用户: ${config.username} |
                        路径: ${config.remote_path}
                        ${autoSyncIcon}
                    </small>
                </div>
            `;
        }).join('');

    } catch (error) {
        window.showError('加载 WebDAV 配置失败: ' + getErrorMessage(error));
    }
}

// 选择 WebDAV 配置
async function selectWebdavConfig(id) {
    selectedWebdavConfig = webdavConfigs.find(c => c.id === id);
    console.log('选中的 WebDAV 配置:', selectedWebdavConfig);
    await loadWebdavConfigs();  // 重新渲染列表以显示选中状态
    await loadWebdavOperationPanel();  // 更新操作面板
}

// 加载 WebDAV 操作面板
async function loadWebdavOperationPanel() {
    const panel = document.getElementById('webdavOperationPanel');

    if (!selectedWebdavConfig) {
        panel.innerHTML = `
            <div class="alert alert-info">
                <i class="fas fa-info-circle me-2"></i>
                请先选择或添加 WebDAV 配置
            </div>
        `;
        return;
    }

    const today = new Date().toISOString().split('T')[0];
    panel.innerHTML = `
        <div class="mb-3">
            <h6><i class="fas fa-info-circle me-2"></i>当前配置: ${selectedWebdavConfig.name}</h6>
            <p class="small text-muted mb-0">${selectedWebdavConfig.url}</p>
        </div>

        <div class="row g-2 mb-3">
            <div class="col-6">
                <button class="btn btn-success w-100" onclick="testWebdavConnection()">
                    <i class="fas fa-plug me-2"></i>测试连接
                </button>
            </div>
            <div class="col-6">
                <button class="btn btn-primary w-100" onclick="setAsActiveWebdav()">
                    <i class="fas fa-check-circle me-2"></i>设为活跃
                </button>
            </div>
        </div>

        <hr>

        <h6><i class="fas fa-upload me-2"></i>上传配置</h6>
        <div class="input-group mb-3">
            <input type="text" class="form-control" id="uploadFilename"
                   value="config-${today}.json"
                   placeholder="文件名">
            <button class="btn btn-success" onclick="uploadConfigToWebdav()">
                <i class="fas fa-cloud-upload-alt me-2"></i>上传
            </button>
        </div>

        <h6><i class="fas fa-download me-2"></i>下载配置</h6>
        <div class="mb-3">
            <button class="btn btn-info btn-sm mb-2" onclick="listRemoteFiles()">
                <i class="fas fa-list me-2"></i>查看远程文件
            </button>
            <div id="remoteFilesList"></div>
        </div>
    `;
}

// 保存 WebDAV 配置
async function saveWebdavConfig() {
    const config = {
        name: document.getElementById('webdavName').value,
        url: document.getElementById('webdavUrl').value,
        username: document.getElementById('webdavUsername').value,
        password: document.getElementById('webdavPassword').value,
        remotePath: document.getElementById('webdavRemotePath').value || '/claude-config',
        autoSync: document.getElementById('webdavAutoSync').checked,
        syncInterval: parseInt(document.getElementById('webdavSyncInterval').value) || 3600
    };

    try {
        await tauriCreateWebdavConfig(config);
        window.showSuccess('WebDAV 配置创建成功');
        bootstrap.Modal.getInstance(document.getElementById('webdavConfigModal')).hide();
        await loadWebdavConfigs();
        await loadWebdavOperationPanel();  // 刷新操作面板
    } catch (error) {
        window.showError('创建 WebDAV 配置失败: ' + getErrorMessage(error));
    }
}

// 编辑 WebDAV 配置
async function editWebdavConfig(id) {
    const config = webdavConfigs.find(c => c.id === id);
    if (!config) return;

    document.getElementById('webdavName').value = config.name;
    document.getElementById('webdavUrl').value = config.url;
    document.getElementById('webdavUsername').value = config.username;
    document.getElementById('webdavPassword').value = config.password;
    document.getElementById('webdavRemotePath').value = config.remote_path;
    document.getElementById('webdavAutoSync').checked = config.auto_sync;
    document.getElementById('webdavSyncInterval').value = config.sync_interval;

    const modal = new bootstrap.Modal(document.getElementById('webdavConfigModal'));
    modal.show();

    // 修改保存按钮为更新
    const saveBtn = document.getElementById('saveWebdavConfig');
    saveBtn.textContent = '更新';
    saveBtn.onclick = async () => {
        const updatedConfig = {
            name: document.getElementById('webdavName').value,
            url: document.getElementById('webdavUrl').value,
            username: document.getElementById('webdavUsername').value,
            password: document.getElementById('webdavPassword').value,
            remotePath: document.getElementById('webdavRemotePath').value,
            autoSync: document.getElementById('webdavAutoSync').checked,
            syncInterval: parseInt(document.getElementById('webdavSyncInterval').value)
        };

        try {
            await tauriUpdateWebdavConfig(id, updatedConfig);
            window.showSuccess('WebDAV 配置更新成功');
            modal.hide();
            // 如果更新的是当前选中的配置，需要更新 selectedWebdavConfig
            if (selectedWebdavConfig && selectedWebdavConfig.id === id) {
                selectedWebdavConfig = { ...selectedWebdavConfig, ...updatedConfig, id };
            }
            await loadWebdavConfigs();
            await loadWebdavOperationPanel();  // 刷新操作面板
        } catch (error) {
            window.showError('更新 WebDAV 配置失败: ' + getErrorMessage(error));
        }
    };
}

// 删除 WebDAV 配置
async function deleteWebdavConfig(id) {
    const config = webdavConfigs.find(c => c.id === id);
    if (!config) return;

    const confirmed = await window.customConfirm(
        `确定要删除 WebDAV 配置 "${config.name}" 吗？`,
        '确认删除'
    );

    if (confirmed) {
        try {
            await tauriDeleteWebdavConfig(id);
            window.showSuccess('WebDAV 配置删除成功');
            if (selectedWebdavConfig?.id === id) {
                selectedWebdavConfig = null;
            }
            await loadWebdavConfigs();
            await loadWebdavOperationPanel();
        } catch (error) {
            window.showError('删除 WebDAV 配置失败: ' + getErrorMessage(error));
        }
    }
}

// 测试 WebDAV 连接
async function testWebdavConnection() {
    if (!selectedWebdavConfig) return;

    try {
        const result = await tauriTestWebdavConnection(selectedWebdavConfig.id);
        window.showSuccess(result);
    } catch (error) {
        window.showError('连接测试失败: ' + getErrorMessage(error));
    }
}

// 设为活跃配置
async function setAsActiveWebdav() {
    if (!selectedWebdavConfig) return;

    try {
        const updatedConfig = {
            name: selectedWebdavConfig.name,
            url: selectedWebdavConfig.url,
            username: selectedWebdavConfig.username,
            password: selectedWebdavConfig.password,
            remotePath: selectedWebdavConfig.remote_path,
            autoSync: selectedWebdavConfig.auto_sync,
            syncInterval: selectedWebdavConfig.sync_interval,
            isActive: true
        };
        await tauriUpdateWebdavConfig(selectedWebdavConfig.id, updatedConfig);
        // 更新 selectedWebdavConfig 的 is_active 状态
        selectedWebdavConfig.is_active = true;
        window.showSuccess('已设置为活跃配置');
        await loadWebdavConfigs();
        await loadWebdavOperationPanel();  // 刷新操作面板
    } catch (error) {
        window.showError('设置失败: ' + getErrorMessage(error));
    }
}

// 上传配置到 WebDAV
async function uploadConfigToWebdav() {
    if (!selectedWebdavConfig) return;

    const today = new Date().toISOString().split('T')[0];
    const filename = document.getElementById('uploadFilename').value || 'config-' + today + '.json';

    try {
        const result = await tauriUploadConfigToWebdav(selectedWebdavConfig.id, filename);
        window.showSuccess(result);
        await loadSyncLogs();
    } catch (error) {
        window.showError('上传失败: ' + getErrorMessage(error));
    }
}

// 列出远程文件
async function listRemoteFiles() {
    if (!selectedWebdavConfig) return;

    try {
        const files = await tauriListWebdavFiles(selectedWebdavConfig.id);
        console.log('远程文件列表:', files);
        const listElement = document.getElementById('remoteFilesList');

        if (files.length === 0) {
            listElement.innerHTML = '<div class="alert alert-info small">远程目录为空</div>';
            return;
        }

        listElement.innerHTML = `
            <div class="list-group small">
                ${files.map(file => `
                    <div class="list-group-item d-flex justify-content-between align-items-center">
                        <span><i class="fas fa-file-code me-2"></i>${file}</span>
                        <div>
                            <button class="btn btn-sm btn-success me-1" onclick="downloadConfigFromWebdav('${file}')" title="下载">
                                <i class="fas fa-download"></i>
                            </button>
                            <button class="btn btn-sm btn-danger" onclick="deleteRemoteFile('${file}')" title="删除">
                                <i class="fas fa-trash"></i>
                            </button>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    } catch (error) {
        window.showError('获取文件列表失败: ' + getErrorMessage(error));
    }
}

// 从 WebDAV 下载配置
async function downloadConfigFromWebdav(filename) {
    if (!selectedWebdavConfig) return;

    const confirmed = await window.customConfirm(
        `确定要下载并应用配置文件 "${filename}" 吗？\n这将覆盖当前配置！`,
        '确认下载'
    );

    if (confirmed) {
        try {
            const result = await tauriDownloadConfigFromWebdav(selectedWebdavConfig.id, filename);
            window.showSuccess('配置下载成功！');
            await loadSyncLogs();
        } catch (error) {
            window.showError('下载失败: ' + getErrorMessage(error));
        }
    }
}

// 删除远程文件
async function deleteRemoteFile(filename) {
    if (!selectedWebdavConfig) return;

    const confirmed = await window.customConfirm(
        `确定要删除远程文件 "${filename}" 吗？\n\n此操作不可撤销！`,
        '确认删除'
    );

    if (confirmed) {
        try {
            await tauriDeleteRemoteFile(selectedWebdavConfig.id, filename);
            window.showSuccess('文件删除成功');
            // 刷新文件列表
            await listRemoteFiles();
        } catch (error) {
            window.showError('删除文件失败: ' + getErrorMessage(error));
        }
    }
}

// 加载同步日志
async function loadSyncLogs() {
    try {
        const logs = await tauriGetSyncLogs(selectedWebdavConfig?.id, 3);
        const listElement = document.getElementById('syncLogsList');

        if (logs.length === 0) {
            listElement.innerHTML = `
                <div class="text-center text-muted py-3">
                    <i class="fas fa-history fa-2x mb-2"></i>
                    <p>暂无同步日志</p>
                </div>
            `;
            return;
        }

        listElement.innerHTML = logs.map(log => {
            const statusClass = log.status === 'success' ? 'success' :
                               log.status === 'failed' ? 'danger' : 'warning';
            const icon = log.sync_type === 'upload' ? 'fa-upload' : 'fa-download';
            const typeText = log.sync_type === 'upload' ? '上传' : '下载';
            const dateStr = new Date(log.synced_at).toLocaleString('zh-CN');

            return `
                <div class="border-bottom pb-2 mb-2">
                    <div class="d-flex justify-content-between align-items-start">
                        <div>
                            <i class="fas ${icon} me-2"></i>
                            <strong>${typeText}</strong>
                            <span class="badge bg-${statusClass} ms-2">${log.status}</span>
                        </div>
                        <small class="text-muted">${dateStr}</small>
                    </div>
                    ${log.message ? '<p class="small text-muted mb-0 mt-1">' + log.message + '</p>' : ''}
                </div>
            `;
        }).join('');

    } catch (error) {
        console.error('加载同步日志失败:', error);
    }
}

// 初始化 WebDAV 功能
document.addEventListener('DOMContentLoaded', () => {
    const saveBtn = document.getElementById('saveWebdavConfig');
    if (saveBtn) {
        saveBtn.onclick = saveWebdavConfig;
    }

    // 当切换到 WebDAV 标签页时加载数据
    const webdavTab = document.getElementById('webdav-tab');
    if (webdavTab) {
        webdavTab.addEventListener('shown.bs.tab', async () => {
            // 首次加载时运行数据库迁移，确保 WebDAV 表存在
            try {
                await invoke('migrate_database');
                console.log('数据库迁移检查完成');
            } catch (error) {
                console.warn('数据库迁移检查失败:', error);
            }

            await loadWebdavConfigs();
            await loadWebdavOperationPanel();  // 加载操作面板
            await loadSyncLogs();
        });
    }
});

// 导出函数到全局
window.loadWebdavConfigs = loadWebdavConfigs;
window.selectWebdavConfig = selectWebdavConfig;
window.saveWebdavConfig = saveWebdavConfig;
window.editWebdavConfig = editWebdavConfig;
window.deleteWebdavConfig = deleteWebdavConfig;
window.testWebdavConnection = testWebdavConnection;
window.setAsActiveWebdav = setAsActiveWebdav;
window.uploadConfigToWebdav = uploadConfigToWebdav;
window.downloadConfigFromWebdav = downloadConfigFromWebdav;
window.deleteRemoteFile = deleteRemoteFile;
window.listRemoteFiles = listRemoteFiles;
window.loadSyncLogs = loadSyncLogs;

// 调试函数
window.debugWebdav = function() {
    console.log('WebDAV 配置列表:', webdavConfigs);
    console.log('当前选中的配置:', selectedWebdavConfig);
    console.log('配置列表元素存在:', !!document.getElementById('webdavConfigsList'));
    console.log('操作面板元素存在:', !!document.getElementById('webdavOperationPanel'));
};
