const { invoke } = window.__TAURI__.core;
const { open, ask } = window.__TAURI__.dialog;

// 全局变量跟踪测试状态
let lastTestedConnection = null;
let lastTestResult = null;

// Global variables
let accounts = [];
let directories = [];
let baseUrls = [];
let currentAccountPage = 1;
let accountsPerPage = 5;
let currentAccountFilter = {
    search: '',
    base_url: ''
};
let currentDirectoryForAssociation = null;
let associationAccounts = [];
let associationDirectories = [];

// Tauri command wrappers
async function tauriGetAccounts(params = {}) {
    return await invoke('get_accounts', { request: params });
}

async function tauriCreateAccount(name, token, base_url, model) {
    return await invoke('create_account', { 
        name, 
        token, 
        baseUrl: base_url, 
        model 
    });
}

async function tauriUpdateAccount(id, params) {
    const requestParams = { id };
    // Convert base_url to baseUrl if present
    Object.keys(params).forEach(key => {
        if (key === 'base_url') {
            requestParams['baseUrl'] = params[key];
        } else {
            requestParams[key] = params[key];
        }
    });
    return await invoke('update_account', requestParams);
}

async function tauriDeleteAccount(id) {
    return await invoke('delete_account', { id });
}

async function tauriGetAccountBaseUrls() {
    return await invoke('get_account_base_urls');
}

async function tauriGetDirectories() {
    return await invoke('get_directories');
}

async function tauriCreateDirectory(path, name) {
    return await invoke('create_directory', { path, name });
}

async function tauriUpdateDirectory(id, params) {
    return await invoke('update_directory', { id, ...params });
}

async function tauriDeleteDirectory(id) {
    return await invoke('delete_directory', { id });
}

async function tauriCheckDirectoryExists(path) {
    return await invoke('check_directory_exists', { path });
}

async function tauriGetBaseUrls() {
    return await invoke('get_base_urls');
}

async function tauriCreateBaseUrl(name, url, description, is_default) {
    return await invoke('create_base_url', { name, url, description, is_default });
}

async function tauriUpdateBaseUrl(id, params) {
    return await invoke('update_base_url', { id, ...params });
}

async function tauriDeleteBaseUrl(id) {
    return await invoke('delete_base_url', { id });
}

async function tauriSwitchAccount(account_id, directory_id, is_sandbox = false, skip_permissions = false) {
    return await invoke('switch_account', { 
        accountId: parseInt(account_id), 
        directoryId: parseInt(directory_id),
        isSandbox: is_sandbox,
        skipPermissions: skip_permissions
    });
}

async function tauriSwitchAccountWithClaudeSettings(account_id, directory_id, is_sandbox, claude_settings) {
    return await invoke('switch_account_with_claude_settings', { 
        accountId: parseInt(account_id), 
        directoryId: parseInt(directory_id),
        isSandbox: is_sandbox,
        claudeSettings: claude_settings
    });
}

async function tauriGetCurrentConfig(directory_id) {
    return await invoke('get_current_config', { 
        directoryId: parseInt(directory_id) 
    });
}

async function tauriGetAssociations() {
    return await invoke('get_associations');
}

async function tauriSwitchDatabase(connectionName) {
    return await invoke('switch_database', { connectionName });
}

async function tauriTestDatabaseConnection(connectionName) {
    return await invoke('test_database', { connectionName });
}

async function tauriGetDatabaseConnections() {
    return await invoke('get_database_connections');
}

async function tauriGetDatabaseInfo() {
    return await invoke('get_database_info');
}

// API call function for consistency
async function apiCall(fn, params = {}) {
    try {
        return await fn(params);
    } catch (error) {
        throw new Error(error);
    }
}

// 标准化错误消息处理函数
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

// Load accounts list
async function loadAccounts(page = 1, resetPage = false) {
    try {
        if (resetPage) {
            currentAccountPage = 1;
            page = 1;
        } else {
            currentAccountPage = page;
        }
        
        const params = {
            page: page,
            per_page: accountsPerPage
        };
        
        if (currentAccountFilter.search) {
            params.search = currentAccountFilter.search;
        }
        
        if (currentAccountFilter.base_url) {
            params.base_url = currentAccountFilter.base_url;
        }
        
        const response = await tauriGetAccounts(params);
        accounts = response.accounts || [];
        
        await renderAccounts();
        renderAccountsPagination(response.pagination);
    } catch (error) {
        showError('加载账号失败: ' + getErrorMessage(error));
    }
}

// Render accounts list
async function renderAccounts() {
    const container = document.getElementById('accountsList');
    
    if (accounts.length === 0) {
        container.innerHTML = '<div class="text-muted">暂无账号</div>';
        return;
    }
    
    try {
        const allAssociations = await tauriGetAssociations();
        
        const accountsWithAssociations = accounts.map(account => {
            const hasAssociation = allAssociations.some(assoc => assoc.account_id === account.id);
            account.has_associations = hasAssociation;
            return account;
        });
        
        container.innerHTML = accountsWithAssociations.map(account => `
        <div class="list-group-item ${account.has_associations ? 'associated' : ''}">
            <div class="account-item">
                <div class="account-info">
                    <div class="fw-bold">
                        ${account.has_associations ? '<span class="association-indicator me-2"></span>' : ''}
                        ${account.name}
                        ${account.is_active ? '<span class="badge bg-success ms-2">当前活跃</span>' : ''}
                    </div>
                    <div class="small token-preview">${account.token.substring(0, 20)}...</div>
                    <div class="small">${account.base_url}</div>
                </div>
                <div class="account-actions">
                    <button class="btn btn-sm btn-outline-primary" onclick="editAccount(${account.id})">编辑</button>
                    <button class="btn btn-sm btn-outline-danger" onclick="promptDeleteAccount(${account.id})">删除</button>
                </div>
            </div>
        </div>
    `).join('');
    
    } catch (error) {
        console.warn('获取关联关系失败:', getErrorMessage(error));
        container.innerHTML = accounts.map(account => `
        <div class="list-group-item">
            <div class="account-item">
                <div class="account-info">
                    <div class="fw-bold">${account.name}</div>
                    <div class="text-muted">${account.base_url}</div>
                </div>
                <div class="account-actions">
                    <button class="btn btn-sm btn-outline-primary" onclick="editAccount(${account.id})">编辑</button>
                    <button class="btn btn-sm btn-outline-danger" onclick="promptDeleteAccount(${account.id})">删除</button>
                </div>
            </div>
        </div>
    `).join('');
    }
}

// Render pagination component
function renderAccountsPagination(pagination) {
    const container = document.getElementById('accountsPagination');
    
    if (!pagination || pagination.pages <= 1) {
        container.innerHTML = '';
        return;
    }
    
    let paginationHtml = '';
    
    // Previous page button
    if (pagination.has_prev) {
        paginationHtml += `
            <li class="page-item">
                <button type="button" class="page-link" onclick="loadAccounts(${pagination.prev_num})">
                    <i class="fas fa-chevron-left"></i> 上一页
                </button>
            </li>
        `;
    } else {
        paginationHtml += `
            <li class="page-item disabled">
                <span class="page-link"><i class="fas fa-chevron-left"></i> 上一页</span>
            </li>
        `;
    }
    
    // Page number buttons
    const startPage = Math.max(1, pagination.page - 2);
    const endPage = Math.min(pagination.pages, pagination.page + 2);
    
    if (startPage > 1) {
        paginationHtml += `
            <li class="page-item">
                <button type="button" class="page-link" onclick="loadAccounts(1)">1</button>
            </li>
        `;
        if (startPage > 2) {
            paginationHtml += `<li class="page-item disabled"><span class="page-link">...</span></li>`;
        }
    }
    
    for (let i = startPage; i <= endPage; i++) {
        if (i === pagination.page) {
            paginationHtml += `
                <li class="page-item active">
                    <span class="page-link">${i}</span>
                </li>
            `;
        } else {
            paginationHtml += `
                <li class="page-item">
                    <button type="button" class="page-link" onclick="loadAccounts(${i})">${i}</button>
                </li>
            `;
        }
    }
    
    if (endPage < pagination.pages) {
        if (endPage < pagination.pages - 1) {
            paginationHtml += `<li class="page-item disabled"><span class="page-link">...</span></li>`;
        }
        paginationHtml += `
            <li class="page-item">
                <button type="button" class="page-link" onclick="loadAccounts(${pagination.pages})">${pagination.pages}</button>
            </li>
        `;
    }
    
    // Next page button
    if (pagination.has_next) {
        paginationHtml += `
            <li class="page-item">
                <button type="button" class="page-link" onclick="loadAccounts(${pagination.next_num})">
                    下一页 <i class="fas fa-chevron-right"></i>
                </button>
            </li>
        `;
    } else {
        paginationHtml += `
            <li class="page-item disabled">
                <span class="page-link">下一页 <i class="fas fa-chevron-right"></i></span>
            </li>
        `;
    }
    
    container.innerHTML = paginationHtml;
}

// Load account base URL options
async function loadAccountBaseUrlOptions() {
    try {
        // 获取账号中实际使用的URL作为筛选选项
        const baseUrlOptions = await tauriGetAccountBaseUrls();
        const select = document.getElementById('baseUrlFilter');
        
        // Keep "All URLs" option
        select.innerHTML = '<option value="">所有URL</option>';
        
        baseUrlOptions.forEach(url => {
            const option = document.createElement('option');
            option.value = url;
            option.textContent = url;
            select.appendChild(option);
        });
        
        // 恢复之前选择的筛选值
        if (currentAccountFilter.base_url) {
            select.value = currentAccountFilter.base_url;
        }
    } catch (error) {
        // 加载base_url选项失败，静默处理
    }
}

// Set up filter and search event listeners
function setupAccountFilters() {
    const searchInput = document.getElementById('accountSearch');
    const baseUrlFilter = document.getElementById('baseUrlFilter');
    const perPageSelect = document.getElementById('perPageSelect');
    
    // Search with delay
    let searchTimeout;
    searchInput.addEventListener('input', function() {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            currentAccountFilter.search = this.value;
            loadAccounts(1, true);
        }, 500);
    });
    
    // URL filter
    baseUrlFilter.addEventListener('change', function() {
        currentAccountFilter.base_url = this.value;
        loadAccounts(1, true);
    });
    
    // Per page count
    perPageSelect.addEventListener('change', function() {
        accountsPerPage = parseInt(this.value);
        loadAccounts(1, true);
    });
}

// Load directories list
async function loadDirectories() {
    try {
        directories = await tauriGetDirectories();
        renderDirectories();
    } catch (error) {
        showError('加载目录失败: ' + getErrorMessage(error));
    }
}

