// Type definitions for Nagari runtime

export type NagariValue =
    | number
    | string
    | boolean
    | null
    | undefined
    | NagariValue[]
    | { [key: string]: NagariValue }
    | NagariFunction;

export interface NagariFunction {
    (...args: NagariValue[]): NagariValue | Promise<NagariValue>;
    __nagari_function__: true;
    name?: string;
    arity?: number;
}

export interface NagariClass {
    new(...args: NagariValue[]): NagariValue;
    __nagari_class__: true;
    name: string;
}

export interface NagariModule {
    [key: string]: NagariValue | NagariFunction | NagariClass;
    __nagari_module__: true;
    name: string;
}

// Type guards
export function isNagariFunction(value: any): value is NagariFunction {
    return typeof value === 'function' && value.__nagari_function__ === true;
}

export function isNagariClass(value: any): value is NagariClass {
    return typeof value === 'function' && value.__nagari_class__ === true;
}

export function isNagariModule(value: any): value is NagariModule {
    return typeof value === 'object' && value && value.__nagari_module__ === true;
}

// Utility to create Nagari-compatible functions
export function createFunction(
    fn: (...args: any[]) => any,
    name?: string,
    arity?: number
): NagariFunction {
    const nagariFunction = fn as NagariFunction;
    nagariFunction.__nagari_function__ = true;
    if (name) nagariFunction.name = name;
    if (arity !== undefined) nagariFunction.arity = arity;
    return nagariFunction;
}
