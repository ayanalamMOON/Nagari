export declare class AsyncContext {
    private static instance;
    private pendingTasks;
    static getInstance(): AsyncContext;
    trackPromise<T>(promise: Promise<T>): Promise<T>;
    waitForAll(): Promise<void>;
    getPendingCount(): number;
}
export declare function nagariAwait<T>(value: T | Promise<T>): Promise<T>;
export declare function sleep(ms: number): Promise<void>;
export declare function withTimeout<T>(promise: Promise<T>, timeoutMs: number, timeoutMessage?: string): Promise<T>;
export declare function retry<T>(fn: () => Promise<T>, maxAttempts?: number, delayMs?: number): Promise<T>;
export declare class AsyncArrowContext {
    private static contexts;
    /**
     * Create an async arrow function with context tracking
     */
    static create<T extends any[], R>(fn: (...args: T) => Promise<R>, options?: {
        timeout?: number;
        retries?: number;
        onError?: (error: Error) => void;
    }): (...args: T) => Promise<R>;
    /**
     * Get the async context for an arrow function
     */
    static getContext(fn: Function): AsyncContext | undefined;
}
export declare const createAsyncArrowWithContext: typeof AsyncArrowContext.create;
export declare const getAsyncContext: typeof AsyncArrowContext.getContext;
//# sourceMappingURL=async.d.ts.map