// Render directories list
async function renderDirectories() {
    const container = document.getElementById('directoriesList');
    
    if (directories.length === 0) {
        container.innerHTML = '<div class="text-muted">暂无目录</div>';
        return;
    }
    
    // 检查每个目录是否在文件系统中存在
    const directoriesWithStatus = await Promise.all(
        directories.map(async directory => {
            try {
                const exists = await tauriCheckDirectoryExists(directory.path);
                return { ...directory, exists };
            } catch (error) {
                console.warn('检查目录存在性失败:', error);
                return { ...directory, exists: true }; // 默认认为存在
            }
        })
    );
    
    container.innerHTML = directoriesWithStatus.map(directory => `
        <div class="list-group-item ${!directory.exists ? 'directory-missing' : ''}">
            <div class="directory-item">
                <div class="directory-info">
                    <div class="fw-bold">
                        ${directory.is_active ? '<span class="directory-active-indicator"></span>' : ''}${directory.name}
                        ${!directory.exists ? '<span class="badge bg-warning text-dark ms-2">文件夹不存在</span>' : ''}
                    </div>
                    <div class="small text-muted">${directory.path}</div>
                    ${!directory.exists ? '<div class="small text-warning"><i class="fas fa-exclamation-triangle me-1"></i>该目录在文件系统中不存在，点击删除将清理数据库记录</div>' : ''}
                </div>
                <div class="directory-actions">
                    <button class="btn btn-sm btn-outline-info" onclick="viewConfig(${directory.id})" ${!directory.exists ? 'disabled title="目录不存在，无法查看配置"' : ''}>查看配置</button>
                    <button class="btn btn-sm btn-outline-primary" onclick="editDirectory(${directory.id})" ${!directory.exists ? 'disabled title="目录不存在，请先删除记录"' : ''}>编辑</button>
                    <button class="btn btn-sm ${!directory.exists ? 'btn-warning' : 'btn-outline-danger'}" onclick="promptDeleteDirectory(${directory.id}, '${directory.name.replace(/'/g, '\\\'')}')" ${isDeleting ? 'disabled' : ''}>
                        ${!directory.exists ? '清理记录' : '删除'}
                    </button>
                </div>
            </div>
        </div>
    `).join('');
}

// Save account
async function saveAccount() {
    const name = document.getElementById('accountName').value.trim();
    const token = document.getElementById('accountToken').value.trim();
    const base_url = document.getElementById('accountBaseUrl').value.trim();
    
    if (!name || !token || !base_url) {
        showError('请填写所有必需字段');
        return;
    }
    
    // 验证base_url格式
    try {
        new URL(base_url);
    } catch (e) {
        showError('请输入有效的Base URL格式（例如：https://api.anthropic.com）');
        return;
    }
    
    // 确保URL以http或https开头
    if (!base_url.startsWith('http://') && !base_url.startsWith('https://')) {
        showError('Base URL必须以http://或https://开头');
        return;
    }
    
    try {
        const result = await tauriCreateAccount(name, token, base_url, 'claude-sonnet-4-20250514');
        
        // Close modal and reset form
        const modal = bootstrap.Modal.getInstance(document.getElementById('accountModal'));
        modal.hide();
        document.getElementById('accountForm').reset();
        
        // Reload accounts list and base_url options
        await loadAccounts(currentAccountPage);
        await loadAccountBaseUrlOptions();
        showSuccess('账号添加成功');
    } catch (error) {
        // 处理特定的数据库错误
        let errorMessage = getErrorMessage(error);
        if (errorMessage.includes('UNIQUE constraint failed: accounts.name')) {
            errorMessage = '账号名称已存在，请使用不同的名称';
        }
        
        showError('添加账号失败: ' + errorMessage);
    }
}

// Global variable to track editing state
let editingDirectoryId = null;

// Save directory (handles both create and update)
async function saveDirectory() {
    const name = document.getElementById('directoryName').value.trim();
    const path = document.getElementById('directoryPath').value.trim();
    
    if (!name || !path) {
        showError('请填写所有必需字段');
        return;
    }
    
    // 验证路径是否存在
    try {
        const pathExists = await tauriCheckDirectoryExists(path);
        if (!pathExists) {
            const confirmed = confirm(`路径 "${path}" 在文件系统中不存在。\n\n确定要继续添加吗？\n\n注意：您稍后可以在目录列表中清理不存在的目录记录。`);
            if (!confirmed) {
                return;
            }
        }
    } catch (error) {
        console.warn('无法验证路径存在性:', error);
        // 继续执行，不阻止操作
    }
    
    try {
        if (editingDirectoryId) {
            // Update existing directory
            await tauriUpdateDirectory(editingDirectoryId, { name, path });
            showSuccess('目录更新成功');
        } else {
            // Create new directory
            await tauriCreateDirectory(path, name);
            showSuccess('目录添加成功');
        }
        
        // Close modal and reset form
        const modal = bootstrap.Modal.getInstance(document.getElementById('directoryModal'));
        modal.hide();
        resetDirectoryModal();
        
        // Reload directories list
        await loadDirectories();
    } catch (error) {
        if (editingDirectoryId) {
            showError('更新目录失败: ' + getErrorMessage(error));
        } else {
            showError('添加目录失败: ' + getErrorMessage(error));
        }
    }
}

// Edit account
async function editAccount(accountId) {
    try {
        const account = accounts.find(acc => acc.id === accountId);
        if (!account) {
            showError('找不到账号信息');
            return;
        }
        
        // Fill form
        document.getElementById('accountName').value = account.name;
        document.getElementById('accountToken').value = account.token;
        document.getElementById('accountBaseUrl').value = account.base_url;
        
        // Change modal title
        document.querySelector('#accountModal .modal-title').textContent = '编辑账号';
        
        // Change save button behavior
        const saveBtn = document.getElementById('saveAccount');
        // 移除所有现有的事件监听器
        saveBtn.removeEventListener('click', saveAccount);
        // 创建新的更新账号处理函数
        const updateAccountHandler = async function() {
            await updateAccount(accountId);
        };
        saveBtn.addEventListener('click', updateAccountHandler);
        // 保存处理函数引用，以便后续清理
        saveBtn._updateHandler = updateAccountHandler;
        
        // Show modal
        const modal = new bootstrap.Modal(document.getElementById('accountModal'));
        modal.show();
        
    } catch (error) {
        showError('编辑账号失败: ' + getErrorMessage(error));
    }
}

// Update account
async function updateAccount(accountId) {
    const name = document.getElementById('accountName').value.trim();
    const token = document.getElementById('accountToken').value.trim();
    const base_url = document.getElementById('accountBaseUrl').value.trim();
    
    if (!name || !token || !base_url) {
        showError('请填写所有必需字段');
        return;
    }
    
    // 验证base_url格式
    try {
        new URL(base_url);
    } catch (e) {
        showError('请输入有效的Base URL格式（例如：https://api.anthropic.com）');
        return;
    }
    
    // 确保URL以http或https开头
    if (!base_url.startsWith('http://') && !base_url.startsWith('https://')) {
        showError('Base URL必须以http://或https://开头');
        return;
    }
    
    try {
        const result = await tauriUpdateAccount(accountId, { name, token, base_url, model: 'claude-sonnet-4-20250514' });
        
        // Close modal and reset form
        const modal = bootstrap.Modal.getInstance(document.getElementById('accountModal'));
        modal.hide();
        
        // Reset form and modal state
        resetAccountModal();
        
        // Reload accounts list and base_url options
        await loadAccounts(currentAccountPage);
        await loadAccountBaseUrlOptions();
        showSuccess('账号更新成功');
    } catch (error) {
        // 处理特定的数据库错误
        let errorMessage = getErrorMessage(error);
        if (errorMessage.includes('UNIQUE constraint failed: accounts.name')) {
            errorMessage = '账号名称已存在，请使用不同的名称';
        }
        
        showError('更新账号失败: ' + errorMessage);
    }
}

// Reset account modal state
function resetAccountModal() {
    document.getElementById('accountForm').reset();
    document.querySelector('#accountModal .modal-title').textContent = '添加账号';
    
    // Reset save button behavior
    const saveBtn = document.getElementById('saveAccount');
    // 移除可能存在的更新账号处理函数
    if (saveBtn._updateHandler) {
        saveBtn.removeEventListener('click', saveBtn._updateHandler);
        saveBtn._updateHandler = null;
    }
    // 确保添加账号处理函数存在
    saveBtn.removeEventListener('click', saveAccount);
    saveBtn.addEventListener('click', saveAccount);
    saveBtn.onclick = null; // 清除onclick属性
}

// Prompt delete account - shows confirmation first
async function promptDeleteAccount(accountId) {
    // 找到要删除的账号信息
    const account = accounts.find(acc => acc.id === accountId);
    const accountName = account ? account.name : `ID: ${accountId}`;
    
    // 显示确认框
    try {
        const userConfirmed = await ask(
            `确定要删除账号 "${accountName}" 吗？\n\n此操作不可撤销！`,
            { 
                title: '确认删除账号', 
                type: 'warning' 
            }
        );
        
        if (!userConfirmed) {
            return;
        }
        
        // 用户确认后才执行删除
        await executeDeleteAccount(accountId, accountName);
    } catch (error) {
        // 如果 Tauri 对话框失败，fallback 到浏览器 confirm
        const userConfirmed = confirm(`确定要删除账号 "${accountName}" 吗？\n\n此操作不可撤销！`);
        if (userConfirmed) {
            await executeDeleteAccount(accountId, accountName);
        }
    }
}

// 执行实际的账号删除操作
async function executeDeleteAccount(accountId, accountName) {
    try {
        await tauriDeleteAccount(accountId);
        await loadAccounts(currentAccountPage);
        await loadAccountBaseUrlOptions();
        showSuccess(`账号 "${accountName}" 删除成功`);
    } catch (error) {
        showError(`删除账号 "${accountName}" 失败: ` + getErrorMessage(error));
    }
}

// Delete account (legacy function for compatibility)
async function deleteAccount(accountId) {
    if (!confirm('确定要删除这个账号吗？')) {
        return;
    }
    
    try {
        await tauriDeleteAccount(accountId);
        await loadAccounts(currentAccountPage);
        await loadAccountBaseUrlOptions();
        showSuccess('账号删除成功');
    } catch (error) {
        showError('删除账号失败: ' + getErrorMessage(error));
    }
}

// Edit directory
async function editDirectory(directoryId) {
    try {
        const directory = directories.find(dir => dir.id === directoryId);
        if (!directory) {
            showError('找不到目录信息');
            return;
        }
        
        // Set editing state
        editingDirectoryId = directoryId;
        
        // Fill form
        document.getElementById('directoryName').value = directory.name;
        document.getElementById('directoryPath').value = directory.path;
        
        // Change modal title
        document.querySelector('#directoryModal .modal-title').textContent = '编辑目录';
        
        // Show modal
        const modal = new bootstrap.Modal(document.getElementById('directoryModal'));
        modal.show();
        
    } catch (error) {
        showError('编辑目录失败: ' + getErrorMessage(error));
    }
}


