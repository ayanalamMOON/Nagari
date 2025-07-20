// Built-in function implementations for Nagari

export function len(obj: any): number {
    if (obj === null || obj === undefined) {
        throw new Error("object has no len()");
    }

    if (typeof obj === 'string' || Array.isArray(obj)) {
        return obj.length;
    }

    if (typeof obj === 'object') {
        return Object.keys(obj).length;
    }

    throw new Error(`object of type '${typeof obj}' has no len()`);
}

export function type(obj: any): string {
    if (obj === null) return 'none';
    if (obj === undefined) return 'none';
    if (Array.isArray(obj)) return 'list';
    if (typeof obj === 'object') return 'dict';
    return typeof obj;
}

export function str(obj: any): string {
    if (obj === null || obj === undefined) return 'none';
    if (typeof obj === 'boolean') return obj ? 'true' : 'false';
    if (Array.isArray(obj)) {
        return '[' + obj.map(str).join(', ') + ']';
    }
    if (typeof obj === 'object') {
        const pairs = Object.entries(obj).map(([k, v]) => `${k}: ${str(v)}`);
        return '{' + pairs.join(', ') + '}';
    }
    return String(obj);
}

export function int(obj: any): number {
    if (typeof obj === 'number') return Math.floor(obj);
    if (typeof obj === 'string') {
        const parsed = parseInt(obj, 10);
        if (isNaN(parsed)) throw new Error(`invalid literal for int(): '${obj}'`);
        return parsed;
    }
    if (typeof obj === 'boolean') return obj ? 1 : 0;
    throw new Error(`int() argument must be a string, a bytes-like object or a number, not '${typeof obj}'`);
}

export function float(obj: any): number {
    if (typeof obj === 'number') return obj;
    if (typeof obj === 'string') {
        const parsed = parseFloat(obj);
        if (isNaN(parsed)) throw new Error(`could not convert string to float: '${obj}'`);
        return parsed;
    }
    if (typeof obj === 'boolean') return obj ? 1.0 : 0.0;
    throw new Error(`float() argument must be a string or a number, not '${typeof obj}'`);
}

export function bool(obj: any): boolean {
    if (obj === null || obj === undefined) return false;
    if (typeof obj === 'boolean') return obj;
    if (typeof obj === 'number') return obj !== 0;
    if (typeof obj === 'string') return obj.length > 0;
    if (Array.isArray(obj)) return obj.length > 0;
    if (typeof obj === 'object') return Object.keys(obj).length > 0;
    return true;
}

// Export a print function that can be used in both Node and browser
export const print = typeof console !== 'undefined'
    ? (...args: any[]) => console.log(...args.map(str))
    : (...args: any[]) => { };

// Python-style range function
export function range(start: number, stop?: number, step: number = 1): number[] {
    if (stop === undefined) {
        stop = start;
        start = 0;
    }

    const result: number[] = [];
    if (step > 0) {
        for (let i = start; i < stop; i += step) {
            result.push(i);
        }
    } else if (step < 0) {
        for (let i = start; i > stop; i += step) {
            result.push(i);
        }
    }
    return result;
}

// Exception classes
export class Exception extends Error {
    constructor(message: string = '') {
        super(message);
        this.name = 'Exception';
    }
}

export class ValueError extends Exception {
    constructor(message: string = '') {
        super(message);
        this.name = 'ValueError';
    }
}

export class TypeError extends Exception {
    constructor(message: string = '') {
        super(message);
        this.name = 'TypeError';
    }
}

export class KeyError extends Exception {
    constructor(message: string = '') {
        super(message);
        this.name = 'KeyError';
    }
}

export class IndexError extends Exception {
    constructor(message: string = '') {
        super(message);
        this.name = 'IndexError';
    }
}

// JavaScript error alias (for catching JS errors in try/except)
export const js_error = Error;

// Additional utility functions
export function hasattr(obj: any, attr: string): boolean {
    return attr in obj;
}

