// ============================================================
// 版本更新检测服务
// 通过 GitHub Releases API 检查公仓是否有新版本
// ============================================================

import { ElMessageBox } from 'element-plus';

// 公仓 Releases API 地址
const RELEASES_API = 'https://api.github.com/repos/1837620622/windsurf-account-manager-releases/releases/latest';

// 公仓 Releases 页面地址
const RELEASES_PAGE = 'https://github.com/1837620622/windsurf-account-manager-releases/releases';

// ============================================================
// 版本号比较工具函数
// 将 "v2.2.0" 或 "2.2.0" 转换为数值数组 [2, 2, 0] 进行逐段比较
// ============================================================
function parseVersion(version: string): number[] {
  return version
    .replace(/^v/, '')
    .split('.')
    .map(n => parseInt(n, 10) || 0);
}

// 比较两个版本号，返回 1（remote > local）、0（相等）、-1（local > remote）
function compareVersions(remote: string, local: string): number {
  const r = parseVersion(remote);
  const l = parseVersion(local);
  const maxLen = Math.max(r.length, l.length);

  for (let i = 0; i < maxLen; i++) {
    const rv = r[i] || 0;
    const lv = l[i] || 0;
    if (rv > lv) return 1;
    if (rv < lv) return -1;
  }
  return 0;
}

// ============================================================
// 检查更新主函数
// currentVersion: 当前应用版本号（如 "2.2.0"）
// ============================================================
export async function checkForUpdate(currentVersion: string): Promise<void> {
  try {
    const controller = new AbortController();
    // 设置 10 秒超时，避免网络问题阻塞启动
    const timeout = setTimeout(() => controller.abort(), 10000);

    const response = await fetch(RELEASES_API, {
      headers: {
        'Accept': 'application/vnd.github.v3+json',
      },
      signal: controller.signal,
    });
    clearTimeout(timeout);

    if (!response.ok) {
      console.log('[更新检测] API 请求失败:', response.status);
      return;
    }

    const data = await response.json();
    const latestTag: string = data.tag_name || '';
    const latestVersion = latestTag.replace(/^v/, '');
    const releaseUrl: string = data.html_url || RELEASES_PAGE;
    const releaseBody: string = data.body || '';

    console.log(`[更新检测] 当前版本: ${currentVersion}, 最新版本: ${latestVersion}`);

    // 对比版本号
    if (compareVersions(latestVersion, currentVersion) > 0) {
      // 提取更新内容摘要（取前 500 字符）
      const changelogPreview = releaseBody.length > 500
        ? releaseBody.substring(0, 500) + '...'
        : releaseBody;

      // 弹出更新提示框
      ElMessageBox.confirm(
        `<div style="line-height: 1.8;">
          <p><strong>发现新版本 v${latestVersion}</strong>（当前版本 v${currentVersion}）</p>
          <div style="margin-top: 12px; padding: 12px; background: rgba(64,158,255,0.08); border-radius: 8px; font-size: 13px; max-height: 200px; overflow-y: auto; white-space: pre-wrap;">${changelogPreview}</div>
          <p style="margin-top: 12px; color: #909399; font-size: 12px;">点击「前往下载」跳转到 GitHub Releases 页面</p>
        </div>`,
        '版本更新提醒',
        {
          confirmButtonText: '前往下载',
          cancelButtonText: '稍后再说',
          dangerouslyUseHTMLString: true,
          type: 'info',
          customClass: 'update-dialog',
        }
      ).then(() => {
        // 用户点击"前往下载"，打开浏览器
        window.open(releaseUrl, '_blank');
      }).catch(() => {
        // 用户点击"稍后再说"，不做任何事
      });
    } else {
      console.log('[更新检测] 已是最新版本');
    }
  } catch (error: any) {
    // 网络超时或其他错误，静默忽略（不影响正常使用）
    if (error.name === 'AbortError') {
      console.log('[更新检测] 请求超时，跳过检测');
    } else {
      console.log('[更新检测] 检测失败:', error.message);
    }
  }
}
