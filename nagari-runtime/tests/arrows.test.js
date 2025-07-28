// Tests for arrow function support

import { describe, expect, it, jest } from '@jest/globals';
import {
    ArrowFunctionSupport,
    createArrow,
    createAsyncArrow,
    curry,
    debounce,
    memoize,
    throttle
} from '../dist/arrows.js';

describe('Arrow Function Support', () => {
    describe('createAsyncArrow', () => {
        it('should create async arrow function', async () => {
            const asyncFn = createAsyncArrow(async (x, y) => x + y);
            const result = await asyncFn(3, 4);
            expect(result).toBe(7);
        });

        it('should handle errors in async arrow functions', async () => {
            const errorFn = createAsyncArrow(async () => {
                throw new Error('Test error');
            });

            await expect(errorFn()).rejects.toThrow('Async arrow function error');
        });

        it('should preserve function name', () => {
            const namedFn = createAsyncArrow(async function testFunction() {
                return 'test';
            });
            expect(namedFn.name).toBe('testFunction');
        });
    });

    describe('createArrow', () => {
        it('should create regular arrow function', () => {
            const arrowFn = createArrow((x, y) => x * y);
            const result = arrowFn(3, 4);
            expect(result).toBe(12);
        });

        it('should handle errors in arrow functions', () => {
            const errorFn = createArrow(() => {
                throw new Error('Test error');
            });

            expect(() => errorFn()).toThrow('Arrow function error');
        });

        it('should bind context when provided', () => {
            const context = { value: 10 };
            const arrowFn = createArrow(function () {
                return this.value;
            }, context);

            expect(arrowFn()).toBe(10);
        });
    });

    describe('curry', () => {
        it('should curry a function', () => {
            const add = (a, b, c) => a + b + c;
            const curriedAdd = curry(add);

            expect(curriedAdd(1)(2)(3)).toBe(6);
            expect(curriedAdd(1, 2)(3)).toBe(6);
            expect(curriedAdd(1)(2, 3)).toBe(6);
            expect(curriedAdd(1, 2, 3)).toBe(6);
        });

        it('should respect custom arity', () => {
            const fn = (...args) => args.length;
            const curried = curry(fn, 2);

            expect(typeof curried(1)).toBe('function');
            expect(curried(1, 2)).toBe(2);
        });
    });

    describe('memoize', () => {
        it('should memoize function results', () => {
            let callCount = 0;
            const expensiveFn = (x) => {
                callCount++;
                return x * x;
            };

            const memoized = memoize(expensiveFn);

            expect(memoized(5)).toBe(25);
            expect(memoized(5)).toBe(25);
            expect(callCount).toBe(1);

            expect(memoized(3)).toBe(9);
            expect(callCount).toBe(2);
        });

        it('should use custom key generator', () => {
            let callCount = 0;
            const fn = (obj) => {
                callCount++;
                return obj.value;
            };

            const memoized = memoize(fn, (obj) => obj.id);

            expect(memoized({ id: 1, value: 10 })).toBe(10);
            expect(memoized({ id: 1, value: 20 })).toBe(10); // Same id, cached result
            expect(callCount).toBe(1);
        });
    });

    describe('throttle', () => {
        beforeEach(() => {
            jest.useFakeTimers();
        });

        afterEach(() => {
            jest.useRealTimers();
        });

        it('should throttle function calls', () => {
            let callCount = 0;
            const fn = () => ++callCount;
            const throttled = throttle(fn, 100);

            throttled();
            throttled();
            throttled();

            expect(callCount).toBe(1);

            jest.advanceTimersByTime(150);
            throttled();
            expect(callCount).toBe(2);
        });
    });

    describe('debounce', () => {
        beforeEach(() => {
            jest.useFakeTimers();
        });

        afterEach(() => {
            jest.useRealTimers();
        });

        it('should debounce function calls', () => {
            let callCount = 0;
            const fn = () => ++callCount;
            const debounced = debounce(fn, 100);

            debounced();
            debounced();
            debounced();

            expect(callCount).toBe(0);

            jest.advanceTimersByTime(150);
            expect(callCount).toBe(1);
        });
    });

    describe('Class methods', () => {
        it('should provide static methods', () => {
            const asyncFn = ArrowFunctionSupport.createAsyncArrow(async (x) => x);
            const regularFn = ArrowFunctionSupport.createArrow((x) => x);

            expect(typeof asyncFn).toBe('function');
            expect(typeof regularFn).toBe('function');
        });
    });
});
