/**
 * Arrow function helper utilities
 * Provides runtime support for arrow functions including async arrow functions
 */
export declare class ArrowFunctionSupport {
    /**
     * Create an async arrow function with proper context binding
     */
    static createAsyncArrow<T extends any[], R>(fn: (...args: T) => Promise<R>, context?: any): (...args: T) => Promise<R>;
    /**
     * Create a regular arrow function with proper context binding
     */
    static createArrow<T extends any[], R>(fn: (...args: T) => R, context?: any): (...args: T) => R;
    /**
     * Validate arrow function parameters
     */
    static validateParams(params: any[], expected: number): void;
    /**
     * Create a curried arrow function
     */
    static curry<T extends any[], R>(fn: (...args: T) => R, arity?: number): (...args: any[]) => any;
    /**
     * Create a memoized arrow function
     */
    static memoize<T extends any[], R>(fn: (...args: T) => R, keyGenerator?: (...args: T) => string): (...args: T) => R;
    /**
     * Create a throttled arrow function
     */
    static throttle<T extends any[], R>(fn: (...args: T) => R, delay: number): (...args: T) => R | undefined;
    /**
     * Create a debounced arrow function
     */
    static debounce<T extends any[], R>(fn: (...args: T) => R, delay: number): (...args: T) => void;
}
export declare const createAsyncArrow: typeof ArrowFunctionSupport.createAsyncArrow;
export declare const createArrow: typeof ArrowFunctionSupport.createArrow;
export declare const validateParams: typeof ArrowFunctionSupport.validateParams;
export declare const curry: typeof ArrowFunctionSupport.curry;
export declare const memoize: typeof ArrowFunctionSupport.memoize;
export declare const throttle: typeof ArrowFunctionSupport.throttle;
export declare const debounce: typeof ArrowFunctionSupport.debounce;
//# sourceMappingURL=arrows.d.ts.map