// Reset directory modal state
function resetDirectoryModal() {
    document.getElementById('directoryForm').reset();
    document.querySelector('#directoryModal .modal-title').textContent = '添加目录';
    
    // Reset editing state
    editingDirectoryId = null;
}

// Global variable to prevent multiple delete operations
let isDeleting = false;

// Prompt delete directory - shows confirmation first
async function promptDeleteDirectory(directoryId, directoryName) {
    // 防止重复删除操作
    if (isDeleting) {
        await ask('删除操作正在进行中，请稍候...', { title: '提示', type: 'info' });
        return;
    }
    
    // 获取目录信息以检查是否存在
    const directory = directories.find(dir => dir.id === directoryId);
    let exists = true;
    if (directory) {
        try {
            exists = await tauriCheckDirectoryExists(directory.path);
        } catch (error) {
            console.warn('检查目录存在性失败:', error);
        }
    }
    
    // 根据目录是否存在显示不同的确认信息
    let confirmMessage, confirmTitle;
    if (exists) {
        confirmMessage = `确定要删除目录 "${directoryName}" 吗？\n\n此操作将删除数据库记录，但不会删除文件系统中的目录文件。`;
        confirmTitle = '确认删除目录';
    } else {
        confirmMessage = `目录 "${directoryName}" 在文件系统中不存在。\n\n确定要清理数据库中的记录吗？`;
        confirmTitle = '确认清理记录';
    }
    
    // 显示确认框
    try {
        const userConfirmed = await ask(
            confirmMessage,
            { 
                title: confirmTitle, 
                type: 'warning' 
            }
        );
        
        if (!userConfirmed) {
            return;
        }
        
        // 用户确认后才执行删除
        await executeDelete(directoryId, directoryName);
    } catch (error) {
        // 如果 Tauri 对话框失败，fallback 到浏览器 confirm
        const userConfirmed = confirm(confirmMessage);
        if (userConfirmed) {
            await executeDelete(directoryId, directoryName);
        }
    }
}

// 执行实际的删除操作
async function executeDelete(directoryId, directoryName) {
    isDeleting = true;
    
    try {
        await tauriDeleteDirectory(directoryId);
        await loadDirectories();
        showSuccess(`目录 "${directoryName}" 删除成功`);
    } catch (error) {
        showError(`删除目录 "${directoryName}" 失败: ` + getErrorMessage(error));
    } finally {
        isDeleting = false;
    }
}


// View config
async function viewConfig(directoryId) {
    try {
        const config = await tauriGetCurrentConfig(directoryId);
        const envConfig = config.env_config;
        const directory = config.directory;
        
        // Build config content HTML
        let configHtml = `
            <div class="mb-4">
                <h6 class="fw-bold text-primary">
                    <i class="fas fa-folder me-2"></i>目录信息
                </h6>
                <div class="card bg-light">
                    <div class="card-body">
                        <div class="row">
                            <div class="col-sm-3"><strong>目录名称:</strong></div>
                            <div class="col-sm-9">${directory.name}</div>
                        </div>
                        <div class="row mt-2">
                            <div class="col-sm-3"><strong>目录路径:</strong></div>
                            <div class="col-sm-9"><code>${directory.path}</code></div>
                        </div>
                        <div class="row mt-2">
                            <div class="col-sm-3"><strong>状态:</strong></div>
                            <div class="col-sm-9">
                                <span class="badge ${directory.is_active ? 'bg-success' : 'bg-secondary'}">
                                    ${directory.is_active ? '当前活跃' : '非活跃'}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;
        
        if (Object.keys(envConfig).length > 0) {
            configHtml += `
                <div class="mb-4">
                    <h6 class="fw-bold text-success">
                        <i class="fas fa-cog me-2"></i>环境配置
                    </h6>
                    <div class="card">
                        <div class="card-body">
                            <div class="table-responsive">
                                <table class="table table-sm table-hover">
                                    <thead class="table-light">
                                        <tr>
                                            <th style="width: 30%">配置项</th>
                                            <th>配置值</th>
                                        </tr>
                                    </thead>
                                    <tbody>
            `;
            
            for (const [key, value] of Object.entries(envConfig)) {
                let displayValue = value;
                let valueClass = '';
                
                // Handle sensitive information display
                if (key.includes('TOKEN') || key.includes('KEY')) {
                    displayValue = value.substring(0, 20) + '...';
                    valueClass = 'text-muted font-monospace';
                } else {
                    valueClass = 'font-monospace';
                }
                
                configHtml += `
                    <tr>
                        <td><strong>${key}</strong></td>
                        <td class="${valueClass}">${displayValue}</td>
                    </tr>
                `;
            }
            
            configHtml += `
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        } else {
            configHtml += `
                <div class="mb-4">
                    <h6 class="fw-bold text-warning">
                        <i class="fas fa-exclamation-triangle me-2"></i>环境配置
                    </h6>
                    <div class="alert alert-warning">
                        <i class="fas fa-info-circle me-2"></i>
                        当前目录暂无环境配置信息
                    </div>
                </div>
            `;
        }
        
        // Add timestamp
        configHtml += `
            <div class="mt-4 pt-3 border-top">
                <small class="text-muted">
                    <i class="fas fa-clock me-1"></i>
                    查看时间: ${new Date().toLocaleString()}
                </small>
            </div>
        `;
        
        // Update modal title and content
        document.querySelector('#configModal .modal-title').innerHTML = 
            `<i class="fas fa-folder-open me-2"></i>${directory.name} - 配置详情`;
        document.getElementById('configContent').innerHTML = configHtml;
        
        // Show modal
        const modal = new bootstrap.Modal(document.getElementById('configModal'));
        modal.show();
        
    } catch (error) {
        showError('获取配置失败: ' + getErrorMessage(error));
    }
}

// Account association management
async function loadAssociationPage() {
    try {
        await loadAssociationAccounts();
        await loadAssociationDirectories();
        await loadClaudeConfigStatusInAssociation();
    } catch (error) {
        showError('加载账号关联页面失败: ' + getErrorMessage(error));
    }
}

// Load association directories
async function loadAssociationDirectories() {
    try {
        associationDirectories = await tauriGetDirectories();
        
        const select = document.getElementById('directorySelect');
        select.innerHTML = '<option value="">请选择目录</option>';
        
        associationDirectories.forEach(directory => {
            const option = document.createElement('option');
            option.value = directory.id;
            option.textContent = `${directory.name} (${directory.path})`;
            if (directory.is_active) {
                option.selected = true;
            }
            select.appendChild(option);
        });
        
        // If there's an active directory, auto-select and load
        const activeDirectory = associationDirectories.find(dir => dir.is_active);
        if (activeDirectory) {
            await onDirectorySelectionChange(activeDirectory.id);
        }
        
    } catch (error) {
        showError('加载目录列表失败: ' + getErrorMessage(error));
    }
}

// Load association accounts
async function loadAssociationAccounts() {
    try {
        const accountsResponse = await tauriGetAccounts({ per_page: 1000 });
        associationAccounts = accountsResponse.accounts || accountsResponse;
        
        const select = document.getElementById('associationAccountSelect');
        select.innerHTML = '<option value="">请选择账号</option>';
        
        associationAccounts.forEach(account => {
            const option = document.createElement('option');
            option.value = account.id;
            option.textContent = `${account.name} (${account.base_url})`;
            select.appendChild(option);
        });
        
        await renderAssociationAccountsList();
        
    } catch (error) {
        showError('加载账号列表失败: ' + getErrorMessage(error));
    }
}

// Directory selection change handler
async function onDirectorySelectionChange(directoryId) {
    if (!directoryId) {
        document.getElementById('selectedDirectoryInfo').classList.add('d-none');
        document.getElementById('associationConfigStatus').innerHTML = '<div class="text-muted">请先选择目录</div>';
        document.getElementById('associationSwitchBtn').disabled = true;
        currentDirectoryForAssociation = null;
        return;
    }
    
    currentDirectoryForAssociation = directoryId;
    
    try {
        // Show directory info
        const directory = associationDirectories.find(dir => dir.id == directoryId);
        if (directory) {
            const infoDiv = document.getElementById('selectedDirectoryInfo');
            infoDiv.innerHTML = `
                <h6 class="alert-heading">
                    <i class="fas fa-folder-open me-2"></i>${directory.name}
                </h6>
                <p class="mb-1"><strong>路径:</strong> <code>${directory.path}</code></p>
                <p class="mb-0"><strong>状态:</strong> 
                    <span class="badge ${directory.is_active ? 'bg-success' : 'bg-secondary'}">
                        ${directory.is_active ? '当前活跃' : '非活跃'}
                    </span>
                </p>
            `;
            infoDiv.classList.remove('d-none');
        }
        
        // Get config status
        const config = await tauriGetCurrentConfig(directoryId);
        const envConfig = config.env_config;
        
        let statusHtml = '';
        if (Object.keys(envConfig).length > 0) {
            const hasToken = envConfig.ANTHROPIC_API_KEY || envConfig.CLAUDE_API_KEY || envConfig.ANTHROPIC_AUTH_TOKEN;
            const hasBaseUrl = envConfig.ANTHROPIC_BASE_URL || envConfig.CLAUDE_BASE_URL;
            
            statusHtml = `
                <div class="alert alert-success">
                    <h6 class="alert-heading">
                        <i class="fas fa-check-circle me-2"></i>配置状态
                    </h6>
                    <p class="mb-1">
                        <strong>API密钥:</strong> 
                        <span class="badge ${hasToken ? 'bg-success' : 'bg-danger'}">
                            ${hasToken ? '已配置' : '未配置'}
                        </span>
                    </p>
                    <p class="mb-0">
                        <strong>Base URL:</strong> 
                        <span class="badge ${hasBaseUrl ? 'bg-success' : 'bg-danger'}">
                            ${hasBaseUrl ? '已配置' : '未配置'}
                        </span>
                    </p>
                    ${hasToken ? `<p class="mb-0 mt-2"><small class="text-muted">Token预览: ${hasToken.toString().substring(0, 20)}...</small></p>` : ''}
                    ${hasBaseUrl ? `<p class="mb-0"><small class="text-muted">URL: ${hasBaseUrl}</small></p>` : ''}
                </div>
            `;
        } else {
            statusHtml = `
                <div class="alert alert-warning">
                    <i class="fas fa-exclamation-triangle me-2"></i>
                    当前目录暂无环境配置
                </div>
            `;
        }
        
        document.getElementById('associationConfigStatus').innerHTML = statusHtml;
        
        // Enable switch button
        updateSwitchButtonState();
        
    } catch (error) {
        const errorMessage = getErrorMessage(error);
        document.getElementById('associationConfigStatus').innerHTML = `
            <div class="alert alert-danger">
                <i class="fas fa-times-circle me-2"></i>
                获取配置状态失败: ${errorMessage}
            </div>
        `;
    }
}

