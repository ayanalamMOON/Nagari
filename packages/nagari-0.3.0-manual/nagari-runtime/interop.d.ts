import { NagariClass, NagariFunction, NagariModule, NagariValue } from './types.js';
/**
 * JavaScript to Nagari value conversion
 */
export declare function jsToNagari(value: any): NagariValue;
/**
 * Nagari to JavaScript value conversion
 */
export declare function nagariToJS(value: NagariValue): any;
/**
 * Wrap a JavaScript function to be callable from Nagari
 */
export declare function wrapJSFunction(fn: Function, name?: string): NagariFunction;
/**
 * Wrap a JavaScript class to be instantiable from Nagari
 */
export declare function wrapJSClass(jsClass: any, name?: string): NagariClass;
/**
 * Create a Nagari module from a JavaScript module/object
 */
export declare function createNagariModule(jsModule: any, name: string): NagariModule;
/**
 * Dynamic import wrapper for Nagari
 */
export declare function dynamicImport(modulePath: string): Promise<NagariModule>;
/**
 * DOM API wrappers
 */
export declare const DOMInterop: {
    getElementById: NagariFunction;
    querySelector: NagariFunction;
    querySelectorAll: NagariFunction;
    createElement: NagariFunction;
    setTimeout: NagariFunction;
    setInterval: NagariFunction;
    clearTimeout: NagariFunction;
    clearInterval: NagariFunction;
    fetch: NagariFunction;
};
/**
 * Node.js API wrappers (conditional)
 */
export declare const NodeInterop: {
    readFile: NagariFunction;
    writeFile: NagariFunction;
    createServer: NagariFunction;
    process: NagariValue;
} | {
    readFile?: undefined;
    writeFile?: undefined;
    createServer?: undefined;
    process?: undefined;
};
/**
 * Console wrapper
 */
export declare const ConsoleInterop: {
    log: NagariFunction;
    error: NagariFunction;
    warn: NagariFunction;
    info: NagariFunction;
    debug: NagariFunction;
};
/**
 * Math wrapper
 */
export declare const MathInterop: NagariModule;
/**
 * JSON wrapper
 */
export declare const JSONInterop: {
    parse: NagariFunction;
    stringify: NagariFunction;
};
/**
 * Promise utilities
 */
export declare const PromiseInterop: {
    resolve: NagariFunction;
    reject: NagariFunction;
    all: NagariFunction;
    race: NagariFunction;
};
/**
 * HTTP module for Nagari
 */
export declare const HTTPInterop: {
    get: (url: string, headers?: any) => Promise<{
        status: number;
        statusText: string;
        headers: {
            [k: string]: string;
        };
        body: string;
        json: () => any;
        get: (key: string, defaultValue?: any) => any;
    }>;
    post: (url: string, data?: any, headers?: any) => Promise<{
        status: number;
        statusText: string;
        headers: {
            [k: string]: string;
        };
        body: string;
        json: () => any;
        get: (key: string, defaultValue?: any) => any;
    }>;
    put: (url: string, data?: any, headers?: any) => Promise<{
        status: number;
        statusText: string;
        headers: {
            [k: string]: string;
        };
        body: string;
        json: () => any;
        get: (key: string, defaultValue?: any) => any;
    }>;
    delete: (url: string, headers?: any) => Promise<{
        status: number;
        statusText: string;
        headers: {
            [k: string]: string;
        };
        body: string;
        json: () => any;
        get: (key: string, defaultValue?: any) => any;
    }>;
    parseUrl: NagariFunction;
};
/**
 * Global interop registry
 */
export declare class InteropRegistry {
    private static modules;
    private static globals;
    static registerModule(name: string, module: NagariModule): void;
    static getModule(name: string): NagariModule | undefined;
    static registerGlobal(name: string, value: NagariValue): void;
    static getGlobal(name: string): NagariValue | undefined;
    static initialize(): void;
}
//# sourceMappingURL=interop.d.ts.map