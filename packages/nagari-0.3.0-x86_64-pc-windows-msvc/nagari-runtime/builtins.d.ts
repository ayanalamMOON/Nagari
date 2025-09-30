export declare function len(obj: any): number;
export declare function type(obj: any): string;
export declare function str(obj: any): string;
export declare function int(obj: any): number;
export declare function float(obj: any): number;
export declare function bool(obj: any): boolean;
export declare const print: (...args: any[]) => void;
export declare function range(start: number, stop?: number, step?: number): number[];
export declare class Exception extends Error {
    constructor(message?: string);
}
export declare class ValueError extends Exception {
    constructor(message?: string);
}
export declare class TypeError extends Exception {
    constructor(message?: string);
}
export declare class KeyError extends Exception {
    constructor(message?: string);
}
export declare class IndexError extends Exception {
    constructor(message?: string);
}
export declare const js_error: ErrorConstructor;
export declare function hasattr(obj: any, attr: string): boolean;
export declare function getattr(obj: any, attr: string, defaultValue?: any): any;
export declare function setattr(obj: any, attr: string, value: any): void;
export declare function delattr(obj: any, attr: string): void;
export declare function isinstance(obj: any, types: any): boolean;
export declare const dict: ObjectConstructor;
export declare const list: ArrayConstructor;
export declare function str_capitalize(s: string): string;
export declare function str_title(s: string): string;
export declare function str_reverse(s: string): string;
export declare function str_count(s: string, substring: string): number;
export declare function str_pad_left(s: string, width: number, fillchar?: string): string;
export declare function str_pad_right(s: string, width: number, fillchar?: string): string;
export declare function str_center(s: string, width: number, fillchar?: string): string;
export declare function format_percentage(value: number, precision?: number): string;
export declare function format_currency(value: number, precision?: number, symbol?: string): string;
export declare function format_number_with_commas(value: number): string;
export declare function format_scientific(value: number, precision?: number): string;
export declare function center_string(str: string, width: number, fill?: string): string;
//# sourceMappingURL=builtins.d.ts.map