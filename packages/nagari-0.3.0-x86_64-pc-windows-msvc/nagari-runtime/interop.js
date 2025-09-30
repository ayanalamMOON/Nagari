// Nagari <-> JavaScript Interoperability Layer
/**
 * JavaScript to Nagari value conversion
 */
export function jsToNagari(value) {
    if (value === null || value === undefined) {
        return null;
    }
    if (typeof value === 'boolean' || typeof value === 'number' || typeof value === 'string') {
        return value;
    }
    if (Array.isArray(value)) {
        return value.map(jsToNagari);
    }
    if (typeof value === 'function') {
        return wrapJSFunction(value);
    }
    if (typeof value === 'object') {
        const result = {};
        for (const [key, val] of Object.entries(value)) {
            result[key] = jsToNagari(val);
        }
        return result;
    }
    return value;
}
/**
 * Nagari to JavaScript value conversion
 */
export function nagariToJS(value) {
    if (value === null || value === undefined) {
        return value;
    }
    if (typeof value === 'boolean' || typeof value === 'number' || typeof value === 'string') {
        return value;
    }
    if (Array.isArray(value)) {
        return value.map(nagariToJS);
    }
    if (typeof value === 'function') {
        // If it's already a Nagari function, unwrap it
        if (value.__nagari_function__) {
            return async (...args) => {
                const result = value(...args.map(jsToNagari));
                return nagariToJS(await result);
            };
        }
        return value;
    }
    if (typeof value === 'object' && value !== null) {
        const result = {};
        for (const [key, val] of Object.entries(value)) {
            result[key] = nagariToJS(val);
        }
        return result;
    }
    return value;
}
/**
 * Wrap a JavaScript function to be callable from Nagari
 */
export function wrapJSFunction(fn, name) {
    const wrapped = (...args) => {
        const jsArgs = args.map(nagariToJS);
        const result = fn(...jsArgs);
        return jsToNagari(result);
    };
    wrapped.__nagari_function__ = true;
    // Set function name using defineProperty to avoid read-only issues
    try {
        Object.defineProperty(wrapped, 'name', {
            value: name || fn.name,
            configurable: true
        });
    }
    catch (e) {
        // Ignore if we can't set the name
    }
    wrapped.arity = fn.length;
    return wrapped;
}
/**
 * Wrap a JavaScript class to be instantiable from Nagari
 */
export function wrapJSClass(jsClass, name) {
    const wrapped = function (...args) {
        const jsArgs = args.map(nagariToJS);
        const instance = new jsClass(...jsArgs);
        return jsToNagari(instance);
    };
    wrapped.__nagari_class__ = true;
    wrapped.name = name || jsClass.name;
    // Copy static methods
    for (const key of Object.getOwnPropertyNames(jsClass)) {
        if (key !== 'length' && key !== 'name' && key !== 'prototype') {
            wrapped[key] = wrapJSFunction(jsClass[key], `${wrapped.name}.${key}`);
        }
    }
    return wrapped;
}
/**
 * Create a Nagari module from a JavaScript module/object
 */
export function createNagariModule(jsModule, name) {
    const module = {
        __nagari_module__: true,
        name
    };
    for (const [key, value] of Object.entries(jsModule)) {
        if (typeof value === 'function') {
            // Check if it's a constructor function (class)
            if (value.prototype && value.prototype.constructor === value) {
                module[key] = wrapJSClass(value, key);
            }
            else {
                module[key] = wrapJSFunction(value, key);
            }
        }
        else {
            module[key] = jsToNagari(value);
        }
    }
    return module;
}
/**
 * Dynamic import wrapper for Nagari
 */
export async function dynamicImport(modulePath) {
    try {
        const jsModule = await import(modulePath);
        return createNagariModule(jsModule, modulePath);
    }
    catch (error) {
        throw new Error(`Failed to import module '${modulePath}': ${error}`);
    }
}
/**
 * DOM API wrappers
 */