// Render accounts list for association
async function renderAssociationAccountsList() {
    const container = document.getElementById('associationAccountsList');
    
    if (associationAccounts.length === 0) {
        container.innerHTML = '<div class="text-muted">暂无账号</div>';
        return;
    }
    
    try {
        const allAssociations = await tauriGetAssociations();
        
        const accountsWithAssociations = associationAccounts.map(account => {
            const accountAssociations = allAssociations.filter(assoc => assoc.account_id === account.id);
            account.associated_directories = accountAssociations.map(assoc => ({
                id: assoc.directory_id,
                name: assoc.directory_name,
                association_id: assoc.id,
                created_at: assoc.created_at
            }));
            return account;
        });
        
        container.innerHTML = accountsWithAssociations.map(account => `
        <div class="list-group-item ${account.associated_directories.length > 0 ? 'associated' : ''}">
            <div class="d-flex justify-content-between align-items-start">
                <div>
                    <h6 class="mb-1">
                        ${account.associated_directories.length > 0 ? '<span class="association-indicator me-2"></span>' : ''}
                        ${account.name}
                        ${account.is_active ? '<span class="badge bg-success ms-2">当前活跃</span>' : ''}
                    </h6>
                    <p class="mb-1"><small class="text-muted">${account.base_url}</small></p>
                    <small class="text-muted">Token: ${account.token.substring(0, 20)}...</small>
                    ${account.associated_directories.length > 0 ? `
                        <div class="mt-2">
                            <small class="text-info">
                                <i class="fas fa-folder me-1"></i>
                                关联目录: ${account.associated_directories.map(dir => dir.name).join(', ')}
                            </small>
                        </div>
                    ` : ''}
                </div>
                <div>
                    ${account.is_active ? 
                        '<span class="badge bg-primary">活跃中</span>' : 
                        `<button class="btn btn-sm btn-outline-primary" onclick="quickSwitchFromList(${account.id})">
                            <i class="fas fa-sync-alt"></i> 切换
                        </button>`
                    }
                </div>
            </div>
        </div>
    `).join('');
    
    } catch (error) {
        container.innerHTML = '<div class="text-muted text-danger">加载账号关联信息失败</div>';
    }
}

// Quick switch from list
async function quickSwitchFromList(accountId) {
    if (!currentDirectoryForAssociation) {
        showError('请先选择目录');
        return;
    }
    
    await performAccountSwitchInternal(accountId);
}

// Perform account switch
async function performAccountSwitch() {
    const accountId = document.getElementById('associationAccountSelect').value;
    
    if (!accountId) {
        showError('请选择要切换的账号');
        return;
    }
    
    // 默认设置IS_SANDBOX为true
    const isSandbox = true;
    
    await performAccountSwitchInternal(accountId, isSandbox);
}

// Internal account switch function
async function performAccountSwitchInternal(accountId, isSandbox = true) {
    if (!currentDirectoryForAssociation) {
        showError('请先选择目录');
        return;
    }
    
    // 验证选择的目录是否存在
    try {
        const directory = associationDirectories.find(dir => dir.id == currentDirectoryForAssociation);
        if (directory) {
            const pathExists = await tauriCheckDirectoryExists(directory.path);
            if (!pathExists) {
                const confirmed = confirm(`目录 "${directory.name}" (${directory.path}) 在文件系统中不存在。\n\n确定要继续切换账号吗？\n\n注意：这可能会导致配置写入失败。`);
                if (!confirmed) {
                    return;
                }
            }
        }
    } catch (error) {
        console.warn('无法验证目录存在性:', error);
        // 继续执行，不阻止操作
    }
    
    try {
        // 获取Claude配置
        const claudeSettings = await getClaudeSettingsForSwitch();
        
        const result = await tauriSwitchAccountWithClaudeSettings(
            parseInt(accountId), 
            parseInt(currentDirectoryForAssociation),
            isSandbox,
            claudeSettings
        );
        
        showSuccess(result);
        
        // Reload data
        await loadAssociationAccounts();
        await onDirectorySelectionChange(currentDirectoryForAssociation);
        
        // Reset selector
        document.getElementById('associationAccountSelect').value = '';
        
    } catch (error) {
        showError('切换账号失败: ' + getErrorMessage(error));
    }
}

// Update switch button state
function updateSwitchButtonState() {
    const accountId = document.getElementById('associationAccountSelect').value;
    const directoryId = currentDirectoryForAssociation;
    const btn = document.getElementById('associationSwitchBtn');
    
    btn.disabled = !accountId || !directoryId;
}

// Database management functions
async function loadDatabaseInfo() {
    try {
        const [info, connections] = await Promise.all([
            tauriGetDatabaseInfo(),
            tauriGetDatabaseConnections()
        ]);
        
        // 显示当前连接信息
        displayDatabaseInfo(info);
        
        // 更新连接选择器
        const select = document.getElementById('dbConnectionSelect');
        if (select) {
            // 保存当前选择的值
            const currentSelection = select.value;
            
            select.innerHTML = '<option value="">选择数据库连接</option>';
            
            for (const [name, config] of Object.entries(connections.connections)) {
                const option = document.createElement('option');
                option.value = name;
                option.textContent = `${name} - ${config.url.split('://')[0].toUpperCase()}`;
                if (name === connections.current) {
                    option.selected = true;
                }
                select.appendChild(option);
            }
            
            // 如果之前有选择且不是当前连接，恢复选择
            if (currentSelection && currentSelection !== connections.current) {
                select.value = currentSelection;
                // 显示选择的连接信息预览
                await previewDatabaseConnection(currentSelection);
            }
            
            // 添加选择变化事件监听器
            select.removeEventListener('change', onDatabaseSelectionChange);
            select.addEventListener('change', onDatabaseSelectionChange);
        }
    } catch (error) {
        const currentDbInfo = document.getElementById('currentDbInfo');
        currentDbInfo.innerHTML = '加载数据库信息失败: ' + getErrorMessage(error);
        currentDbInfo.className = 'alert alert-danger';
    }
}

// 显示数据库信息
function displayDatabaseInfo(info) {
    const currentDbInfo = document.getElementById('currentDbInfo');
    if (info.name) {
        currentDbInfo.innerHTML = `
            <strong>连接名称:</strong> ${info.name}<br>
            <strong>数据库URL:</strong> <code>${info.url}</code><br>
            <strong>连接池大小:</strong> ${info.pool_size}<br>
            <strong>已签出连接:</strong> ${info.checked_out}<br>
            <strong>已签入连接:</strong> ${info.checked_in}
        `;
        currentDbInfo.className = 'alert alert-success';
    } else {
        currentDbInfo.innerHTML = '无连接信息';
        currentDbInfo.className = 'alert alert-warning';
    }
}

// 数据库选择变化事件处理
async function onDatabaseSelectionChange(event) {
    const connectionName = event.target.value;
    
    // 重置切换按钮状态
    const switchBtn = document.getElementById('switchDbBtn');
    switchBtn.disabled = true;
    switchBtn.className = 'btn btn-outline-secondary';
    
    if (!connectionName) {
        // 如果没有选择，显示当前活跃连接信息
        await loadDatabaseInfo();
        return;
    }
    
    // 预览选择的连接信息
    await previewDatabaseConnection(connectionName);
}

// 预览数据库连接信息
async function previewDatabaseConnection(connectionName) {
    try {
        const connections = await tauriGetDatabaseConnections();
        const config = connections.connections[connectionName];
        
        if (!config) {
            showDbMessage('选择的数据库连接不存在', 'error');
            return;
        }
        
        // 显示预览信息
        const currentDbInfo = document.getElementById('currentDbInfo');
        const isCurrentConnection = connectionName === connections.current;
        
        currentDbInfo.innerHTML = `
            <div class="d-flex justify-content-between align-items-start mb-2">
                <strong>连接名称:</strong> 
                <span>
                    ${connectionName}
                    ${isCurrentConnection ? '<span class="badge bg-success ms-2">当前活跃</span>' : '<span class="badge bg-secondary ms-2">预览</span>'}
                </span>
            </div>
            <strong>数据库URL:</strong> <code>${config.url}</code><br>
            <strong>连接池大小:</strong> ${config.pool_size}<br>
            ${!isCurrentConnection ? '<div class="mt-2"><small class="text-muted"><i class="fas fa-info-circle me-1"></i>这是预览信息，点击"切换连接"按钮来激活此连接</small></div>' : ''}
        `;
        currentDbInfo.className = isCurrentConnection ? 'alert alert-success' : 'alert alert-info';
        
    } catch (error) {
        showDbMessage('预览连接信息失败: ' + getErrorMessage(error), 'error');
    }
}

async function switchDatabase() {
    const connectionName = document.getElementById('dbConnectionSelect').value;
    if (!connectionName) {
        showDbMessage('请选择数据库连接', 'error');
        return;
    }
    
    // 检查是否已经测试过该连接且测试成功
    if (lastTestedConnection !== connectionName || lastTestResult !== true) {
        showDbMessage('请先测试数据库连接成功后再切换', 'warning');
        return;
    }
    
    try {
        const result = await tauriSwitchDatabase(connectionName);
        showDbMessage(result, 'success');
        
        // 刷新数据库信息以显示当前连接状态
        await loadDatabaseInfo();
        
        // 重新加载数据以确保使用新连接（仅在实际切换时需要）
        if (connectionName === 'sqlite') {
            await loadAccounts();
            await loadDirectories();
        }
    } catch (error) {
        showDbMessage('切换数据库失败: ' + getErrorMessage(error), 'error');
    }
}

async function testDatabase() {
    const connectionName = document.getElementById('dbConnectionSelect').value;
    if (!connectionName) {
        showDbMessage('请选择数据库连接', 'error');
        return;
    }
    
    // 获取测试按钮并显示加载状态
    const testBtn = document.getElementById('testDbBtn');
    const originalText = testBtn.innerHTML;
    
    // 设置加载状态
    testBtn.disabled = true;
    testBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>测试中...';
    
    // 显示测试开始消息
    showDbMessage('正在测试数据库连接，请稍候...', 'info');
    
    try {
        const result = await tauriTestDatabaseConnection(connectionName);
        
        // 记录测试成功状态
        lastTestedConnection = connectionName;
        lastTestResult = true;
        
        // 启用切换按钮
        const switchBtn = document.getElementById('switchDbBtn');
        switchBtn.disabled = false;
        switchBtn.className = 'btn btn-success';
        
        showDbMessage(result, 'success');
        
    } catch (error) {
        // 记录测试失败状态
        lastTestedConnection = connectionName;
        lastTestResult = false;
        
        // 使用标准化的错误消息处理
        const errorMessage = getErrorMessage(error);
        
        // 如果错误信息已经包含"测试失败"，直接使用
        if (errorMessage.includes('测试失败')) {
            showDbMessage(errorMessage, 'error');
        } else {
            showDbMessage('数据库连接测试失败: ' + errorMessage, 'error');
        }
        
    } finally {
        // 恢复按钮状态
        testBtn.disabled = false;
        testBtn.innerHTML = originalText;
    }
}

