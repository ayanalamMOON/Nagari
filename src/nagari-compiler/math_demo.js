"use strict";

import { jsToNagari, nagariToJS, InteropRegistry, str_capitalize, str_title, str_reverse, str_count, str_pad_left, str_pad_right, str_center } from 'nagari-runtime';


// Browser polyfills
if (typeof globalThis === 'undefined') {
    window.globalThis = window;
}

// Node.js-like APIs for browser
if (typeof require === 'undefined') {
    globalThis.require = (module) => {
        throw new Error(`Module '${module}' not available in browser environment. Use import instead.`);
    };
}

// Process object for browser
if (typeof process === 'undefined') {
    globalThis.process = {
        env: {},
        argv: [],
        cwd: () => '/',
        exit: (code) => console.log(`Process would exit with code: ${code}`)
    };
}

// Initialize Nagari runtime
if (typeof globalThis !== 'undefined' && !globalThis.__nagari__) {
    InteropRegistry.initialize();
}

function calculate_circle_area(radius) {
    let PI = 3.141592653589793;
    return ((PI * radius) * radius);
}
function calculate_rectangle_area(width, height) {
    return (width * height);
}
function sum_list(numbers) {
    let result = 0;
    for (const num of numbers) {
        result = (result + num);
    }
    return result;
}
function max_list(numbers) {
    if ((numbers.length === 0)) {
        return 0;
    }
    let result = /* TODO: Implement this expression type */;
    for (const num of numbers) {
        if ((num > result)) {
            result = num;
        }
    }
    return result;
}
function min_list(numbers) {
    if ((numbers.length === 0)) {
        return 0;
    }
    let result = /* TODO: Implement this expression type */;
    for (const num of numbers) {
        if ((num < result)) {
            result = num;
        }
    }
    return result;
}
function main() {
    console.log("Circle area (radius 5):", calculate_circle_area(5));
    console.log("Rectangle area (4x6):", calculate_rectangle_area(4, 6));
    let numbers = [1, 2, 3, 4, 5];
    console.log("Numbers:", numbers);
    console.log("Sum:", sum_list(numbers));
    console.log("Max:", max_list(numbers));
    console.log("Min:", min_list(numbers));
}
main();

// Python-style range function
function range(start, stop, step = 1) {
    if (arguments.length === 1) {
        stop = start;
        start = 0;
    }
    const result = [];
    if (step > 0) {
        for (let i = start; i < stop; i += step) {
            result.push(i);
        }
    } else {
        for (let i = start; i > stop; i += step) {
            result.push(i);
        }
    }
    return result;
}


// Python-style zip function
function zip(...iterables) {
    const length = Math.min(...iterables.map(arr => arr.length));
    const result = [];
    for (let i = 0; i < length; i++) {
        result.push(iterables.map(arr => arr[i]));
    }
    return result;
}


// Python-style sum function
function sum(iterable, start = 0) {
    return iterable.reduce((acc, val) => acc + val, start);
}


// Python-style enumerate function
function enumerate(iterable, start = 0) {
    return iterable.map((item, index) => [index + start, item]);
}


// Python-style f-string formatting
function formatString(template, ...values) {
    let result = template;
    let valueIndex = 0;
    result = result.replace(/\{([^}]*)\}/g, (match, expr) => {
        if (expr === '') {
            return values[valueIndex++];
        }
        // Simple expression evaluation (can be enhanced)
        try {
            return eval(expr);
        } catch {
            return match;
        }
    });
    return result;
}


// List comprehension helpers
function listComp(iterable, transform, condition = () => true) {
    return iterable.filter(condition).map(transform);
}

function dictComp(iterable, keyTransform, valueTransform, condition = () => true) {
    const result = {};
    iterable.filter(condition).forEach(item => {
        result[keyTransform(item)] = valueTransform(item);
    });
    return result;
}

function setComp(iterable, transform, condition = () => true) {
    return new Set(iterable.filter(condition).map(transform));
}