export const DOMInterop = {
    // Document methods
    getElementById: wrapJSFunction((id) => document.getElementById(id), 'getElementById'),
    querySelector: wrapJSFunction((selector) => document.querySelector(selector), 'querySelector'),
    querySelectorAll: wrapJSFunction((selector) => Array.from(document.querySelectorAll(selector)), 'querySelectorAll'),
    createElement: wrapJSFunction((tagName) => document.createElement(tagName), 'createElement'),
    // Window methods
    setTimeout: wrapJSFunction((callback, delay) => setTimeout(() => callback(), delay), 'setTimeout'),
    setInterval: wrapJSFunction((callback, delay) => setInterval(() => callback(), delay), 'setInterval'),
    clearTimeout: wrapJSFunction(clearTimeout, 'clearTimeout'),
    clearInterval: wrapJSFunction(clearInterval, 'clearInterval'),
    // Fetch API
    fetch: wrapJSFunction(async (url, options) => {
        const response = await fetch(url, options);
        return {
            ok: response.ok,
            status: response.status,
            statusText: response.statusText,
            headers: Object.fromEntries(response.headers.entries()),
            json: wrapJSFunction(() => response.json(), 'json'),
            text: wrapJSFunction(() => response.text(), 'text'),
            blob: wrapJSFunction(() => response.blob(), 'blob'),
            arrayBuffer: wrapJSFunction(() => response.arrayBuffer(), 'arrayBuffer')
        };
    }, 'fetch')
};
/**
 * Node.js API wrappers (conditional)
 */
export const NodeInterop = typeof globalThis !== 'undefined' &&
    globalThis.process !== undefined ? {
    readFile: wrapJSFunction(async (path, encoding = 'utf8') => {
        try {
            const fs = await import('node:fs/promises');
            return fs.readFile(path, encoding);
        }
        catch {
            throw new Error('fs module not available in this environment');
        }
    }, 'readFile'),
    writeFile: wrapJSFunction(async (path, data, encoding = 'utf8') => {
        try {
            const fs = await import('node:fs/promises');
            return fs.writeFile(path, data, encoding);
        }
        catch {
            throw new Error('fs module not available in this environment');
        }
    }, 'writeFile'),
    // HTTP
    createServer: wrapJSFunction(async (handler) => {
        try {
            const http = await import('node:http');
            return http.createServer((req, res) => {
                const nagariReq = jsToNagari({
                    method: req.method,
                    url: req.url,
                    headers: req.headers
                });
                const nagariRes = jsToNagari({
                    writeHead: (statusCode, headers) => res.writeHead(statusCode, headers),
                    end: (data) => res.end(data)
                });
                handler(nagariReq, nagariRes);
            });
        }
        catch {
            throw new Error('http module not available in this environment');
        }
    }, 'createServer'),
    // Process
    process: jsToNagari({
        argv: globalThis.process?.argv || [],
        env: globalThis.process?.env || {},
        cwd: () => globalThis.process?.cwd() || '/',
        exit: (code = 0) => globalThis.process?.exit(code)
    })
} : {};
/**
 * Console wrapper
 */
export const ConsoleInterop = {
    log: wrapJSFunction(console.log, 'log'),
    error: wrapJSFunction(console.error, 'error'),
    warn: wrapJSFunction(console.warn, 'warn'),
    info: wrapJSFunction(console.info, 'info'),
    debug: wrapJSFunction(console.debug, 'debug')
};
/**
 * Math wrapper
 */
export const MathInterop = createNagariModule(Math, 'Math');
/**
 * JSON wrapper
 */
export const JSONInterop = {
    parse: wrapJSFunction((text) => JSON.parse(text), 'parse'),
    stringify: wrapJSFunction((value, replacer, space) => JSON.stringify(nagariToJS(value), replacer, space), 'stringify')
};
/**
 * Promise utilities
 */
export const PromiseInterop = {
    resolve: wrapJSFunction((value) => Promise.resolve(jsToNagari(value)), 'resolve'),
    reject: wrapJSFunction((reason) => Promise.reject(jsToNagari(reason)), 'reject'),
    all: wrapJSFunction((promises) => Promise.all(promises.map(p => p.then(jsToNagari))), 'all'),
    race: wrapJSFunction((promises) => Promise.race(promises.map(p => p.then(jsToNagari))), 'race')
};
/**
 * HTTP module for Nagari
 */