function showDbMessage(message, type) {
    const container = document.getElementById('dbSwitchStatus');
    if (!container) return;
    
    let alertClass = 'alert-info';
    let icon = 'fas fa-info-circle';
    
    if (type === 'success') {
        alertClass = 'alert-success';
        icon = 'fas fa-check-circle';
    } else if (type === 'error') {
        alertClass = 'alert-danger';
        icon = 'fas fa-exclamation-circle';
    } else if (type === 'warning') {
        alertClass = 'alert-warning';
        icon = 'fas fa-exclamation-triangle';
    }
    
    container.innerHTML = `
        <div class="${alertClass} alert alert-dismissible">
            <i class="${icon} me-2"></i>${message}
            ${type === 'error' || type === 'warning' ? '<button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="关闭"></button>' : ''}
        </div>
    `;
    
    // 只有成功消息自动清除，错误和警告消息保持显示让用户手动关闭
    // info类型消息（如加载中）不自动清除，等待手动更新
    if (type === 'success') {
        setTimeout(() => {
            container.innerHTML = '';
        }, 3000);
    }
}

// URL management functions
async function loadBaseUrls() {
    try {
        baseUrls = await tauriGetBaseUrls();
        renderBaseUrls();
        updateBaseUrlSelect();
    } catch (error) {
        showError('加载URL失败: ' + getErrorMessage(error));
    }
}

// Render base URLs list
function renderBaseUrls() {
    const container = document.getElementById('urlsList');
    
    if (baseUrls.length === 0) {
        container.innerHTML = '<div class="text-muted">暂无URL</div>';
        return;
    }
    
    container.innerHTML = baseUrls.map(url => `
        <div class="list-group-item">
            <div class="url-item">
                <div class="url-info">
                    <div class="fw-bold">
                        ${url.is_default ? '<span class="url-default-indicator"></span>' : ''}${url.name} ${url.is_default ? '<span class="badge bg-primary">默认</span>' : ''}
                    </div>
                    <div class="small text-muted">${url.url}</div>
                    ${url.description ? `<div class="small">${url.description}</div>` : ''}
                </div>
                <div class="url-actions">
                    <button class="btn btn-sm btn-outline-primary" onclick="editBaseUrl(${url.id})">编辑</button>
                    <button class="btn btn-sm btn-outline-danger" onclick="promptDeleteBaseUrl(${url.id})">删除</button>
                </div>
            </div>
        </div>
    `).join('');
}

// Update base URL select in account form
function updateBaseUrlSelect() {
    const select = document.getElementById('accountBaseUrlSelect');
    if (select) {
        select.innerHTML = '<option value="">选择预设URL</option>' +
            baseUrls.map(url => `<option value="${url.url}">${url.name}</option>`).join('');
        
        // Auto-select default URL
        const defaultUrl = baseUrls.find(url => url.is_default);
        if (defaultUrl) {
            select.value = defaultUrl.url;
            document.getElementById('accountBaseUrl').value = defaultUrl.url;
        }
    }
}

// Save base URL
async function saveBaseUrl() {
    const name = document.getElementById('urlName').value.trim();
    const url = document.getElementById('urlAddress').value.trim();
    const description = document.getElementById('urlDescription').value;
    const isDefault = document.getElementById('urlIsDefault').checked;
    
    if (!name || !url) {
        showError('请填写所有必需字段');
        return;
    }
    
    // 验证URL格式
    try {
        new URL(url);
    } catch (e) {
        showError('请输入有效的URL格式（例如：https://api.example.com）');
        return;
    }
    
    // 确保URL以http或https开头
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
        showError('URL必须以http://或https://开头');
        return;
    }
    
    try {
        await tauriCreateBaseUrl(name, url, description, isDefault);
        
        // Close modal and reset form
        const modal = bootstrap.Modal.getInstance(document.getElementById('urlModal'));
        modal.hide();
        document.getElementById('urlForm').reset();
        
        // Reload URL list
        await loadBaseUrls();
        showSuccess('URL添加成功');
    } catch (error) {
        // 处理特定的数据库错误
        let errorMessage = getErrorMessage(error);
        if (errorMessage.includes('UNIQUE constraint failed: base_urls.name')) {
            errorMessage = 'URL名称已存在，请使用不同的名称';
        } else if (errorMessage.includes('UNIQUE constraint failed: base_urls.url')) {
            errorMessage = 'URL地址已存在，请使用不同的URL地址';
        }
        
        showError('添加URL失败: ' + errorMessage);
    }
}

// Edit base URL
async function editBaseUrl(urlId) {
    try {
        const url = baseUrls.find(u => u.id === urlId);
        if (!url) {
            showError('找不到URL信息');
            return;
        }
        
        // Fill form
        document.getElementById('urlName').value = url.name;
        document.getElementById('urlAddress').value = url.url;
        document.getElementById('urlDescription').value = url.description || '';
        document.getElementById('urlIsDefault').checked = url.is_default;
        
        // Change modal title
        document.querySelector('#urlModal .modal-title').textContent = '编辑URL';
        
        // Change save button behavior
        const saveBtn = document.getElementById('saveUrl');
        // 移除所有现有的事件监听器
        saveBtn.removeEventListener('click', saveBaseUrl);
        if (saveBtn._updateHandler) {
            saveBtn.removeEventListener('click', saveBtn._updateHandler);
        }
        // 创建新的更新URL处理函数
        const updateUrlHandler = async function() {
            await updateBaseUrl(urlId);
        };
        saveBtn.addEventListener('click', updateUrlHandler);
        // 保存处理函数引用，以便后续清理
        saveBtn._updateHandler = updateUrlHandler;
        saveBtn.onclick = null; // 清除onclick属性
        
        // Show modal
        const modal = new bootstrap.Modal(document.getElementById('urlModal'));
        modal.show();
        
    } catch (error) {
        showError('编辑URL失败: ' + getErrorMessage(error));
    }
}

// Update base URL
async function updateBaseUrl(urlId) {
    const name = document.getElementById('urlName').value.trim();
    const url = document.getElementById('urlAddress').value.trim();
    const description = document.getElementById('urlDescription').value;
    const isDefault = document.getElementById('urlIsDefault').checked;
    
    if (!name || !url) {
        showError('请填写所有必需字段');
        return;
    }
    
    // 验证URL格式
    try {
        new URL(url);
    } catch (e) {
        showError('请输入有效的URL格式（例如：https://api.example.com）');
        return;
    }
    
    // 确保URL以http或https开头
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
        showError('URL必须以http://或https://开头');
        return;
    }
    
    try {
        await tauriUpdateBaseUrl(urlId, { name, url, description, is_default: isDefault });
        
        // Close modal and reset form
        const modal = bootstrap.Modal.getInstance(document.getElementById('urlModal'));
        modal.hide();
        
        // Reset form and modal state
        resetUrlModal();
        
        // Reload URL list
        await loadBaseUrls();
        showSuccess('URL更新成功');
    } catch (error) {
        // 处理特定的数据库错误
        let errorMessage = getErrorMessage(error);
        if (errorMessage.includes('UNIQUE constraint failed: base_urls.name')) {
            errorMessage = 'URL名称已存在，请使用不同的名称';
        } else if (errorMessage.includes('UNIQUE constraint failed: base_urls.url')) {
            errorMessage = 'URL地址已存在，请使用不同的URL地址';
        }
        
        showError('更新URL失败: ' + errorMessage);
    }
}

// Reset URL modal state
function resetUrlModal() {
    document.getElementById('urlForm').reset();
    document.querySelector('#urlModal .modal-title').textContent = '添加URL';
    
    // Reset save button behavior
    const saveBtn = document.getElementById('saveUrl');
    // 移除可能存在的更新URL处理函数
    if (saveBtn._updateHandler) {
        saveBtn.removeEventListener('click', saveBtn._updateHandler);
        saveBtn._updateHandler = null;
    }
    // 确保添加URL处理函数存在
    saveBtn.removeEventListener('click', saveBaseUrl);
    saveBtn.addEventListener('click', saveBaseUrl);
    saveBtn.onclick = null; // 清除onclick属性
}

// Prompt delete base URL - shows confirmation first
async function promptDeleteBaseUrl(urlId) {
    // 找到要删除的URL信息
    const url = baseUrls.find(u => u.id === urlId);
    const urlName = url ? url.name : `ID: ${urlId}`;
    
    // 显示确认框
    try {
        const userConfirmed = await ask(
            `确定要删除URL "${urlName}" 吗？\n\n此操作不可撤销！`,
            { 
                title: '确认删除URL', 
                type: 'warning' 
            }
        );
        
        if (!userConfirmed) {
            return;
        }
        
        // 用户确认后才执行删除
        await executeDeleteBaseUrl(urlId, urlName);
    } catch (error) {
        // 如果 Tauri 对话框失败，fallback 到浏览器 confirm
        const userConfirmed = confirm(`确定要删除URL "${urlName}" 吗？\n\n此操作不可撤销！`);
        if (userConfirmed) {
            await executeDeleteBaseUrl(urlId, urlName);
        }
    }
}

// 执行实际的URL删除操作
async function executeDeleteBaseUrl(urlId, urlName) {
    try {
        await tauriDeleteBaseUrl(urlId);
        await loadBaseUrls();
        showSuccess(`URL "${urlName}" 删除成功`);
    } catch (error) {
        showError(`删除URL "${urlName}" 失败: ` + getErrorMessage(error));
    }
}

// Delete base URL (legacy function for compatibility)
async function deleteBaseUrl(urlId) {
    if (!confirm('确定要删除这个URL吗？')) {
        return;
    }
    
    try {
        await tauriDeleteBaseUrl(urlId);
        await loadBaseUrls();
        showSuccess('URL删除成功');
    } catch (error) {
        showError('删除URL失败: ' + getErrorMessage(error));
    }
}

