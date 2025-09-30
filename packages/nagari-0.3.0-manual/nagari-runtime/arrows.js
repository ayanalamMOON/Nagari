// Arrow function utilities and runtime support for Nagari
/**
 * Arrow function helper utilities
 * Provides runtime support for arrow functions including async arrow functions
 */
export class ArrowFunctionSupport {
    /**
     * Create an async arrow function with proper context binding
     */
    static createAsyncArrow(fn, context) {
        const asyncFn = async (...args) => {
            try {
                return await fn.apply(context, args);
            }
            catch (error) {
                throw new Error(`Async arrow function error: ${error}`);
            }
        };
        // Preserve function properties
        Object.defineProperty(asyncFn, 'name', {
            value: fn.name || 'asyncArrow',
            configurable: true
        });
        return asyncFn;
    }
    /**
     * Create a regular arrow function with proper context binding
     */
    static createArrow(fn, context) {
        const arrowFn = (...args) => {
            try {
                return fn.apply(context, args);
            }
            catch (error) {
                throw new Error(`Arrow function error: ${error}`);
            }
        };
        // Preserve function properties
        Object.defineProperty(arrowFn, 'name', {
            value: fn.name || 'arrow',
            configurable: true
        });
        return arrowFn;
    }
    /**
     * Validate arrow function parameters
     */
    static validateParams(params, expected) {
        if (params.length !== expected) {
            throw new Error(`Arrow function expects ${expected} parameters, got ${params.length}`);
        }
    }
    /**
     * Create a curried arrow function
     */
    static curry(fn, arity) {
        const expectedArity = arity || fn.length;
        return function curried(...args) {
            if (args.length >= expectedArity) {
                return fn(...args);
            }
            return (...nextArgs) => curried(...args, ...nextArgs);
        };
    }
    /**
     * Create a memoized arrow function
     */
    static memoize(fn, keyGenerator) {
        const cache = new Map();
        const defaultKeyGen = (...args) => JSON.stringify(args);
        const getKey = keyGenerator || defaultKeyGen;
        return (...args) => {
            const key = getKey(...args);
            if (cache.has(key)) {
                return cache.get(key);
            }
            const result = fn(...args);
            cache.set(key, result);
            return result;
        };
    }
    /**
     * Create a throttled arrow function
     */
    static throttle(fn, delay) {
        let lastCall = 0;
        let lastResult;
        return (...args) => {
            const now = Date.now();
            if (now - lastCall >= delay) {
                lastCall = now;
                lastResult = fn(...args);
                return lastResult;
            }
            return lastResult;
        };
    }
    /**
     * Create a debounced arrow function
     */
    static debounce(fn, delay) {
        let timeoutId;
        return (...args) => {
            clearTimeout(timeoutId);
            timeoutId = setTimeout(() => fn(...args), delay);
        };
    }
}
// Export convenience functions
export const createAsyncArrow = ArrowFunctionSupport.createAsyncArrow;
export const createArrow = ArrowFunctionSupport.createArrow;
export const validateParams = ArrowFunctionSupport.validateParams;
export const curry = ArrowFunctionSupport.curry;
export const memoize = ArrowFunctionSupport.memoize;
export const throttle = ArrowFunctionSupport.throttle;
export const debounce = ArrowFunctionSupport.debounce;
// Global registration for runtime use
if (typeof globalThis !== 'undefined') {
    globalThis.__nagari_arrows__ = ArrowFunctionSupport;
}
//# sourceMappingURL=arrows.js.map