export const HTTPInterop = {
    get: async (url, headers) => {
        try {
            const response = await fetch(url, {
                method: 'GET',
                headers: headers || {}
            });
            const body = await response.text();
            const responseObj = {
                status: response.status,
                statusText: response.statusText,
                headers: Object.fromEntries(response.headers.entries()),
                body: body,
                json: () => {
                    try {
                        return JSON.parse(body);
                    }
                    catch {
                        throw new Error('Response is not valid JSON');
                    }
                },
                get: (key, defaultValue) => {
                    try {
                        const data = JSON.parse(body);
                        return data[key] !== undefined ? data[key] : defaultValue;
                    }
                    catch {
                        return defaultValue;
                    }
                }
            };
            // Don't use jsToNagari on objects with functions
            return responseObj;
        }
        catch (error) {
            throw new Error(`HTTP GET failed: ${error?.message || 'Unknown error'}`);
        }
    },
    post: async (url, data, headers) => {
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...(headers || {})
                },
                body: typeof data === 'string' ? data : JSON.stringify(data)
            });
            const body = await response.text();
            const responseObj = {
                status: response.status,
                statusText: response.statusText,
                headers: Object.fromEntries(response.headers.entries()),
                body: body,
                json: () => {
                    try {
                        return JSON.parse(body);
                    }
                    catch {
                        throw new Error('Response is not valid JSON');
                    }
                },
                get: (key, defaultValue) => {
                    try {
                        const data = JSON.parse(body);
                        return data[key] !== undefined ? data[key] : defaultValue;
                    }
                    catch {
                        return defaultValue;
                    }
                }
            };
            // Don't use jsToNagari on objects with functions
            return responseObj;
        }
        catch (error) {
            throw new Error(`HTTP POST failed: ${error?.message || 'Unknown error'}`);
        }
    },
    put: async (url, data, headers) => {
        try {
            const response = await fetch(url, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    ...(headers || {})
                },
                body: typeof data === 'string' ? data : JSON.stringify(data)
            });
            const body = await response.text();
            return {
                status: response.status,
                statusText: response.statusText,
                headers: Object.fromEntries(response.headers.entries()),
                body: body,
                json: () => {
                    try {
                        return JSON.parse(body);
                    }
                    catch {
                        throw new Error('Response is not valid JSON');
                    }
                },
                get: (key, defaultValue) => {
                    try {
                        const data = JSON.parse(body);
                        return data[key] !== undefined ? data[key] : defaultValue;
                    }
                    catch {
                        return defaultValue;
                    }
                }
            };
        }
        catch (error) {
            throw new Error(`HTTP PUT failed: ${error?.message || 'Unknown error'}`);
        }
    },
    delete: async (url, headers) => {
        try {
            const response = await fetch(url, {
                method: 'DELETE',
                headers: headers || {}
            });
            const body = await response.text();
            return {
                status: response.status,
                statusText: response.statusText,
                headers: Object.fromEntries(response.headers.entries()),
                body: body,
                json: () => {
                    try {
                        return JSON.parse(body);
                    }
                    catch {
                        throw new Error('Response is not valid JSON');
                    }
                },
                get: (key, defaultValue) => {
                    try {
                        const data = JSON.parse(body);
                        return data[key] !== undefined ? data[key] : defaultValue;
                    }
                    catch {
                        return defaultValue;
                    }
                }
            };
        }
        catch (error) {
            throw new Error(`HTTP DELETE failed: ${error?.message || 'Unknown error'}`);
        }
    },
    parseUrl: wrapJSFunction((url) => {
        try {
            const parsed = new URL(url);
            return jsToNagari({
                protocol: parsed.protocol,
                hostname: parsed.hostname,
                port: parsed.port,
                pathname: parsed.pathname,
                search: parsed.search,
                hash: parsed.hash,
                href: parsed.href
            });
        }
        catch (error) {
            throw new Error(`Invalid URL: ${error?.message || 'Unknown error'}`);
        }
    }, 'parseUrl')
};
/**
 * Global interop registry
 */
export class InteropRegistry {
    static registerModule(name, module) {
        this.modules.set(name, module);
    }
    static getModule(name) {
        return this.modules.get(name);
    }
    static registerGlobal(name, value) {
        this.globals.set(name, value);
    }
    static getGlobal(name) {
        return this.globals.get(name);
    }
    static initialize() {
        // Register built-in modules
        this.registerModule('console', ConsoleInterop);
        this.registerModule('Math', MathInterop);
        this.registerModule('JSON', JSONInterop);
        this.registerModule('Promise', PromiseInterop);
        this.registerModule('http', HTTPInterop);
        if (typeof document !== 'undefined') {
            this.registerModule('DOM', DOMInterop);
        }
        if (globalThis.process !== undefined) {
            this.registerModule('Node', NodeInterop);
        }
        // Register global functions
        this.registerGlobal('setTimeout', DOMInterop.setTimeout);
        this.registerGlobal('setInterval', DOMInterop.setInterval);
        this.registerGlobal('clearTimeout', DOMInterop.clearTimeout);
        this.registerGlobal('clearInterval', DOMInterop.clearInterval);
        this.registerGlobal('fetch', DOMInterop.fetch);
    }
}
InteropRegistry.modules = new Map();
InteropRegistry.globals = new Map();
// Initialize the registry
if (typeof globalThis !== 'undefined') {
    InteropRegistry.initialize();
}
//# sourceMappingURL=interop.js.map