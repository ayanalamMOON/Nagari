// Compound assignment operators for Nagari runtime
/**
 * Compound assignment operator utilities
 * These functions provide safe implementations of compound assignment operators
 * with proper type checking and error handling
 */
export class CompoundAssignmentOperators {
    /**
     * Addition assignment: target += value
     */
    static addAssign(target, value) {
        if (typeof target === 'number' && typeof value === 'number') {
            return target + value;
        }
        if (Array.isArray(target) && Array.isArray(value)) {
            return target.concat(value);
        }
        // Try numeric coercion first for numeric strings
        const numTarget = Number(target);
        const numValue = Number(value);
        if (!isNaN(numTarget) && !isNaN(numValue)) {
            return numTarget + numValue;
        }
        // Fall back to string concatenation
        if (typeof target === 'string' || typeof value === 'string') {
            return String(target) + String(value);
        }
        throw new Error(`Cannot perform += on types ${typeof target} and ${typeof value}`);
    }
    /**
     * Subtraction assignment: target -= value
     */
    static subtractAssign(target, value) {
        const numTarget = Number(target);
        const numValue = Number(value);
        if (isNaN(numTarget) || isNaN(numValue)) {
            throw new Error(`Cannot perform -= on types ${typeof target} and ${typeof value}`);
        }
        return numTarget - numValue;
    }
    /**
     * Multiplication assignment: target *= value
     */
    static multiplyAssign(target, value) {
        if (typeof target === 'string' && typeof value === 'number') {
            return target.repeat(Math.max(0, Math.floor(value)));
        }
        if (Array.isArray(target) && typeof value === 'number') {
            const result = [];
            for (let i = 0; i < Math.max(0, Math.floor(value)); i++) {
                result.push(...target);
            }
            return result;
        }
        const numTarget = Number(target);
        const numValue = Number(value);
        if (isNaN(numTarget) || isNaN(numValue)) {
            throw new Error(`Cannot perform *= on types ${typeof target} and ${typeof value}`);
        }
        return numTarget * numValue;
    }
    /**
     * Division assignment: target /= value
     */
    static divideAssign(target, value) {
        const numTarget = Number(target);
        const numValue = Number(value);
        if (isNaN(numTarget) || isNaN(numValue)) {
            throw new Error(`Cannot perform /= on types ${typeof target} and ${typeof value}`);
        }
        if (numValue === 0) {
            throw new Error('Division by zero');
        }
        return numTarget / numValue;
    }
    /**
     * Modulo assignment: target %= value
     */
    static moduloAssign(target, value) {
        const numTarget = Number(target);
        const numValue = Number(value);
        if (isNaN(numTarget) || isNaN(numValue)) {
            throw new Error(`Cannot perform %= on types ${typeof target} and ${typeof value}`);
        }
        if (numValue === 0) {
            throw new Error('Modulo by zero');
        }
        return numTarget % numValue;
    }
    /**
     * Power assignment: target **= value
     */
    static powerAssign(target, value) {
        const numTarget = Number(target);
        const numValue = Number(value);
        if (isNaN(numTarget) || isNaN(numValue)) {
            throw new Error(`Cannot perform **= on types ${typeof target} and ${typeof value}`);
        }
        return Math.pow(numTarget, numValue);
    }
}
// Export convenience functions for direct use
export const addAssign = CompoundAssignmentOperators.addAssign;
export const subtractAssign = CompoundAssignmentOperators.subtractAssign;
export const multiplyAssign = CompoundAssignmentOperators.multiplyAssign;
export const divideAssign = CompoundAssignmentOperators.divideAssign;
export const moduloAssign = CompoundAssignmentOperators.moduloAssign;
export const powerAssign = CompoundAssignmentOperators.powerAssign;
// Global registration for runtime use
if (typeof globalThis !== 'undefined') {
    globalThis.__nagari_operators__ = CompoundAssignmentOperators;
}
//# sourceMappingURL=operators.js.map