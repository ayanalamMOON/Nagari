// Nagari Runtime - Core utilities and polyfills

export * from './arrows.js';
export * from './async.js';
export * from './builtins.js';
export * from './interop.js';
export * from './jsx.js';
export * from './operators.js';
export * from './types.js';

import { ArrowFunctionSupport } from './arrows.js';
import { InteropRegistry } from './interop.js';
import { ReactInterop } from './jsx.js';
import { CompoundAssignmentOperators } from './operators.js';

// Global runtime initialization
if (typeof globalThis !== 'undefined') {
    // Initialize Nagari runtime globals
    (globalThis as any).__nagari__ = {
        version: '0.5.0',
        runtime: 'js',
        interop: InteropRegistry,
        operators: CompoundAssignmentOperators,
        arrows: ArrowFunctionSupport,
        features: {
            formatSpecifiers: true,
            percentageFormatting: true,
            pythonCompatibility: true,
            compoundAssignments: true,
            asyncArrowFunctions: true,
            arrowFunctions: true
        }
    };

    // Initialize interop registry
    InteropRegistry.initialize();

    // Register React interop if React is available
    if ((globalThis as any).React) {
        InteropRegistry.registerModule('React', ReactInterop as any);
    }
}
