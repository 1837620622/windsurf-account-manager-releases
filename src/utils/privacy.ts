/**
 * 隐私模式工具函数
 */

// 缓存已生成的隐私邮箱，保证同一邮箱每次显示相同
const privacyEmailCache = new Map<string, string>();

/**
 * 将邮箱地址转换为隐私模式显示
 * @param email 原始邮箱
 * @returns 隐私邮箱
 */
export function maskEmail(email: string): string {
  if (!email) return email;
  
  // 检查缓存
  if (privacyEmailCache.has(email)) {
    return privacyEmailCache.get(email)!;
  }
  
  // 基于邮箱生成固定的随机字符串（使用简单hash保证一致性）
  let hash = 0;
  for (let i = 0; i < email.length; i++) {
    hash = ((hash << 5) - hash) + email.charCodeAt(i);
    hash = hash & hash;
  }
  
  // 使用hash作为种子生成固定的随机字符串
  const seed = Math.abs(hash);
  const chars = 'abcdefghijklmnopqrstuvwxyz';
  let randomPart = '';
  let tempSeed = seed;
  for (let i = 0; i < 12; i++) {
    randomPart += chars.charAt(tempSeed % chars.length);
    tempSeed = Math.floor(tempSeed / 26) + i;
  }
  
  const maskedEmail = `${randomPart}@fuckwindsurf.com`;
  privacyEmailCache.set(email, maskedEmail);
  
  return maskedEmail;
}

/**
 * 清除隐私邮箱缓存
 */
export function clearPrivacyCache(): void {
  privacyEmailCache.clear();
}
