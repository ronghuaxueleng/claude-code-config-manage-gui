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
        return window.i18n.t('error.unknown');
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
        console.log(window.i18n.t('webdav.configs_loaded') + ':', webdavConfigs);
        const listElement = document.getElementById('webdavConfigsList');

        if (webdavConfigs.length === 0) {
            listElement.innerHTML = `
                <div class="text-center text-muted py-4">
                    <i class="fas fa-cloud-upload-alt fa-3x mb-3"></i>
                    <p>${window.i18n.t('text.no_webdav')}</p>
                    <p class="small">${window.i18n.t('text.webdav_help')}</p>
                </div>
            `;
            return;
        }

        // 如果还没有选中的配置，自动选中活跃的配置
        if (!selectedWebdavConfig) {
            const activeConfig = webdavConfigs.find(c => c.is_active);
            if (activeConfig) {
                selectedWebdavConfig = activeConfig;
                console.log(window.i18n.t('webdav.auto_selected') + ':', selectedWebdavConfig);
            }
        }

        listElement.innerHTML = webdavConfigs.map(config => {
            const selectedStyle = selectedWebdavConfig?.id === config.id ? 'border-primary border-2' : '';
            const activeBadge = config.is_active ? '<span class="badge bg-success ms-2">' + window.i18n.t('text.active') + '</span>' : '';
            const autoSyncIcon = config.auto_sync ? ' | <i class="fas fa-sync-alt"></i> ' + window.i18n.t('text.auto_sync') : '';

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
                        ${window.i18n.t('text.user')}: ${config.username} |
                        ${window.i18n.t('text.path')}: ${config.remote_path}
                        ${autoSyncIcon}
                    </small>
                </div>
            `;
        }).join('');

    } catch (error) {
        window.showError(window.i18n.t('text.webdav_load_failed') + ': ' + getErrorMessage(error));
    }
}

// 选择 WebDAV 配置
async function selectWebdavConfig(id) {
    selectedWebdavConfig = webdavConfigs.find(c => c.id === id);
    console.log(window.i18n.t('webdav.config_selected') + ':', selectedWebdavConfig);
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
                ${window.i18n.t('info.select_directory_or_add')}
            </div>
        `;
        return;
    }

    const today = new Date().toISOString().split('T')[0];
    panel.innerHTML = `
        <div class="mb-3">
            <h6><i class="fas fa-info-circle me-2"></i>${window.i18n.t('text.current_config')}: ${selectedWebdavConfig.name}</h6>
            <p class="small text-muted mb-0">${selectedWebdavConfig.url}</p>
        </div>

        <div class="row g-2 mb-3">
            <div class="col-6">
                <button class="btn btn-success w-100" onclick="testWebdavConnection()">
                    <i class="fas fa-plug me-2"></i>${window.i18n.t('text.test_connection')}
                </button>
            </div>
            <div class="col-6">
                <button class="btn btn-primary w-100" onclick="setAsActiveWebdav()">
                    <i class="fas fa-check-circle me-2"></i>${window.i18n.t('text.set_active')}
                </button>
            </div>
        </div>

        <hr>

        <h6><i class="fas fa-upload me-2"></i>${window.i18n.t('text.upload_config')}</h6>
        <div class="input-group mb-3">
            <input type="text" class="form-control" id="uploadFilename"
                   value="config-${today}.json"
                   placeholder="${window.i18n.t('text.filename')}">
            <button class="btn btn-success" onclick="uploadConfigToWebdav()">
                <i class="fas fa-cloud-upload-alt me-2"></i>${window.i18n.t('text.upload')}
            </button>
        </div>

        <h6><i class="fas fa-download me-2"></i>${window.i18n.t('text.download_config')}</h6>
        <div class="mb-3">
            <button class="btn btn-info btn-sm mb-2" onclick="listRemoteFiles()">
                <i class="fas fa-list me-2"></i>${window.i18n.t('text.view_remote_files')}
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
        window.showSuccess(window.i18n.t('text.webdav_create_success'));
        bootstrap.Modal.getInstance(document.getElementById('webdavConfigModal')).hide();
        await loadWebdavConfigs();
        await loadWebdavOperationPanel();  // 刷新操作面板
    } catch (error) {
        window.showError(window.i18n.t('text.webdav_create_failed') + ': ' + getErrorMessage(error));
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
    saveBtn.textContent = window.i18n.t('button.update');
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
            window.showSuccess(window.i18n.t('text.webdav_update_success'));
            modal.hide();
            // 如果更新的是当前选中的配置，需要更新 selectedWebdavConfig
            if (selectedWebdavConfig && selectedWebdavConfig.id === id) {
                selectedWebdavConfig = { ...selectedWebdavConfig, ...updatedConfig, id };
            }
            await loadWebdavConfigs();
            await loadWebdavOperationPanel();  // 刷新操作面板
        } catch (error) {
            window.showError(window.i18n.t('text.webdav_update_failed') + ': ' + getErrorMessage(error));
        }
    };
}

// 删除 WebDAV 配置
async function deleteWebdavConfig(id) {
    const config = webdavConfigs.find(c => c.id === id);
    if (!config) return;

    const confirmed = await window.customConfirm(
        window.i18n.t('text.webdav_confirm_delete').replace('{name}', config.name),
        window.i18n.t('text.webdav_confirm_delete_title')
    );

    if (confirmed) {
        try {
            await tauriDeleteWebdavConfig(id);
            window.showSuccess(window.i18n.t('text.webdav_delete_success'));
            if (selectedWebdavConfig?.id === id) {
                selectedWebdavConfig = null;
            }
            await loadWebdavConfigs();
            await loadWebdavOperationPanel();
        } catch (error) {
            window.showError(window.i18n.t('text.webdav_delete_failed') + ': ' + getErrorMessage(error));
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
        window.showError(window.i18n.t('text.webdav_test_failed') + ': ' + getErrorMessage(error));
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
        window.showSuccess(window.i18n.t('text.webdav_set_active_success'));
        await loadWebdavConfigs();
        await loadWebdavOperationPanel();  // 刷新操作面板
    } catch (error) {
        window.showError(window.i18n.t('text.webdav_set_active_failed') + ': ' + getErrorMessage(error));
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
        window.showError(window.i18n.t('text.webdav_upload_failed') + ': ' + getErrorMessage(error));
    }
}

// 列出远程文件
async function listRemoteFiles() {
    if (!selectedWebdavConfig) return;

    try {
        const files = await tauriListWebdavFiles(selectedWebdavConfig.id);
        console.log(window.i18n.t('webdav.remote_files') + ':', files);
        const listElement = document.getElementById('remoteFilesList');

        if (files.length === 0) {
            listElement.innerHTML = '<div class="alert alert-info small">' + window.i18n.t('text.remote_dir_empty') + '</div>';
            return;
        }

        listElement.innerHTML = `
            <div class="list-group small">
                ${files.map(file => `
                    <div class="list-group-item d-flex justify-content-between align-items-center">
                        <span><i class="fas fa-file-code me-2"></i>${file}</span>
                        <div>
                            <button class="btn btn-sm btn-success me-1" onclick="downloadConfigFromWebdav('${file}')" title="${window.i18n.t('button.download')}">
                                <i class="fas fa-download"></i>
                            </button>
                            <button class="btn btn-sm btn-danger" onclick="deleteRemoteFile('${file}')" title="${window.i18n.t('common.delete')}">
                                <i class="fas fa-trash"></i>
                            </button>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    } catch (error) {
        window.showError(window.i18n.t('text.webdav_list_failed') + ': ' + getErrorMessage(error));
    }
}

// 从 WebDAV 下载配置
async function downloadConfigFromWebdav(filename) {
    if (!selectedWebdavConfig) return;

    const confirmed = await window.customConfirm(
        window.i18n.t('text.webdav_confirm_download').replace('{filename}', filename),
        window.i18n.t('text.webdav_confirm_download_title')
    );

    if (confirmed) {
        try {
            const result = await tauriDownloadConfigFromWebdav(selectedWebdavConfig.id, filename);
            window.showSuccess(window.i18n.t('text.webdav_download_success'));
            await loadSyncLogs();
        } catch (error) {
            window.showError(window.i18n.t('text.webdav_download_failed') + ': ' + getErrorMessage(error));
        }
    }
}

// 删除远程文件
async function deleteRemoteFile(filename) {
    if (!selectedWebdavConfig) return;

    const confirmed = await window.customConfirm(
        window.i18n.t('text.webdav_confirm_delete_file').replace('{filename}', filename),
        window.i18n.t('text.webdav_confirm_delete_title')
    );

    if (confirmed) {
        try {
            await tauriDeleteRemoteFile(selectedWebdavConfig.id, filename);
            window.showSuccess(window.i18n.t('text.webdav_file_delete_success'));
            // 刷新文件列表
            await listRemoteFiles();
        } catch (error) {
            window.showError(window.i18n.t('text.webdav_file_delete_failed') + ': ' + getErrorMessage(error));
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
                    <p>${window.i18n.t('text.no_sync_logs')}</p>
                </div>
            `;
            return;
        }

        listElement.innerHTML = logs.map(log => {
            const statusClass = log.status === 'success' ? 'success' :
                               log.status === 'failed' ? 'danger' : 'warning';
            const icon = log.sync_type === 'upload' ? 'fa-upload' : 'fa-download';
            const typeText = log.sync_type === 'upload' ? window.i18n.t('webdav.upload_text') : window.i18n.t('webdav.download_text');

            // 翻译状态文本
            let statusText = log.status;
            if (log.status === 'success') {
                statusText = window.i18n.t('webdav.status_success');
            } else if (log.status === 'failed') {
                statusText = window.i18n.t('webdav.status_failed');
            } else if (log.status === 'warning') {
                statusText = window.i18n.t('webdav.status_warning');
            }

            // 根据当前语言格式化日期
            const locale = window.i18n.getLanguage() === 'zh-CN' ? 'zh-CN' : 'en-US';
            const dateStr = new Date(log.synced_at).toLocaleString(locale);

            return `
                <div class="border-bottom pb-2 mb-2">
                    <div class="d-flex justify-content-between align-items-start">
                        <div>
                            <i class="fas ${icon} me-2"></i>
                            <strong>${typeText}</strong>
                            <span class="badge bg-${statusClass} ms-2">${statusText}</span>
                        </div>
                        <small class="text-muted">${dateStr}</small>
                    </div>
                    ${log.message ? '<p class="small text-muted mb-0 mt-1">' + log.message + '</p>' : ''}
                </div>
            `;
        }).join('');

    } catch (error) {
        console.error(window.i18n.t('webdav.load_sync_logs_failed') + ':', error);
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
                console.log(window.i18n.t('webdav.migration_complete'));
            } catch (error) {
                console.warn(window.i18n.t('webdav.migration_failed') + ':', error);
            }

            await loadWebdavConfigs();
            await loadWebdavOperationPanel();  // 加载操作面板
            await loadSyncLogs();
        });
    }
});

// 导出函数到全局
window.loadWebdavConfigs = loadWebdavConfigs;
window.loadWebdavOperationPanel = loadWebdavOperationPanel;
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
    console.log(window.i18n.t('webdav.debug_configs') + ':', webdavConfigs);
    console.log(window.i18n.t('webdav.debug_selected') + ':', selectedWebdavConfig);
    console.log(window.i18n.t('webdav.debug_list_element') + ':', !!document.getElementById('webdavConfigsList'));
    console.log(window.i18n.t('webdav.debug_panel_element') + ':', !!document.getElementById('webdavOperationPanel'));
};