// Global message display functions
function showGlobalMessage(message, type = 'error', duration = 3000) {
    const container = document.getElementById('globalMessageContainer');
    if (!container) {
        return;
    }
    
    const messageId = 'msg-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
    
    // Determine Bootstrap alert type
    let alertClass = 'alert-danger';
    let icon = 'fas fa-exclamation-circle';
    
    if (type === 'success') {
        alertClass = 'alert-success';
        icon = 'fas fa-check-circle';
    } else if (type === 'warning') {
        alertClass = 'alert-warning';
        icon = 'fas fa-exclamation-triangle';
    } else if (type === 'info') {
        alertClass = 'alert-info';
        icon = 'fas fa-info-circle';
    }
    
    // Create message element
    const messageElement = document.createElement('div');
    messageElement.id = messageId;
    messageElement.className = `alert ${alertClass} alert-dismissible fade show mb-2`;
    messageElement.style.cssText = 'animation: slideInRight 0.3s ease-out;';
    messageElement.innerHTML = `
        <i class="${icon} me-2"></i>
        <span>${message}</span>
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="关闭"></button>
    `;
    
    // Add to container top
    container.insertBefore(messageElement, container.firstChild);
    
    // Auto remove function
    const removeMessage = () => {
        const element = document.getElementById(messageId);
        if (element) {
            element.style.animation = 'slideOutRight 0.3s ease-in forwards';
            
            setTimeout(() => {
                if (element && element.parentNode) {
                    element.parentNode.removeChild(element);
                }
            }, 300);
        }
    };
    
    // Auto-disappear timer
    if (duration > 0) {
        setTimeout(removeMessage, duration);
    }
    
    // Manual close button event
    const closeBtn = messageElement.querySelector('.btn-close');
    if (closeBtn) {
        closeBtn.addEventListener('click', removeMessage);
    }
}

// Show success message
function showSuccess(message) {
    showGlobalMessage(message, 'success', 3000);
}

// Show error message
function showError(message) {
    showGlobalMessage(message, 'error', 0); // 设置duration为0，错误消息不自动关闭
}

// Tab management functions
function saveActiveTab(tabId) {
    localStorage.setItem('activeTab', tabId);
}

function getActiveTab() {
    return localStorage.getItem('activeTab') || 'accounts-pane';
}

function activateTab(tabId) {
    // Remove all active states
    document.querySelectorAll('#mainTabs .nav-link').forEach(link => {
        link.classList.remove('active');
        link.setAttribute('aria-selected', 'false');
    });
    
    document.querySelectorAll('.tab-pane').forEach(pane => {
        pane.classList.remove('show', 'active');
    });
    
    // Activate specified tab
    const tabButton = document.querySelector(`#mainTabs button[data-bs-target="#${tabId}"]`);
    const tabPane = document.querySelector(`#${tabId}`);
    
    if (tabButton && tabPane) {
        tabButton.classList.add('active');
        tabButton.setAttribute('aria-selected', 'true');
        tabPane.classList.add('show', 'active');
    }
}

// Event listeners
window.addEventListener("DOMContentLoaded", () => {
    // Bind save button events
    document.getElementById('saveAccount').addEventListener('click', saveAccount);
    document.getElementById('saveDirectory').addEventListener('click', saveDirectory);
    // URL按钮事件在resetUrlModal中绑定，避免重复绑定
    
    // URL selector change event
    const urlSelect = document.getElementById('accountBaseUrlSelect');
    if (urlSelect) {
        urlSelect.addEventListener('change', function() {
            if (this.value) {
                document.getElementById('accountBaseUrl').value = this.value;
            }
        });
    }
    
    // Modal close event listeners to reset state
    const accountModal = document.getElementById('accountModal');
    accountModal.addEventListener('hidden.bs.modal', function () {
        resetAccountModal();
    });
    
    const directoryModal = document.getElementById('directoryModal');
    directoryModal.addEventListener('hidden.bs.modal', function () {
        resetDirectoryModal();
    });
    
    const urlModal = document.getElementById('urlModal');
    urlModal.addEventListener('hidden.bs.modal', function () {
        resetUrlModal();
    });
    
    // Initialize modal states and data loading
    resetDirectoryModal();
    resetUrlModal();
    loadBaseUrls();
    
    // Set up account filters and search
    setupAccountFilters();
    
    // Restore last active tab
    const lastActiveTab = getActiveTab();
    activateTab(lastActiveTab);
    
    // Load data based on active tab
    if (lastActiveTab === 'accounts-pane') {
        loadAccounts();
        loadAccountBaseUrlOptions();
    } else if (lastActiveTab === 'directories-pane') {
        loadDirectories();
    } else if (lastActiveTab === 'association-pane') {
        loadAssociationPage();
    } else if (lastActiveTab === 'urls-pane') {
        loadBaseUrls();
    } else if (lastActiveTab === 'database-pane') {
        loadDatabaseInfo();
    } else if (lastActiveTab === 'claude-settings-pane') {
        loadClaudeSettingsPage();
    }
    
    // Tab switch event listeners
    const tabTriggerList = document.querySelectorAll('#mainTabs button[data-bs-toggle="tab"]');
    tabTriggerList.forEach(function (tabTrigger) {
        tabTrigger.addEventListener('click', function (event) {
            const target = event.target.getAttribute('data-bs-target');
            const tabId = target.substring(1); // Remove # sign
            saveActiveTab(tabId);
        });
        
        tabTrigger.addEventListener('shown.bs.tab', function (event) {
            const target = event.target.getAttribute('data-bs-target');
            if (target === '#database-pane') {
                loadDatabaseInfo();
            } else if (target === '#directories-pane') {
                loadDirectories();
            } else if (target === '#association-pane') {
                loadAssociationPage();
            } else if (target === '#urls-pane') {
                loadBaseUrls();
            } else if (target === '#accounts-pane') {
                loadAccounts();
                loadAccountBaseUrlOptions();
            } else if (target === '#claude-settings-pane') {
                loadClaudeSettingsPage();
            }
        });
    });
    
    // Set up association page event listeners
    setupAssociationEventListeners();
    
    // Bind directory selection button
    const selectDirBtn = document.getElementById('selectDirectoryBtn');
    if (selectDirBtn) {
        selectDirBtn.addEventListener('click', selectDirectory);
    }
});

// Set up association page event listeners
function setupAssociationEventListeners() {
    // Directory selection change
    const directorySelect = document.getElementById('directorySelect');
    if (directorySelect) {
        directorySelect.addEventListener('change', function() {
            onDirectorySelectionChange(this.value);
        });
    }
    
    // Account selection change
    const accountSelect = document.getElementById('associationAccountSelect');
    if (accountSelect) {
        accountSelect.addEventListener('change', function() {
            updateSwitchButtonState();
        });
    }
}

// Directory selection function
async function selectDirectory() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: '选择目录'
        });
        
        if (selected) {
            document.getElementById('directoryPath').value = selected;
            
            // Auto-generate directory name if empty
            const nameInput = document.getElementById('directoryName');
            if (!nameInput.value.trim()) {
                const pathParts = selected.split(/[/\\]/);
                const folderName = pathParts[pathParts.length - 1] || pathParts[pathParts.length - 2];
                if (folderName) {
                    nameInput.value = folderName;
                }
            }
        }
    } catch (error) {
        showError('选择目录失败: ' + getErrorMessage(error));
    }
}

// Claude Settings Management
let claudeSettingsData = {
    permissions: {
        defaultMode: 'bypassPermissions',
        allow: ['*'],
        deny: []
    },
    env: {
        IS_SANDBOX: '1',
        DISABLE_AUTOUPDATER: 1
    }
};

// 标志位，防止重复绑定事件监听器
let claudeSettingsEventListenersSetup = false;

// Available Claude tools
const availableClaudeTools = [
    'Read', 'Write', 'Edit', 'MultiEdit', 'Bash', 'Glob', 'Grep', 
    'WebSearch', 'WebFetch', 'Task', 'TodoWrite', 'BashOutput', 
    'KillBash', 'NotebookEdit', 'ExitPlanMode'
];

// Load Claude settings page
async function loadClaudeSettingsPage() {
    try {
        // 加载现有的配置文件
        const loadResult = await loadCurrentClaudeSettings();

        // 如果加载失败，显示详细警告信息
        if (loadResult !== true) {
            if (loadResult && loadResult.error) {
                showClaudeSettingsMessage(
                    '从数据库加载配置失败，正在使用默认配置。错误: ' + loadResult.error,
                    'warning'
                );
            } else {
                showClaudeSettingsMessage('从数据库加载配置失败，正在使用默认配置', 'warning');
            }
        }

        // 初始化工具列表
        renderToolsList();

        // 初始化环境变量列表
        renderCustomEnvVars();

        // 更新预览
        updatePreview();

        // 绑定事件监听器
        setupClaudeSettingsEventListeners();

    } catch (error) {
        showClaudeSettingsMessage('加载Claude配置失败: ' + getErrorMessage(error), 'error');
    }
}

// 加载当前的Claude配置
async function loadCurrentClaudeSettings() {
    try {
        const settings = await invoke('get_claude_settings_from_db');

        console.log('从数据库加载的Claude配置:', settings);

        // 验证数据结构完整性
        if (!settings || typeof settings !== 'object') {
            throw new Error('数据库返回的配置格式无效');
        }

        // 确保 permissions 对象存在
        if (!settings.permissions) {
            settings.permissions = {
                defaultMode: 'bypassPermissions',
                allow: ['*'],
                deny: []
            };
        }

        // 确保 permissions.allow 是数组
        if (!Array.isArray(settings.permissions.allow)) {
            settings.permissions.allow = ['*'];
        }

        // 确保 permissions.deny 是数组
        if (!Array.isArray(settings.permissions.deny)) {
            settings.permissions.deny = [];
        }

        // 确保 env 对象存在
        if (!settings.env) {
            settings.env = {
                IS_SANDBOX: '1',
                DISABLE_AUTOUPDATER: 1
            };
        }

        claudeSettingsData = settings;

        console.log('验证后的Claude配置:', claudeSettingsData);

        // 更新UI
        document.getElementById('defaultPermissionMode').value =
            claudeSettingsData.permissions.defaultMode || 'bypassPermissions';

        // 更新工具选择状态
        updateToolsSelection();

        // 更新环境变量开关
        document.getElementById('sandboxMode').checked =
            claudeSettingsData.env.IS_SANDBOX === '1' || claudeSettingsData.env.IS_SANDBOX === 1;
        document.getElementById('disableAutoUpdater').checked =
            claudeSettingsData.env.DISABLE_AUTOUPDATER === 1 || claudeSettingsData.env.DISABLE_AUTOUPDATER === '1';
        document.getElementById('disablePromptCaching').checked =
            claudeSettingsData.env.DISABLE_PROMPT_CACHING === 1 || claudeSettingsData.env.DISABLE_PROMPT_CACHING === '1';

        // 更新文本型环境变量
        document.getElementById('anthropicModel').value = claudeSettingsData.env.ANTHROPIC_MODEL || '';
        document.getElementById('smallFastModel').value = claudeSettingsData.env.ANTHROPIC_SMALL_FAST_MODEL || '';

        // 更新数值型环境变量
        document.getElementById('maxOutputTokens').value = claudeSettingsData.env.CLAUDE_CODE_MAX_OUTPUT_TOKENS || '';
        document.getElementById('maxThinkingTokens').value = claudeSettingsData.env.MAX_THINKING_TOKENS || '';
        document.getElementById('maxMcpOutputTokens').value = claudeSettingsData.env.MAX_MCP_OUTPUT_TOKENS || '';
        document.getElementById('bashTimeout').value = claudeSettingsData.env.BASH_DEFAULT_TIMEOUT_MS || '';
        document.getElementById('mcpTimeout').value = claudeSettingsData.env.MCP_TIMEOUT || '';

        return true; // 加载成功

    } catch (error) {
        // 显示详细错误信息
        const errorMsg = getErrorMessage(error);
        console.error('加载Claude配置失败，使用默认配置:', error);
        console.error('详细错误:', errorMsg);

        // 如果数据库中没有配置，使用默认设置
        claudeSettingsData = {
            permissions: {
                defaultMode: 'bypassPermissions',
                allow: ['*'],
                deny: []
            },
            env: {
                IS_SANDBOX: '1',
                DISABLE_AUTOUPDATER: 1
            }
        };

        // 更新UI为默认值
        document.getElementById('defaultPermissionMode').value = 'bypassPermissions';
        updateToolsSelection();
        document.getElementById('sandboxMode').checked = true;
        document.getElementById('disableAutoUpdater').checked = true;

        // 返回错误信息以便显示
        return { success: false, error: errorMsg };
    }
}

