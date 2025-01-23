import api from './request.js';


export const getSecure = () => api.get(`/secure`)

export const getAllTopic = () => api.get(`/c/get-all-topic`)

export const getConfig = () => api.get(`/c/config`)

export const getConfigGuide = () => api.get(`/g/config`)

export const putConfig = ( /** @type {any} */ body) => api.post('/c/config', body)

export const putConfigGuide = ( /** @type {any} */ body) => api.post('/g/config', body)

export const getApi = (/** @type {string} */ api_token) => api.get('/check', undefined, undefined, {
    'Authorization': 'Bearer ' + api_token,
})

export const uploadCrt = (/** @type {string} */ filename, /** @type {FormData} */ body) => api.post(`/c/upload-crt/${filename}`, body, "", "multipart/form-data")

export const uploadCrtGuide = (/** @type {string} */ filename, /** @type {FormData} */ body) => api.post(`/g/upload-crt/${filename}`, body, "", "multipart/form-data")

export const getMessagesByHeader = (/** @type {string} */ header) => api.get(`/c/msgs/${encodeURIComponent(header)}`)

export const getMessagesCountByHeader = (/** @type {string} */ header) => api.get(`/c/msg-count/${encodeURIComponent(header)}`)

export const getPageMessagesByHeader = (/** @type {string} */ header, /** @type {any} */ page_param) => api.get(`/c/page-msgs/${encodeURIComponent(header)}`, page_param)

export const sendMsg = (/** @type {any} */ params) => api.post(`/c/send`, params)

export const getMqttUser = () => api.get(`/c/get-mqtt-user`)

export const getAllTask = () => api.get(`/c/task`)

export const addTask = (/** @type {any} */ body) => api.post(`/c/task`, body)

export const removeTask = (/** @type {string} */ id) => api.delete(`/c/task/${id}`)

export const restartTask = (/** @type {string} */ id) => api.get(`/c/restart-task/${id}`)

export const stopTask = (/** @type {string} */ id) => api.get(`/c/stop-task/${id}`)

export const startTask = (/** @type {string} */ id) => api.get(`/c/start-task/${id}`)

export const updateTask = (/** @type {string} */ id, /** @type {any} */ body) => api.post(`/c/update-task/${id}`, body)

export const getAllScriptFile = () => api.get(`/c/script-file-name`)

export const uploadScript = (/** @type {string} */ filename, /** @type {FormData} */ body) => api.post(`/c/upload-script/${filename}`, body, "", "multipart/form-data")
