// Nagari <-> JavaScript Interoperability Layer

import { NagariClass, NagariFunction, NagariModule, NagariValue } from './types.js';

/**
 * JavaScript to Nagari value conversion
 */
export function jsToNagari(value: any): NagariValue {
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
        const result: { [key: string]: NagariValue } = {};
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
export function nagariToJS(value: NagariValue): any {
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
        if ((value as any).__nagari_function__) {
            return async (...args: any[]) => {
                const result = (value as NagariFunction)(...args.map(jsToNagari));
                return nagariToJS(await result);
            };
        }
        return value;
    }

    if (typeof value === 'object' && value !== null) {
        const result: any = {};
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
export function wrapJSFunction(fn: Function, name?: string): NagariFunction {
    const wrapped = (...args: NagariValue[]): any => {
        const jsArgs = args.map(nagariToJS);
        const result = fn(...jsArgs);
        return jsToNagari(result);
    };

    (wrapped as any).__nagari_function__ = true as const;

    // Set function name using defineProperty to avoid read-only issues
    try {
        Object.defineProperty(wrapped, 'name', {
            value: name || fn.name,
            configurable: true
        });
    } catch (e) {
        // Ignore if we can't set the name
    }

    wrapped.arity = fn.length;

    return wrapped as NagariFunction;
}

/**
 * Wrap a JavaScript class to be instantiable from Nagari
 */
export function wrapJSClass(jsClass: any, name?: string): NagariClass {
    const wrapped = function (...args: NagariValue[]) {
        const jsArgs = args.map(nagariToJS);
        const instance = new jsClass(...jsArgs);
        return jsToNagari(instance);
    } as any;

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
export function createNagariModule(jsModule: any, name: string): NagariModule {
    const module: NagariModule = {
        __nagari_module__: true,
        name
    };

    for (const [key, value] of Object.entries(jsModule)) {
        if (typeof value === 'function') {
            // Check if it's a constructor function (class)
            if (value.prototype && value.prototype.constructor === value) {
                module[key] = wrapJSClass(value, key);
            } else {
                module[key] = wrapJSFunction(value, key);
            }
        } else {
            module[key] = jsToNagari(value);
        }
    }

    return module;
}

/**
 * Dynamic import wrapper for Nagari
 */
export async function dynamicImport(modulePath: string): Promise<NagariModule> {
    try {
        const jsModule = await import(modulePath);
        return createNagariModule(jsModule, modulePath);
    } catch (error) {
        throw new Error(`Failed to import module '${modulePath}': ${error}`);
    }
}

/**
 * DOM API wrappers
 */
export const DOMInterop = {
    // Document methods
    getElementById: wrapJSFunction(
        (id: string) => document.getElementById(id),
        'getElementById'
    ),

    querySelector: wrapJSFunction(
        (selector: string) => document.querySelector(selector),
        'querySelector'
    ),

    querySelectorAll: wrapJSFunction(
        (selector: string) => Array.from(document.querySelectorAll(selector)),
        'querySelectorAll'
    ),

    createElement: wrapJSFunction(
        (tagName: string) => document.createElement(tagName),
        'createElement'
    ),

    // Window methods
    setTimeout: wrapJSFunction(
        (callback: Function, delay: number) => setTimeout(() => callback(), delay),
        'setTimeout'
    ),

    setInterval: wrapJSFunction(
        (callback: Function, delay: number) => setInterval(() => callback(), delay),
        'setInterval'
    ),

    clearTimeout: wrapJSFunction(clearTimeout, 'clearTimeout'),
    clearInterval: wrapJSFunction(clearInterval, 'clearInterval'),

    // Fetch API
    fetch: wrapJSFunction(
        async (url: string, options?: any) => {
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
        },
        'fetch'
    )
};

/**
 * Node.js API wrappers (conditional)
 */
export const NodeInterop = typeof globalThis !== 'undefined' &&
    (globalThis as any).process !== undefined ? {    // File system
    readFile: wrapJSFunction(
        async (path: string, encoding: string = 'utf8') => {
            try {
                const fs = await import('node:fs/promises');
                return fs.readFile(path, encoding as any);
            } catch {
                throw new Error('fs module not available in this environment');
            }
        },
        'readFile'
    ),

    writeFile: wrapJSFunction(
        async (path: string, data: string, encoding: string = 'utf8') => {
            try {
                const fs = await import('node:fs/promises');
                return fs.writeFile(path, data, encoding as any);
            } catch {
                throw new Error('fs module not available in this environment');
            }
        },
        'writeFile'
    ),

    // HTTP
    createServer: wrapJSFunction(
        async (handler: Function) => {
            try {
                const http = await import('node:http');
                return http.createServer((req: any, res: any) => {
                    const nagariReq = jsToNagari({
                        method: req.method,
                        url: req.url,
                        headers: req.headers
                    });
                    const nagariRes = jsToNagari({
                        writeHead: (statusCode: number, headers?: any) => res.writeHead(statusCode, headers),
                        end: (data?: string) => res.end(data)
                    });
                    handler(nagariReq, nagariRes);
                });
            } catch {
                throw new Error('http module not available in this environment');
            }
        },
        'createServer'
    ),

    // Process
    process: jsToNagari({
        argv: (globalThis as any).process?.argv || [],
        env: (globalThis as any).process?.env || {},
        cwd: () => (globalThis as any).process?.cwd() || '/',
        exit: (code: number = 0) => (globalThis as any).process?.exit(code)
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
    parse: wrapJSFunction((text: string) => JSON.parse(text), 'parse'),
    stringify: wrapJSFunction((value: any, replacer?: any, space?: any) =>
        JSON.stringify(nagariToJS(value), replacer, space), 'stringify')
};

/**
 * Promise utilities
 */
export const PromiseInterop = {
    resolve: wrapJSFunction((value: any) => Promise.resolve(jsToNagari(value)), 'resolve'),
    reject: wrapJSFunction((reason: any) => Promise.reject(jsToNagari(reason)), 'reject'),
    all: wrapJSFunction((promises: Promise<any>[]) =>
        Promise.all(promises.map(p => p.then(jsToNagari))), 'all'),
    race: wrapJSFunction((promises: Promise<any>[]) =>
        Promise.race(promises.map(p => p.then(jsToNagari))), 'race')
};

/**
 * Global interop registry
 */
export class InteropRegistry {
    private static modules: Map<string, NagariModule> = new Map();
    private static globals: Map<string, NagariValue> = new Map();

    static registerModule(name: string, module: NagariModule): void {
        this.modules.set(name, module);
    }

    static getModule(name: string): NagariModule | undefined {
        return this.modules.get(name);
    }

    static registerGlobal(name: string, value: NagariValue): void {
        this.globals.set(name, value);
    }

    static getGlobal(name: string): NagariValue | undefined {
        return this.globals.get(name);
    }

    static initialize(): void {
        // Register built-in modules
        this.registerModule('console', ConsoleInterop as any);
        this.registerModule('Math', MathInterop);
        this.registerModule('JSON', JSONInterop as any);
        this.registerModule('Promise', PromiseInterop as any);

        if (typeof document !== 'undefined') {
            this.registerModule('DOM', DOMInterop as any);
        }
        if ((globalThis as any).process !== undefined) {
            this.registerModule('Node', NodeInterop as any);
        }

        // Register global functions
        this.registerGlobal('setTimeout', DOMInterop.setTimeout);
        this.registerGlobal('setInterval', DOMInterop.setInterval);
        this.registerGlobal('clearTimeout', DOMInterop.clearTimeout);
        this.registerGlobal('clearInterval', DOMInterop.clearInterval);
        this.registerGlobal('fetch', DOMInterop.fetch);
    }
}

// Initialize the registry
if (typeof globalThis !== 'undefined') {
    InteropRegistry.initialize();
}