// 渲染工具列表
function renderToolsList() {
    const container = document.getElementById('toolsList');

    // 确保数据结构存在
    if (!claudeSettingsData.permissions || !Array.isArray(claudeSettingsData.permissions.allow)) {
        console.warn('Claude配置数据不完整，使用默认值');
        claudeSettingsData.permissions = {
            defaultMode: 'bypassPermissions',
            allow: ['*'],
            deny: []
        };
    }

    const allowAll = claudeSettingsData.permissions.allow.includes('*');

    document.getElementById('allowAllTools').checked = allowAll;

    if (allowAll) {
        container.innerHTML = '<div class="text-muted">已允许所有工具 (*)</div>';
    } else {
        container.innerHTML = availableClaudeTools.map(tool => `
            <div class="form-check">
                <input class="form-check-input tool-checkbox" type="checkbox"
                       id="tool-${tool}" value="${tool}"
                       ${claudeSettingsData.permissions.allow.includes(tool) ? 'checked' : ''}>
                <label class="form-check-label" for="tool-${tool}">
                    ${tool}
                </label>
            </div>
        `).join('');
    }

    // 渲染禁用工具列表
    renderDeniedTools();
}

// 渲染禁用工具列表
function renderDeniedTools() {
    const container = document.getElementById('deniedToolsList');

    // 确保数据结构存在
    if (!claudeSettingsData.permissions || !Array.isArray(claudeSettingsData.permissions.deny)) {
        console.warn('Claude配置的deny数据不完整，初始化为空数组');
        if (!claudeSettingsData.permissions) {
            claudeSettingsData.permissions = {
                defaultMode: 'bypassPermissions',
                allow: ['*'],
                deny: []
            };
        } else {
            claudeSettingsData.permissions.deny = [];
        }
    }

    if (claudeSettingsData.permissions.deny.length === 0) {
        container.innerHTML = '<div class="text-muted small">暂无禁用的工具</div>';
    } else {
        container.innerHTML = claudeSettingsData.permissions.deny.map(tool => `
            <span class="badge bg-danger me-2 mb-2">
                ${tool}
                <button type="button" class="btn-close btn-close-white ms-1"
                        onclick="removeDeniedTool('${tool}')" style="font-size: 0.7em;"></button>
            </span>
        `).join('');
    }
}

// 更新工具选择状态
function updateToolsSelection() {
    // 确保数据结构存在
    if (!claudeSettingsData.permissions || !Array.isArray(claudeSettingsData.permissions.allow)) {
        console.warn('updateToolsSelection: Claude配置数据不完整，使用默认值');
        claudeSettingsData.permissions = {
            defaultMode: 'bypassPermissions',
            allow: ['*'],
            deny: []
        };
    }

    const allowAll = claudeSettingsData.permissions.allow.includes('*');
    document.getElementById('allowAllTools').checked = allowAll;
    renderToolsList();
}

// 渲染自定义环境变量
function renderCustomEnvVars() {
    const container = document.getElementById('customEnvList');

    // 确保 env 对象存在
    if (!claudeSettingsData.env || typeof claudeSettingsData.env !== 'object') {
        console.warn('Claude配置的env数据不完整，初始化为默认值');
        claudeSettingsData.env = {
            IS_SANDBOX: '1',
            DISABLE_AUTOUPDATER: 1
        };
    }

    // 过滤掉系统管理的环境变量
    const systemManagedEnvVars = [
        'IS_SANDBOX',
        'DISABLE_AUTOUPDATER',
        'DISABLE_PROMPT_CACHING',
        'ANTHROPIC_MODEL',
        'ANTHROPIC_SMALL_FAST_MODEL',
        'CLAUDE_CODE_MAX_OUTPUT_TOKENS',
        'MAX_THINKING_TOKENS',
        'MAX_MCP_OUTPUT_TOKENS',
        'BASH_DEFAULT_TIMEOUT_MS',
        'MCP_TIMEOUT'
    ];

    const customEnvVars = Object.entries(claudeSettingsData.env)
        .filter(([key]) => !systemManagedEnvVars.includes(key));

    if (customEnvVars.length === 0) {
        container.innerHTML = '<div class="text-muted small">暂无自定义环境变量</div>';
    } else {
        container.innerHTML = customEnvVars.map(([key, value]) => `
            <div class="d-flex align-items-center mb-2 p-2 bg-light rounded">
                <code class="flex-grow-1">${key} = ${value}</code>
                <button class="btn btn-sm btn-outline-danger ms-2"
                        onclick="removeCustomEnvVar('${key}')">
                    <i class="fas fa-times"></i>
                </button>
            </div>
        `).join('');
    }
}

// 设置事件监听器
function setupClaudeSettingsEventListeners() {
    // 如果已经设置过，则不重复设置
    if (claudeSettingsEventListenersSetup) {
        console.log('Claude配置事件监听器已设置，跳过重复绑定');
        return;
    }

    console.log('设置Claude配置事件监听器');

    // 权限模式变更
    document.getElementById('defaultPermissionMode').addEventListener('change', function() {
        claudeSettingsData.permissions.defaultMode = this.value;
        updatePreview();
    });

    // 全选工具变更
    document.getElementById('allowAllTools').addEventListener('change', function() {
        if (this.checked) {
            claudeSettingsData.permissions.allow = ['*'];
        } else {
            claudeSettingsData.permissions.allow = [];
        }
        updateToolsSelection();
        updatePreview();
    });

    // 工具选择变更
    document.addEventListener('change', function(e) {
        if (e.target.classList.contains('tool-checkbox')) {
            const tool = e.target.value;
            if (e.target.checked) {
                if (!claudeSettingsData.permissions.allow.includes(tool)) {
                    claudeSettingsData.permissions.allow.push(tool);
                }
            } else {
                claudeSettingsData.permissions.allow =
                    claudeSettingsData.permissions.allow.filter(t => t !== tool);
            }
            updatePreview();
        }
    });

    // 沙盒模式变更
    document.getElementById('sandboxMode').addEventListener('change', function() {
        claudeSettingsData.env.IS_SANDBOX = this.checked ? '1' : '0';
        updatePreview();
    });

    // 自动更新变更
    document.getElementById('disableAutoUpdater').addEventListener('change', function() {
        claudeSettingsData.env.DISABLE_AUTOUPDATER = this.checked ? 1 : 0;
        updatePreview();
    });

    // 禁用提示缓存变更
    document.getElementById('disablePromptCaching').addEventListener('change', function() {
        claudeSettingsData.env.DISABLE_PROMPT_CACHING = this.checked ? 1 : 0;
        updatePreview();
    });

    // 指定模型变更
    document.getElementById('anthropicModel').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.ANTHROPIC_MODEL = value;
        } else {
            delete claudeSettingsData.env.ANTHROPIC_MODEL;
        }
        updatePreview();
    });

    // 快速模型变更
    document.getElementById('smallFastModel').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.ANTHROPIC_SMALL_FAST_MODEL = value;
        } else {
            delete claudeSettingsData.env.ANTHROPIC_SMALL_FAST_MODEL;
        }
        updatePreview();
    });

    // 最大输出Token数变更
    document.getElementById('maxOutputTokens').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.CLAUDE_CODE_MAX_OUTPUT_TOKENS = parseInt(value);
        } else {
            delete claudeSettingsData.env.CLAUDE_CODE_MAX_OUTPUT_TOKENS;
        }
        updatePreview();
    });

    // 最大思考Token数变更
    document.getElementById('maxThinkingTokens').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.MAX_THINKING_TOKENS = parseInt(value);
        } else {
            delete claudeSettingsData.env.MAX_THINKING_TOKENS;
        }
        updatePreview();
    });

    // MCP输出Token限制变更
    document.getElementById('maxMcpOutputTokens').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.MAX_MCP_OUTPUT_TOKENS = parseInt(value);
        } else {
            delete claudeSettingsData.env.MAX_MCP_OUTPUT_TOKENS;
        }
        updatePreview();
    });

    // Bash超时时间变更
    document.getElementById('bashTimeout').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.BASH_DEFAULT_TIMEOUT_MS = parseInt(value);
        } else {
            delete claudeSettingsData.env.BASH_DEFAULT_TIMEOUT_MS;
        }
        updatePreview();
    });

    // MCP超时时间变更
    document.getElementById('mcpTimeout').addEventListener('input', function() {
        const value = this.value.trim();
        if (value) {
            claudeSettingsData.env.MCP_TIMEOUT = parseInt(value);
        } else {
            delete claudeSettingsData.env.MCP_TIMEOUT;
        }
        updatePreview();
    });

    // 标记为已设置
    claudeSettingsEventListenersSetup = true;
}

// 添加禁用工具
function addDeniedTool() {
    const input = document.getElementById('newDeniedTool');
    const tool = input.value.trim();
    
    if (!tool) {
        showClaudeSettingsMessage('请输入工具名称', 'warning');
        return;
    }
    
    if (claudeSettingsData.permissions.deny.includes(tool)) {
        showClaudeSettingsMessage('工具已在禁用列表中', 'warning');
        return;
    }
    
    claudeSettingsData.permissions.deny.push(tool);
    input.value = '';
    renderDeniedTools();
    updatePreview();
}

