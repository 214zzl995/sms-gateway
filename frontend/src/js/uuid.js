// uuid.js
/**
 * 生成 UUID v4
 * @returns {string} - 生成的 UUID
 */
export function generateUUID() {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
        // 现代浏览器或 Node.js 20+ 直接使用
        return crypto.randomUUID();
    }

    // Fallback：手动生成安全 UUID v4
    const bytes = new Uint8Array(16);
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
        crypto.getRandomValues(bytes); // 浏览器
    } else {
        // Node.js
        const { randomBytes } = require('crypto');
        const nodeBytes = randomBytes(16);
        bytes.set(nodeBytes);
    }

    // 设置版本和 variant
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant

    // 转成 UUID 字符串
    const hex = [...bytes].map(b => b.toString(16).padStart(2, '0'));
    return `${hex[0]}${hex[1]}${hex[2]}${hex[3]}-${hex[4]}${hex[5]}-${hex[6]}${hex[7]}-${hex[8]}${hex[9]}-${hex[10]}${hex[11]}${hex[12]}${hex[13]}${hex[14]}${hex[15]}`;
}
