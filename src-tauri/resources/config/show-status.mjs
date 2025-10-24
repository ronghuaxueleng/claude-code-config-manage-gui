#!/usr/bin/env node
/**
 * Claude Code 积分状态栏脚本
 * 用途: 在状态栏显示配置信息
 */

import fs from 'fs';
import path from 'path';
import os from 'os';

// 禁用SSL证书验证警告
process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

function getDisplayUrl() {
    const baseUrl = process.env.ANTHROPIC_BASE_URL || '';
    if (baseUrl) {
        const match = baseUrl.match(/https?:\/\/([^\/]+)/);
        if (match) {
            return match[1];
        }
    }
    return '';
}

function getCurrentModel() {
    // 优先使用环境变量
    let model = process.env.ANTHROPIC_MODEL || '';

    // 如果环境变量没有，检查settings.json
    if (!model) {
        try {
            const settingsFile = path.join(os.homedir(), '.claude', 'settings.json');
            if (fs.existsSync(settingsFile)) {
                const settings = JSON.parse(fs.readFileSync(settingsFile, 'utf8'));
                model = settings.model || '';
            }
        } catch (error) {
            // 忽略错误
        }
    }

    if (model) {
        if (model.toLowerCase().includes('claude-3')) {
            if (model.toLowerCase().includes('haiku')) {
                return 'Claude 3 Haiku';
            } else if (model.toLowerCase().includes('sonnet')) {
                return 'Claude 3 Sonnet';
            } else if (model.toLowerCase().includes('opus')) {
                return 'Claude 3 Opus';
            }
        } else if (model.toLowerCase().includes('claude-4') || model.toLowerCase().includes('sonnet-4')) {
            return 'Claude 4 Sonnet';
        } else if (model.toLowerCase().includes('opus-4')) {
            return 'Claude 4 Opus';
        } else if (model.length > 20) {
            return model.substring(0, 20) + '...';
        }
        return model;
    }

    return 'Claude (Auto)';
}

async function main() {
    try {
        const currentUrl = getDisplayUrl();
        const currentModel = getCurrentModel();
        const userName = process.env.USER_NAME || '';

        const parts = [];
        if (userName) parts.push(`👤 ${userName}`);
        parts.push(currentModel);
        parts.push(currentUrl);

        console.log(parts.join(' | '));

    } catch (error) {
        // 即使出错也显示基本信息
        const currentUrl = getDisplayUrl();
        const currentModel = getCurrentModel();
        const userName = process.env.USER_NAME || '';
        const parts = ['🔴 错误'];
        if (userName) parts.push(`👤 ${userName}`);
        parts.push(currentModel);
        parts.push(currentUrl);
        console.log(parts.join(' | '));
    }
}

// ES Module 中直接执行
main();