// 移除禁用工具
function removeDeniedTool(tool) {
    claudeSettingsData.permissions.deny = 
        claudeSettingsData.permissions.deny.filter(t => t !== tool);
    renderDeniedTools();
    updatePreview();
}

// 添加自定义环境变量
function addCustomEnvVar() {
    const keyInput = document.getElementById('newEnvKey');
    const valueInput = document.getElementById('newEnvValue');
    
    const key = keyInput.value.trim();
    const value = valueInput.value.trim();
    
    if (!key) {
        showClaudeSettingsMessage('请输入环境变量名称', 'warning');
        return;
    }
    
    // 检查是否是系统管理的环境变量
    const systemManagedEnvVars = [
        'IS_SANDBOX',
        'DISABLE_AUTOUPDATER',
        'DISABLE_PROMPT_CACHING',
        'ANTHROPIC_MODEL',
        'ANTHROPIC_SMALL_FAST_MODEL',
        'CLAUDE_CODE_MAX_OUTPUT_TOKENS',
        'MAX_THINKING_TOKENS',
        'MAX_MCP_OUTPUT_TOKENS',
        'BASH_DEFAULT_TIMEOUT_MS',
        'MCP_TIMEOUT'
    ];

    if (systemManagedEnvVars.includes(key)) {
        showClaudeSettingsMessage('该环境变量由系统管理，请使用上面的配置选项', 'warning');
        return;
    }
    
    claudeSettingsData.env[key] = value;
    keyInput.value = '';
    valueInput.value = '';
    renderCustomEnvVars();
    updatePreview();
}

// 移除自定义环境变量
function removeCustomEnvVar(key) {
    delete claudeSettingsData.env[key];
    renderCustomEnvVars();
    updatePreview();
}

// 更新预览
function updatePreview() {
    const preview = document.getElementById('settingsJsonPreview');

    // 清理数据，移除空值
    const cleanData = {
        permissions: {
            defaultMode: claudeSettingsData.permissions.defaultMode,
            allow: claudeSettingsData.permissions.allow,
            deny: claudeSettingsData.permissions.deny.length > 0 ? claudeSettingsData.permissions.deny : []
        },
        env: {}
    };

    // 只添加非默认的环境变量
    Object.entries(claudeSettingsData.env).forEach(([key, value]) => {
        if (value !== '' && value !== null && value !== undefined) {
            // 对于开关类型的环境变量，如果值为 '0' 或 0，则不添加到配置中
            const switchEnvVars = ['IS_SANDBOX', 'DISABLE_AUTOUPDATER', 'DISABLE_PROMPT_CACHING'];
            if (switchEnvVars.includes(key) && (value === '0' || value === 0)) {
                return;
            }
            cleanData.env[key] = value;
        }
    });

    // 如果env为空，移除它
    if (Object.keys(cleanData.env).length === 0) {
        delete cleanData.env;
    }

    // 如果deny为空数组，移除它
    if (cleanData.permissions.deny.length === 0) {
        delete cleanData.permissions.deny;
    }

    preview.value = JSON.stringify(cleanData, null, 2);
}

// 保存Claude配置到数据库
async function saveClaudeConfigToDatabase() {
    try {
        const jsonContent = document.getElementById('settingsJsonPreview').value;

        console.log('准备保存Claude配置到数据库:', jsonContent);

        // 验证JSON格式
        let parsedJson;
        try {
            parsedJson = JSON.parse(jsonContent);
            console.log('JSON格式验证通过:', parsedJson);
        } catch (error) {
            const errorMsg = 'JSON格式错误: ' + error.message;
            console.error(errorMsg);
            showClaudeSettingsMessage(errorMsg, 'error');
            return;
        }

        try {
            await invoke('save_claude_settings_to_db', { settingsJson: jsonContent });
            console.log('Claude配置已成功保存到数据库');
            showClaudeSettingsMessage('Claude配置保存成功！配置已保存到数据库', 'success');

            // 如果账号关联页面是活动的，更新配置状态显示
            if (getActiveTab() === 'association-pane') {
                await loadClaudeConfigStatusInAssociation();
            }
        } catch (invokeError) {
            const errorMsg = getErrorMessage(invokeError);
            console.error('调用后端保存接口失败:', errorMsg);
            throw new Error('后端保存失败: ' + errorMsg);
        }

    } catch (error) {
        console.error('保存Claude配置失败:', error);
        const detailedError = getErrorMessage(error);
        showClaudeSettingsMessage(
            '保存Claude配置失败: ' + detailedError +
            '\n\n请检查:\n1. 数据库连接是否正常\n2. 数据库表结构是否完整\n3. 查看浏览器控制台获取详细错误信息',
            'error'
        );
    }
}

// 获取Claude配置用于账号切换（将会写入.claude/settings.local.json）
async function getClaudeSettingsForSwitch() {
    try {
        const settings = await invoke('get_claude_settings_from_db');
        return settings;
    } catch (error) {
        // 如果数据库中没有配置，返回默认配置
        return {
            permissions: {
                defaultMode: 'bypassPermissions',
                allow: ['*'],
                deny: []
            },
            env: {
                IS_SANDBOX: '1',
                DISABLE_AUTOUPDATER: 1
            }
        };
    }
}

// Claude设置消息显示
function showClaudeSettingsMessage(message, type = 'info') {
    const container = document.getElementById('claudeSettingsStatus');

    let alertClass = 'alert-info';
    let icon = 'fas fa-info-circle';

    if (type === 'success') {
        alertClass = 'alert-success';
        icon = 'fas fa-check-circle';
    } else if (type === 'error') {
        alertClass = 'alert-danger';
        icon = 'fas fa-exclamation-circle';
    } else if (type === 'warning') {
        alertClass = 'alert-warning';
        icon = 'fas fa-exclamation-triangle';
    }

    // 将消息中的换行符转换为HTML换行
    const formattedMessage = message.replace(/\n/g, '<br>');

    container.innerHTML = `
        <div class="${alertClass} alert alert-dismissible fade show">
            <i class="${icon} me-2"></i>
            <span style="white-space: pre-wrap;">${formattedMessage}</span>
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        </div>
    `;

    // 成功消息自动消失，错误和警告消息需要手动关闭
    if (type === 'success') {
        setTimeout(() => {
            container.innerHTML = '';
        }, 5000);
    }
}

// 在账号关联页面加载Claude配置状态
async function loadClaudeConfigStatusInAssociation() {
    try {
        const settings = await invoke('get_claude_settings_from_db');
        
        // 显示Claude配置状态区域
        document.getElementById('claudeConfigStatus').style.display = 'block';
        
        // 更新权限模式
        const permissionMode = settings.permissions?.defaultMode || 'normal';
        const permissionElement = document.getElementById('claudePermissionMode');
        
        switch (permissionMode) {
            case 'bypassPermissions':
                permissionElement.textContent = '绕过权限';
                permissionElement.className = 'badge bg-warning';
                break;
            case 'acceptEdits':
                permissionElement.textContent = '自动批准编辑';
                permissionElement.className = 'badge bg-info';
                break;
            case 'normal':
            default:
                permissionElement.textContent = '标准模式';
                permissionElement.className = 'badge bg-success';
                break;
        }
        
        // 更新沙盒模式状态
        const sandboxMode = settings.env?.IS_SANDBOX;
        const sandboxElement = document.getElementById('claudeSandboxMode');
        
        if (sandboxMode === '1' || sandboxMode === 1) {
            sandboxElement.textContent = '已启用';
            sandboxElement.className = 'badge bg-success';
        } else {
            sandboxElement.textContent = '已禁用';
            sandboxElement.className = 'badge bg-secondary';
        }
        
        // 更新自动更新状态
        const autoUpdater = settings.env?.DISABLE_AUTOUPDATER;
        const autoUpdaterElement = document.getElementById('claudeAutoUpdater');
        
        if (autoUpdater === 1 || autoUpdater === '1') {
            autoUpdaterElement.textContent = '已禁用';
            autoUpdaterElement.className = 'badge bg-success';
        } else {
            autoUpdaterElement.textContent = '已启用';
            autoUpdaterElement.className = 'badge bg-secondary';
        }
        
        // 更新允许工具状态
        const allowedTools = settings.permissions?.allow || [];
        const allowedToolsElement = document.getElementById('claudeAllowedTools');
        
        if (allowedTools.includes('*')) {
            allowedToolsElement.textContent = '全部允许 (*)';
            allowedToolsElement.className = 'badge bg-success';
        } else if (allowedTools.length > 0) {
            allowedToolsElement.textContent = `${allowedTools.length} 个工具`;
            allowedToolsElement.className = 'badge bg-info';
            allowedToolsElement.title = allowedTools.join(', ');
        } else {
            allowedToolsElement.textContent = '无';
            allowedToolsElement.className = 'badge bg-warning';
        }
        
    } catch (error) {
        console.warn('加载Claude配置状态失败:', error);
        // 隐藏Claude配置状态区域
        document.getElementById('claudeConfigStatus').style.display = 'none';
    }
}

// Make functions available globally
window.saveAccount = saveAccount;
window.saveDirectory = saveDirectory;
window.saveBaseUrl = saveBaseUrl;
window.editAccount = editAccount;
window.promptDeleteAccount = promptDeleteAccount;
window.editDirectory = editDirectory;
window.viewConfig = viewConfig;
window.editBaseUrl = editBaseUrl;
window.deleteBaseUrl = deleteBaseUrl;
window.promptDeleteBaseUrl = promptDeleteBaseUrl;
window.loadAccounts = loadAccounts;
window.performAccountSwitch = performAccountSwitch;
window.quickSwitchFromList = quickSwitchFromList;
window.onDirectorySelectionChange = onDirectorySelectionChange;
window.loadDatabaseInfo = loadDatabaseInfo;
window.switchDatabase = switchDatabase;
window.testDatabase = testDatabase;
window.selectDirectory = selectDirectory;
window.promptDeleteDirectory = promptDeleteDirectory;

// Claude Settings functions
window.loadClaudeSettingsPage = loadClaudeSettingsPage;
window.addDeniedTool = addDeniedTool;
window.removeDeniedTool = removeDeniedTool;
window.addCustomEnvVar = addCustomEnvVar;
window.removeCustomEnvVar = removeCustomEnvVar;
window.updatePreview = updatePreview;
window.saveClaudeConfigToDatabase = saveClaudeConfigToDatabase;
window.getClaudeSettingsForSwitch = getClaudeSettingsForSwitch;
