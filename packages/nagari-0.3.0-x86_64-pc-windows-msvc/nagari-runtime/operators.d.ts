/**
 * Compound assignment operator utilities
 * These functions provide safe implementations of compound assignment operators
 * with proper type checking and error handling
 */
export declare class CompoundAssignmentOperators {
    /**
     * Addition assignment: target += value
     */
    static addAssign(target: any, value: any): any;
    /**
     * Subtraction assignment: target -= value
     */
    static subtractAssign(target: any, value: any): any;
    /**
     * Multiplication assignment: target *= value
     */
    static multiplyAssign(target: any, value: any): any;
    /**
     * Division assignment: target /= value
     */
    static divideAssign(target: any, value: any): any;
    /**
     * Modulo assignment: target %= value
     */
    static moduloAssign(target: any, value: any): any;
    /**
     * Power assignment: target **= value
     */
    static powerAssign(target: any, value: any): any;
}
export declare const addAssign: typeof CompoundAssignmentOperators.addAssign;
export declare const subtractAssign: typeof CompoundAssignmentOperators.subtractAssign;
export declare const multiplyAssign: typeof CompoundAssignmentOperators.multiplyAssign;
export declare const divideAssign: typeof CompoundAssignmentOperators.divideAssign;
export declare const moduloAssign: typeof CompoundAssignmentOperators.moduloAssign;
export declare const powerAssign: typeof CompoundAssignmentOperators.powerAssign;
//# sourceMappingURL=operators.d.ts.map