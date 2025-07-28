// Async utilities for Nagari

export class AsyncContext {
    private static instance: AsyncContext;
    private pendingTasks: Set<Promise<any>> = new Set();

    static getInstance(): AsyncContext {
        if (!AsyncContext.instance) {
            AsyncContext.instance = new AsyncContext();
        }
        return AsyncContext.instance;
    }

    trackPromise<T>(promise: Promise<T>): Promise<T> {
        this.pendingTasks.add(promise);
        promise.finally(() => {
            this.pendingTasks.delete(promise);
        });
        return promise;
    }

    async waitForAll(): Promise<void> {
        while (this.pendingTasks.size > 0) {
            await Promise.all(Array.from(this.pendingTasks));
        }
    }

    getPendingCount(): number {
        return this.pendingTasks.size;
    }
}

// Enhanced async/await support
export async function nagariAwait<T>(value: T | Promise<T>): Promise<T> {
    if (value instanceof Promise) {
        return AsyncContext.getInstance().trackPromise(value);
    }
    return value;
}

// Sleep function for both sync and async contexts
export function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms * 1000));
}

// Async timeout wrapper
export function withTimeout<T>(
    promise: Promise<T>,
    timeoutMs: number,
    timeoutMessage = 'Operation timed out'
): Promise<T> {
    const timeoutPromise = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error(timeoutMessage)), timeoutMs)
    );

    return Promise.race([promise, timeoutPromise]);
}

// Retry mechanism for async operations
export async function retry<T>(
    fn: () => Promise<T>,
    maxAttempts: number = 3,
    delayMs: number = 1000
): Promise<T> {
    let lastError: Error;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        try {
            return await fn();
        } catch (error) {
            lastError = error as Error;
            if (attempt === maxAttempts) break;
            await sleep(delayMs / 1000);
        }
    }

    throw lastError!;
}

// Enhanced async arrow function support
export class AsyncArrowContext {
    private static contexts = new WeakMap<Function, AsyncContext>();

    /**
     * Create an async arrow function with context tracking
     */
    static create<T extends any[], R>(
        fn: (...args: T) => Promise<R>,
        options?: {
            timeout?: number;
            retries?: number;
            onError?: (error: Error) => void;
        }
    ): (...args: T) => Promise<R> {
        const context = AsyncContext.getInstance();

        const asyncArrow = async (...args: T): Promise<R> => {
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
            return context.trackPromise(
                retriedPromise.catch((error: any) => {
                    if (options?.onError) {
                        options.onError(error);
                    }
                    throw error;
                })
            );
        };

        // Store context reference
        this.contexts.set(asyncArrow, context);

        return asyncArrow;
    }

    /**
     * Get the async context for an arrow function
     */
    static getContext(fn: Function): AsyncContext | undefined {
        return this.contexts.get(fn);
    }
}

// Export async arrow function utilities
export const createAsyncArrowWithContext = AsyncArrowContext.create;
export const getAsyncContext = AsyncArrowContext.getContext;
