#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
批量替换main.js中的中文字符串为i18n翻译调用
"""

import re

# 定义替换规则 (中文字符串 -> i18n键名)
replacements = [
    # 账号相关
    (r"'找不到账号信息'", "window.i18n.t('error.account_not_found')"),
    (r"'编辑账号失败: '", "window.i18n.t('error.edit_account') + ': '"),
    (r"'账号更新成功'", "window.i18n.t('success.account_updated')"),
    (r"'账号名称已存在，请使用不同的名称'", "window.i18n.t('error.account_name_exists')"),
    (r"'更新账号失败: '", "window.i18n.t('error.update_account') + ': '"),
    (r"'编辑账号'", "window.i18n.t('modal.edit_account')"),
    (r"'添加账号'", "window.i18n.t('modal.add_account')"),
    (r"'确定要删除账号 \"' \+ accountName \+ '\" 吗？\\\\n\\\\n此操作不可撤销！'", "window.i18n.t('confirm.delete_account_with_name').replace('{name}', accountName)"),
    (r"'确认删除账号'", "window.i18n.t('confirm.delete_account_title')"),
    (r"'确定要删除这个账号吗？'", "window.i18n.t('confirm.delete_account')"),
    (r"'账号删除成功'", "window.i18n.t('success.account_deleted')"),
    (r"'删除账号失败: '", "window.i18n.t('error.delete_account') + ': '"),
    (r"'账号 \"' \+ accountName \+ '\" 删除成功'", "window.i18n.t('success.account_deleted_with_name').replace('{name}', accountName)"),
    (r"'删除账号 \"' \+ accountName \+ '\" 失败: '", "window.i18n.t('error.delete_account') + ' \"' + accountName + '\": '"),

    # 目录相关
    (r"'找不到目录信息'", "window.i18n.t('error.directory_not_found')"),
    (r"'编辑目录'", "window.i18n.t('modal.edit_directory')"),
    (r"'编辑目录失败: '", "window.i18n.t('error.edit_directory') + ': '"),
    (r"'添加目录'", "window.i18n.t('modal.add_directory')"),
    (r"'删除操作正在进行中，请稍候\.\.\.'", "window.i18n.t('info.deleting')"),
    (r"'提示'", "window.i18n.t('info.deleting_title')"),
    (r"'确定要删除目录 \"' \+ directoryName \+ '\" 吗？\\\\n\\\\n此操作将删除数据库记录，但不会删除文件系统中的目录文件。'", "window.i18n.t('confirm.delete_directory').replace('{name}', directoryName)"),
    (r"'确认删除目录'", "window.i18n.t('confirm.delete_directory_title')"),
    (r"'目录 \"' \+ directoryName \+ '\" 在文件系统中不存在。\\\\n\\\\n确定要清理数据库中的记录吗？'", "window.i18n.t('confirm.cleanup_directory').replace('{name}', directoryName)"),
    (r"'确认清理记录'", "window.i18n.t('confirm.cleanup_directory_title')"),
    (r"'没有发现无效目录'", "window.i18n.t('success.no_invalid_directories')"),
    (r"'目录 \"' \+ directoryName \+ '\" 删除成功'", "window.i18n.t('success.directory_deleted').replace('{name}', directoryName)"),
    (r"'删除目录 \"' \+ directoryName \+ '\" 失败: '", "window.i18n.t('error.delete_directory') + ' \"' + directoryName + '\": '"),
    (r"'清除无效目录失败: '", "window.i18n.t('error.cleanup_directories') + ': '"),

    # URL相关
    (r"'加载URL失败: '", "window.i18n.t('error.load_urls') + ': '"),
    (r"'找不到URL信息'", "window.i18n.t('error.url_not_found')"),
    (r"'编辑URL'", "window.i18n.t('modal.edit_url')"),
    (r"'编辑URL失败: '", "window.i18n.t('error.edit_url') + ': '"),
    (r"'添加URL'", "window.i18n.t('modal.add_url')"),
    (r"'URL添加成功'", "window.i18n.t('success.url_added')"),
    (r"'URL名称已存在，请使用不同的名称'", "window.i18n.t('error.url_name_exists')"),
    (r"'URL地址已存在，请使用不同的URL地址'", "window.i18n.t('error.url_address_exists')"),
    (r"'添加URL失败: '", "window.i18n.t('error.add_url') + ': '"),
    (r"'URL更新成功'", "window.i18n.t('success.url_updated')"),
    (r"'更新URL失败: '", "window.i18n.t('error.update_url') + ': '"),
    (r"'确定要删除URL \"' \+ urlName \+ '\" 吗？\\\\n\\\\n此操作不可撤销！'", "window.i18n.t('confirm.delete_url_with_name').replace('{name}', urlName)"),
    (r"'确认删除URL'", "window.i18n.t('confirm.delete_url_title')"),
    (r"'确定要删除这个URL吗？'", "window.i18n.t('confirm.delete_url')"),
    (r"'URL删除成功'", "window.i18n.t('success.url_deleted')"),
    (r"'删除URL失败: '", "window.i18n.t('error.delete_url') + ': '"),
    (r"'URL \"' \+ urlName \+ '\" 删除成功'", "window.i18n.t('success.url_deleted_with_name').replace('{name}', urlName)"),
    (r"'删除URL \"' \+ urlName \+ '\" 失败: '", "window.i18n.t('error.delete_url') + ' \"' + urlName + '\": '"),

    # 其他错误消息
    (r"'获取配置失败: '", "window.i18n.t('error.get_config') + ': '"),
    (r"'加载账号关联页面失败: '", "window.i18n.t('error.load_association_page') + ': '"),
    (r"'加载目录列表失败: '", "window.i18n.t('error.load_directory_list') + ': '"),
    (r"'加载账号列表失败: '", "window.i18n.t('error.load_account_list') + ': '"),
    (r"'请先选择目录'", "window.i18n.t('error.select_directory_first')"),
    (r"'请选择要切换的账号'", "window.i18n.t('error.select_account')"),
    (r"'切换账号失败: '", "window.i18n.t('error.switch_account') + ': '"),
    (r"'选择目录失败: '", "window.i18n.t('error.select_directory') + ': '"),

    # 数据库相关
    (r"'加载数据库信息失败: '", "window.i18n.t('error.load_database_info') + ': '"),
    (r"'无连接信息'", "window.i18n.t('info.no_connection_info')"),
    (r"'选择的数据库连接不存在'", "window.i18n.t('error.connection_not_exist')"),
    (r"'预览连接信息失败: '", "window.i18n.t('error.preview_connection') + ': '"),
    (r"'请选择数据库连接'", "window.i18n.t('error.select_database')"),
    (r"'请先测试数据库连接成功后再切换'", "window.i18n.t('error.test_before_switch')"),
    (r"'切换数据库失败: '", "window.i18n.t('error.switch_database') + ': '"),
    (r"'正在测试数据库连接，请稍候\.\.\.'", "window.i18n.t('info.testing_connection')"),
    (r"'数据库连接测试失败: '", "window.i18n.t('error.test_database') + ': '"),

    # Claude配置相关
    (r"'加载Claude配置失败: '", "window.i18n.t('error.load_claude_settings') + ': '"),
    (r"'从数据库加载配置失败，正在使用默认配置。错误: '", "window.i18n.t('claude.loading_warning').replace('{error}', '')"),
    (r"'从数据库加载配置失败，正在使用默认配置'", "window.i18n.t('claude.loading_warning_simple')"),
    (r"'数据库返回的配置格式无效'", "window.i18n.t('claude.invalid_format')"),
    (r"'JSON格式错误: '", "window.i18n.t('claude.json_error') + ': '"),
    (r"'后端保存失败: '", "window.i18n.t('claude.save_error') + ': '"),
    (r"'保存Claude配置失败: '", "window.i18n.t('error.save_claude_settings') + ': '"),
    (r"'Claude配置保存成功！配置已保存到数据库'", "window.i18n.t('success.claude_settings_saved')"),
    (r"'请输入工具名称'", "window.i18n.t('claude.tool_name_required')"),
    (r"'工具已在禁用列表中'", "window.i18n.t('claude.tool_already_denied')"),
    (r"'请输入环境变量名称'", "window.i18n.t('claude.env_name_required')"),
    (r"'该环境变量由系统管理，请使用上面的配置选项'", "window.i18n.t('claude.env_system_managed')"),

    # WebDAV相关
    (r"'加载 WebDAV 配置失败: '", "window.i18n.t('webdav.load_failed') + ': '"),
    (r"'WebDAV 配置创建成功'", "window.i18n.t('success.webdav_created')"),
    (r"'创建 WebDAV 配置失败: '", "window.i18n.t('webdav.create_failed') + ': '"),
    (r"'WebDAV 配置更新成功'", "window.i18n.t('success.webdav_updated')"),
    (r"'更新 WebDAV 配置失败: '", "window.i18n.t('webdav.update_failed') + ': '"),
    (r"'确定要删除 WebDAV 配置 \"' \+ config\.name \+ '\" 吗？'", "window.i18n.t('confirm.delete_webdav').replace('{name}', config.name)"),
    (r"'确认删除'", "window.i18n.t('confirm.delete_webdav_title')"),
    (r"'WebDAV 配置删除成功'", "window.i18n.t('success.webdav_deleted')"),
    (r"'删除 WebDAV 配置失败: '", "window.i18n.t('webdav.delete_failed') + ': '"),
    (r"'连接测试失败: '", "window.i18n.t('webdav.test_failed') + ': '"),
    (r"'已设置为活跃配置'", "window.i18n.t('success.webdav_active')"),
    (r"'设置失败: '", "window.i18n.t('webdav.set_active_failed') + ': '"),
    (r"'上传失败: '", "window.i18n.t('webdav.upload_failed') + ': '"),
    (r"'获取文件列表失败: '", "window.i18n.t('webdav.list_failed') + ': '"),
    (r"'确定要下载并应用配置文件 \"' \+ filename \+ '\" 吗？\\\\n这将覆盖当前配置！'", "window.i18n.t('confirm.download_webdav').replace('{filename}', filename)"),
    (r"'确认下载'", "window.i18n.t('confirm.download_webdav_title')"),
    (r"'配置下载成功！'", "window.i18n.t('success.webdav_download')"),
    (r"'下载失败: '", "window.i18n.t('webdav.download_failed') + ': '"),

    # 语言切换
    (r"'已切换到中文'", "window.i18n.t('success.language_switched_zh')"),
    (r"'语言切换失败'", "window.i18n.t('error.language_switch')"),

    # 状态标签
    (r"'当前活跃'", "window.i18n.t('status.active')"),
    (r"'非活跃'", "window.i18n.t('status.inactive')"),
    (r"'已配置'", "window.i18n.t('status.configured')"),
    (r"'未配置'", "window.i18n.t('status.not_configured')"),
    (r"'已启用'", "window.i18n.t('status.enabled')"),
    (r"'已禁用'", "window.i18n.t('status.disabled')"),

    # 其他
    (r"'选择目录'", "window.i18n.t('button.select_directory')"),
    (r"'更新'", "window.i18n.t('button.update')"),
]

def main():
    # 读取文件
    with open('/mnt/e/GitHub/claude-code-config-manage-gui/src/main.js', 'r', encoding='utf-8') as f:
        content = f.read()

    # 执行替换
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content)

    # 写回文件
    with open('/mnt/e/GitHub/claude-code-config-manage-gui/src/main.js', 'w', encoding='utf-8') as f:
        f.write(content)

    print("替换完成！")

if __name__ == '__main__':
    main()
