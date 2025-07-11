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
