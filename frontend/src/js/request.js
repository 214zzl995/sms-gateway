const baseUrl = ''

//参数转换
const queryString = (/** @type {{ [x: string]: string | number | boolean; }} */ params) => '?' + Object
    .keys(params)
    .map(key => `${key}=${encodeURIComponent(params[key])}`)
    .join('&');

const request = (/** @type {string} */ partialUrl, /** @type {any} */ body, /** @type {any} */ query,
    method = 'GET',/** @type {RequestMode} */ mode = 'cors', contentType = 'application/json',/** @type Record<string, string> */ headers = {}) => {


    const needContentType = ['POST', 'PUT', 'GET'].includes(method.toUpperCase());
    const url = baseUrl + partialUrl + (query ? queryString(query) : '');

    if (contentType == "application/json"){
        body = JSON.stringify(body)
    }

    /** @type {RequestInit} */
    let requestConfig = {
        credentials: 'same-origin',
        method,
        body, //请求参数
        headers: {
            ...needContentType ? {
                'Content-Type': contentType
            } : {},
            ...headers
        },
        mode, // 用来决定是否允许跨域请求  值有 三个 same-origin，no-cors（默认）以及 cores;
        cache: "default" // 是否缓存请求资源 可选值有 default 、 no-store 、 reload 、 no-cache 、 force-cache 或者 only-if-cached 。
    }

    return new Promise((success, fail) => fetch(url, requestConfig).then(response => {
        success(response.json());
        fail(response);
    }))
}

class Fetch {
    /**
     * @param {undefined} [before]
     * @param {undefined} [after]
     */
    constructor(before, after) {
        /**
         * @type {() => any}
         */
        this.before = before; //请求前拦截器
        /**
         * @type {() => any}
         */
        this.after = after; //请求后拦截器
    }

    /**
     * @param {string} partialUrl
     * @param {any} body
     * @param {any} query
     * @param {string} method
     * @param {RequestMode} mode
     * @param {string} [contentType]
     * @param {Record<string, string>} [headers]
     */
    _request(partialUrl, body, query, method, mode, contentType, headers) {
        this.before && this.before();	// 执行请求前拦截器。
        const promise = request(partialUrl, body, query, method, mode, contentType, headers);
        promise.finally(() => this.after && this.after()); // 执行请求后拦截器。
        return promise;
    }

    /**
     * @param {string} partialUrl
     * @param {any} query
     * @param {string} contentType
     * @param {Record<string, string>} [headers]
     */
    get(partialUrl, query, contentType, headers) {
        return this._request(partialUrl, undefined, query, "GET", undefined, contentType, headers);
    }

    /**
     * @param {string} partialUrl
     * @param {any} query
     */
    delete(partialUrl, query) {
        return this._request(partialUrl, undefined, query, 'DELETE', undefined);
    }

    /**
     * @param {string} partialUrl
     * @param {any} body
     * @param {any} query
     * @param {string} contentType
     */
    post(partialUrl, body, query, contentType) {
        return this._request(partialUrl, body, query, 'POST', undefined, contentType)
    }

    /**
     * @param {string} partialUrl
     * @param {any} body
     * @param {any} query
     * @param {string} contentType
     */
    put(partialUrl, body, query, contentType) {
        return this._request(partialUrl, body, query, 'PUT', undefined, contentType)
    }

}

const FetchApi = {
    get: (/** @type {string} */ partialUrl, /** @type {string} */ query, /** @type {string} */ contentType,/** @type {Record<string, string>} */ headers) => {
        return new Fetch().get(partialUrl, query, contentType,headers);
    },
    delete: (/** @type {string} */ partialUrl, /** @type {any} */ query) => {
        return new Fetch().delete(partialUrl, query);
    },
    post: (/** @type {string} */ partialUrl, /** @type {any} */ body, /** @type {any} */ query, /** @type {string} */ contentType) => {
        return new Fetch().post(partialUrl, body, query, contentType);
    },
    put: (/** @type {string} */ partialUrl, /** @type {any} */ body, /** @type {any} */ query, /** @type {string} */ contentType) => {
        return new Fetch().put(partialUrl, body, query, contentType);
    }
}

export default FetchApi;