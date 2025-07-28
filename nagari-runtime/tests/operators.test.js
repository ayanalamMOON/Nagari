// Tests for compound assignment operators

import { describe, expect, it } from '@jest/globals';
import {
    CompoundAssignmentOperators,
    addAssign,
    divideAssign,
    moduloAssign,
    multiplyAssign,
    powerAssign,
    subtractAssign
} from '../dist/operators.js';

describe('Compound Assignment Operators', () => {
    describe('Addition Assignment (+=)', () => {
        it('should add numbers', () => {
            expect(addAssign(5, 3)).toBe(8);
            expect(addAssign(10.5, 2.5)).toBe(13);
        });

        it('should concatenate strings', () => {
            expect(addAssign('hello', ' world')).toBe('hello world');
            expect(addAssign('test', 123)).toBe('test123');
        });

        it('should concatenate arrays', () => {
            expect(addAssign([1, 2], [3, 4])).toEqual([1, 2, 3, 4]);
        });

        it('should handle numeric coercion', () => {
            expect(addAssign('5', '3')).toBe(8);
            expect(addAssign('10', 5)).toBe(15);
        });
    });

    describe('Subtraction Assignment (-=)', () => {
        it('should subtract numbers', () => {
            expect(subtractAssign(10, 3)).toBe(7);
            expect(subtractAssign(5.5, 2.5)).toBe(3);
        });

        it('should handle string numbers', () => {
            expect(subtractAssign('10', '3')).toBe(7);
        });

        it('should throw on invalid types', () => {
            expect(() => subtractAssign('hello', 'world')).toThrow();
        });
    });

    describe('Multiplication Assignment (*=)', () => {
        it('should multiply numbers', () => {
            expect(multiplyAssign(4, 3)).toBe(12);
            expect(multiplyAssign(2.5, 4)).toBe(10);
        });

        it('should repeat strings', () => {
            expect(multiplyAssign('abc', 3)).toBe('abcabcabc');
            expect(multiplyAssign('x', 0)).toBe('');
        });

        it('should repeat arrays', () => {
            expect(multiplyAssign([1, 2], 3)).toEqual([1, 2, 1, 2, 1, 2]);
            expect(multiplyAssign(['a'], 2)).toEqual(['a', 'a']);
        });
    });

    describe('Division Assignment (/=)', () => {
        it('should divide numbers', () => {
            expect(divideAssign(12, 3)).toBe(4);
            expect(divideAssign(10, 4)).toBe(2.5);
        });

        it('should throw on division by zero', () => {
            expect(() => divideAssign(5, 0)).toThrow('Division by zero');
        });

        it('should handle string numbers', () => {
            expect(divideAssign('20', '4')).toBe(5);
        });
    });

    describe('Modulo Assignment (%=)', () => {
        it('should calculate modulo', () => {
            expect(moduloAssign(10, 3)).toBe(1);
            expect(moduloAssign(15, 4)).toBe(3);
        });

        it('should throw on modulo by zero', () => {
            expect(() => moduloAssign(5, 0)).toThrow('Modulo by zero');
        });
    });

    describe('Power Assignment (**=)', () => {
        it('should calculate power', () => {
            expect(powerAssign(2, 3)).toBe(8);
            expect(powerAssign(5, 2)).toBe(25);
        });

        it('should handle fractional powers', () => {
            expect(powerAssign(9, 0.5)).toBe(3);
        });
    });

    describe('Class methods', () => {
        it('should provide static methods', () => {
            expect(CompoundAssignmentOperators.addAssign(1, 2)).toBe(3);
            expect(CompoundAssignmentOperators.subtractAssign(5, 2)).toBe(3);
            expect(CompoundAssignmentOperators.multiplyAssign(3, 4)).toBe(12);
            expect(CompoundAssignmentOperators.divideAssign(15, 3)).toBe(5);
        });
    });
});
