// Arrow function utilities and runtime support for Nagari

/**
 * Arrow function helper utilities
 * Provides runtime support for arrow functions including async arrow functions
 */

export class ArrowFunctionSupport {
    /**
     * Create an async arrow function with proper context binding
     */
    static createAsyncArrow<T extends any[], R>(
        fn: (...args: T) => Promise<R>,
        context?: any
    ): (...args: T) => Promise<R> {
        const asyncFn = async (...args: T): Promise<R> => {
            try {
                return await fn.apply(context, args);
            } catch (error) {
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
    static createArrow<T extends any[], R>(
        fn: (...args: T) => R,
        context?: any
    ): (...args: T) => R {
        const arrowFn = (...args: T): R => {
            try {
                return fn.apply(context, args);
            } catch (error) {
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
    static validateParams(params: any[], expected: number): void {
        if (params.length !== expected) {
            throw new Error(`Arrow function expects ${expected} parameters, got ${params.length}`);
        }
    }

    /**
     * Create a curried arrow function
     */
    static curry<T extends any[], R>(
        fn: (...args: T) => R,
        arity?: number
    ): (...args: any[]) => any {
        const expectedArity = arity || fn.length;

        return function curried(...args: any[]): any {
            if (args.length >= expectedArity) {
                return fn(...args as T);
            }
            return (...nextArgs: any[]) => curried(...args, ...nextArgs);
        };
    }

    /**
     * Create a memoized arrow function
     */
    static memoize<T extends any[], R>(
        fn: (...args: T) => R,
        keyGenerator?: (...args: T) => string
    ): (...args: T) => R {
        const cache = new Map<string, R>();
        const defaultKeyGen = (...args: T) => JSON.stringify(args);
        const getKey = keyGenerator || defaultKeyGen;

        return (...args: T): R => {
            const key = getKey(...args);
            if (cache.has(key)) {
                return cache.get(key)!;
            }
            const result = fn(...args);
            cache.set(key, result);
            return result;
        };
    }

    /**
     * Create a throttled arrow function
     */
    static throttle<T extends any[], R>(
        fn: (...args: T) => R,
        delay: number
    ): (...args: T) => R | undefined {
        let lastCall = 0;
        let lastResult: R;

        return (...args: T): R | undefined => {
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
    static debounce<T extends any[], R>(
        fn: (...args: T) => R,
        delay: number
    ): (...args: T) => void {
        let timeoutId: NodeJS.Timeout | number;

        return (...args: T): void => {
            clearTimeout(timeoutId as NodeJS.Timeout);
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
    (globalThis as any).__nagari_arrows__ = ArrowFunctionSupport;
}
