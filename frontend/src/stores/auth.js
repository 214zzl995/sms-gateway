import { writable } from 'svelte/store';
import { getStorageValue, updateStorageValue } from '../js/storage';
import { apiClient } from '../js/api';

// 认证状态管理
export const isAuthenticated = writable(false);
export const isAuthLoading = writable(true);

// 立即开始初始化认证状态
initAuth();

// 初始化认证状态
export async function initAuth() {
    isAuthLoading.set(true);
    
    try {
        const authData = await getStorageValue('auth');
        const isValid = authData && await apiClient.checkAuth();
        
        isAuthenticated.set(!!isValid);
    } catch (error) {
        console.error('Auth initialization failed:', error);
        isAuthenticated.set(false);
    } finally {
        isAuthLoading.set(false);
    }
}

// 登录
export async function login(username, password) {
    try {
        // 这里应该调用实际的登录 API
        // 暂时使用简单的验证逻辑
        if (username && password) {
            await updateStorageValue('auth', { username, timestamp: Date.now() });
            isAuthenticated.set(true);
            return { success: true };
        } else {
            return { success: false, error: '用户名和密码不能为空' };
        }
    } catch (error) {
        console.error('Login failed:', error);
        return { success: false, error: '登录失败，请稍后重试' };
    }
}

// 登出
export async function logout() {
    try {
        await updateStorageValue('auth', null);
        isAuthenticated.set(false);
        window.location.reload();
    } catch (error) {
        console.error('Logout failed:', error);
    }
}