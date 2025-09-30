export type NagariValue = number | string | boolean | null | undefined | NagariValue[] | {
    [key: string]: NagariValue;
} | NagariFunction;
export interface NagariFunction {
    (...args: NagariValue[]): NagariValue | Promise<NagariValue>;
    __nagari_function__: true;
    name?: string;
    arity?: number;
}
export interface NagariClass {
    new (...args: NagariValue[]): NagariValue;
    __nagari_class__: true;
    name: string;
}
export interface NagariModule {
    [key: string]: NagariValue | NagariFunction | NagariClass;
    __nagari_module__: true;
    name: string;
}
export declare function isNagariFunction(value: any): value is NagariFunction;
export declare function isNagariClass(value: any): value is NagariClass;
export declare function isNagariModule(value: any): value is NagariModule;
export declare function createFunction(fn: (...args: any[]) => any, name?: string, arity?: number): NagariFunction;
//# sourceMappingURL=types.d.ts.map