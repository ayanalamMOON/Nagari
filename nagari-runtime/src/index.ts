// Nagari Runtime - Core utilities and polyfills

export * from './async';
export * from './builtins';
export * from './interop';
export * from './jsx';
export * from './types';

import { InteropRegistry } from './interop';
import { ReactInterop } from './jsx';

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
