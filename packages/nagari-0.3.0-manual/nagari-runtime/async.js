// Async utilities for Nagari
export class AsyncContext {
    constructor() {
        this.pendingTasks = new Set();
    }
    static getInstance() {
        if (!AsyncContext.instance) {
            AsyncContext.instance = new AsyncContext();
        }
        return AsyncContext.instance;
    }
    trackPromise(promise) {
        this.pendingTasks.add(promise);
        promise.finally(() => {
            this.pendingTasks.delete(promise);
        });
        return promise;
    }
    async waitForAll() {
        while (this.pendingTasks.size > 0) {
            await Promise.all(Array.from(this.pendingTasks));
        }
    }
    getPendingCount() {
        return this.pendingTasks.size;
    }
}
// Enhanced async/await support
export async function nagariAwait(value) {
    if (value instanceof Promise) {
        return AsyncContext.getInstance().trackPromise(value);
    }
    return value;
}
// Sleep function for both sync and async contexts
export function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms * 1000));
}
// Async timeout wrapper
export function withTimeout(promise, timeoutMs, timeoutMessage = 'Operation timed out') {
    const timeoutPromise = new Promise((_, reject) => setTimeout(() => reject(new Error(timeoutMessage)), timeoutMs));
    return Promise.race([promise, timeoutPromise]);
}
// Retry mechanism for async operations
export async function retry(fn, maxAttempts = 3, delayMs = 1000) {
    let lastError;
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        try {
            return await fn();
        }
        catch (error) {
            lastError = error;
            if (attempt === maxAttempts)
                break;
            await sleep(delayMs / 1000);
        }
    }
    throw lastError;
}
// Enhanced async arrow function support
export class AsyncArrowContext {
    /**
     * Create an async arrow function with context tracking
     */
    static create(fn, options) {
        const context = AsyncContext.getInstance();
        const asyncArrow = async (...args) => {
            const promise = Promise.resolve().then(() => fn(...args));
            // Apply timeout if specified
            const finalPromise = options?.timeout
                ? withTimeout(promise, options.timeout)
                : promise;
            // Apply retries if specified
            const retriedPromise = options?.retries
                ? retry(() => finalPromise, options.retries)
                : finalPromise;
            // Track the promise
            return context.trackPromise(retriedPromise.catch((error) => {
                if (options?.onError) {
                    options.onError(error);
                }
                throw error;
            }));
        };
        // Store context reference
        this.contexts.set(asyncArrow, context);
        return asyncArrow;
    }
    /**
     * Get the async context for an arrow function
     */
    static getContext(fn) {
        return this.contexts.get(fn);
    }
}
AsyncArrowContext.contexts = new WeakMap();
// Export async arrow function utilities
export const createAsyncArrowWithContext = AsyncArrowContext.create;
export const getAsyncContext = AsyncArrowContext.getContext;
//# sourceMappingURL=async.js.map