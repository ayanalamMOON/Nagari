// Type definitions for Nagari runtime
// Type guards
export function isNagariFunction(value) {
    return typeof value === 'function' && value.__nagari_function__ === true;
}
export function isNagariClass(value) {
    return typeof value === 'function' && value.__nagari_class__ === true;
}
export function isNagariModule(value) {
    return typeof value === 'object' && value && value.__nagari_module__ === true;
}
// Utility to create Nagari-compatible functions
export function createFunction(fn, name, arity) {
    const nagariFunction = fn;
    nagariFunction.__nagari_function__ = true;
    if (name)
        nagariFunction.name = name;
    if (arity !== undefined)
        nagariFunction.arity = arity;
    return nagariFunction;
}
//# sourceMappingURL=types.js.map