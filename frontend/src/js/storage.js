export const getStorageValue = async (/** @type {string} */ name) => {
    return new Promise((resolve, reject) => {
        try {
            const storedValue = sessionStorage.getItem(name);
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

export const updateStorageValue = async (/** @type {string} */ name, /** @type {any} */ value) => {
    return new Promise((resolve, reject) => {
        try {
            sessionStorage.setItem(name, JSON.stringify(value));
            resolve("Value updated successfully");
        } catch (error) {
            reject(error);
        }
    });
};