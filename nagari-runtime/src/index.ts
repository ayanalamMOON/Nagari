// Nagari Runtime - Core utilities and polyfills

export * from './async.js';
export * from './builtins.js';
export * from './interop.js';
export * from './jsx.js';
export * from './types.js';

import { InteropRegistry } from './interop.js';
import { ReactInterop } from './jsx.js';

// Global runtime initialization
if (typeof globalThis !== 'undefined') {
    // Initialize Nagari runtime globals
    (globalThis as any).__nagari__ = {
        version: '0.1.0',
        runtime: 'js',
        interop: InteropRegistry
    };

    // Initialize interop registry
    InteropRegistry.initialize();

    // Register React interop if React is available
    if ((globalThis as any).React) {
        InteropRegistry.registerModule('React', ReactInterop as any);
    }
}
