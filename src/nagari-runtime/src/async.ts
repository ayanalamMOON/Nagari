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
