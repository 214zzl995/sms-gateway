// 从 localStorage 中获取特定参数的值
export const getStorageValue = async (/** @type {string} */ name) => {
    return new Promise((resolve, reject) => {
        try {
            const storedValue = localStorage.getItem(name);
            if (storedValue) {
                resolve(JSON.parse(storedValue));
            } else {
                resolve(null);
            }
        } catch (error) {
            reject(error);
        }
    });
};

// 更新特定参数的值并存储到 localStorage
export const updateStorageValue = async (/** @type {string} */ name, /** @type {any} */ value) => {
    return new Promise((resolve, reject) => {
        try {
            localStorage.setItem(name, JSON.stringify(value));
            resolve("Value updated successfully");
        } catch (error) {
            reject(error);
        }
    });
};