export function getattr(obj: any, attr: string, defaultValue?: any): any {
    if (attr in obj) {
        return obj[attr];
    }
    if (defaultValue !== undefined) {
        return defaultValue;
    }
    throw new Error(`'${typeof obj}' object has no attribute '${attr}'`);
}

export function setattr(obj: any, attr: string, value: any): void {
    obj[attr] = value;
}

export function delattr(obj: any, attr: string): void {
    delete obj[attr];
}

export function isinstance(obj: any, types: any): boolean {
    if (Array.isArray(types)) {
        return types.some(t => {
            if (t === Array) return Array.isArray(obj);
            if (t === Object || t.name === 'dict') return typeof obj === 'object' && obj !== null && !Array.isArray(obj);
            if (t === String || t.name === 'str') return typeof obj === 'string';
            if (t === Number || t.name === 'int' || t.name === 'float') return typeof obj === 'number';
            if (t === Boolean || t.name === 'bool') return typeof obj === 'boolean';
            return obj instanceof t;
        });
    } else {
        const t = types;
        if (t === Array) return Array.isArray(obj);
        if (t === Object || t.name === 'dict') return typeof obj === 'object' && obj !== null && !Array.isArray(obj);
        if (t === String || t.name === 'str') return typeof obj === 'string';
        if (t === Number || t.name === 'int' || t.name === 'float') return typeof obj === 'number';
        if (t === Boolean || t.name === 'bool') return typeof obj === 'boolean';
        return obj instanceof t;
    }
}

// Type aliases for common types
export const dict = Object;
export const list = Array;

// String manipulation functions
export function str_capitalize(s: string): string {
    if (s.length === 0) {
        return s;
    }
    return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}

export function str_title(s: string): string {
    return s.split(' ').map(word => {
        if (word.length > 0) {
            return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
        }
        return word;
    }).join(' ');
}

export function str_reverse(s: string): string {
    return s.split('').reverse().join('');
}

export function str_count(s: string, substring: string): number {
    if (substring.length === 0) {
        return s.length + 1;
    }

    let count = 0;
    let pos = 0;

    while (true) {
        const index = s.indexOf(substring, pos);
        if (index === -1) break;
        count++;
        pos = index + 1;
    }

    return count;
}

export function str_pad_left(s: string, width: number, fillchar: string = ' '): string {
    if (s.length >= width) {
        return s;
    }

    const paddingNeeded = width - s.length;
    const char = fillchar.length === 0 ? ' ' : fillchar.charAt(0);

    return char.repeat(paddingNeeded) + s;
}

export function str_pad_right(s: string, width: number, fillchar: string = ' '): string {
    if (s.length >= width) {
        return s;
    }

    const paddingNeeded = width - s.length;
    const char = fillchar.length === 0 ? ' ' : fillchar.charAt(0);

    return s + char.repeat(paddingNeeded);
}

export function str_center(s: string, width: number, fillchar: string = ' '): string {
    if (s.length >= width) {
        return s;
    }

    const paddingNeeded = width - s.length;
    const char = fillchar.length === 0 ? ' ' : fillchar.charAt(0);

    const leftPadding = Math.floor(paddingNeeded / 2);
    const rightPadding = paddingNeeded - leftPadding;

    return char.repeat(leftPadding) + s + char.repeat(rightPadding);
}

// Format helper functions for enhanced f-string support
export function format_percentage(value: number, precision: number = 2): string {
    return (value * 100).toFixed(precision) + '%';
}

export function format_currency(value: number, precision: number = 2, symbol: string = '$'): string {
    return symbol + value.toFixed(precision);
}

export function format_number_with_commas(value: number): string {
    return value.toLocaleString();
}

export function format_scientific(value: number, precision: number = 2): string {
    return value.toExponential(precision);
}

export function center_string(str: string, width: number, fill: string = ' '): string {
    const s = str.toString();
    if (s.length >= width) return s;
    const padding = width - s.length;
    const leftPad = Math.floor(padding / 2);
    const rightPad = padding - leftPad;
    return fill.repeat(leftPad) + s + fill.repeat(rightPad);
}
