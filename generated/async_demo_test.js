"use strict";

import { InteropRegistry } from './nagari-runtime/dist/index.js';


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

const http = InteropRegistry.getModule('http');
async function fetch_user_data(user_id) {
    let url = ("https://jsonplaceholder.typicode.com/users/" + String(user_id));
    let response = await http.get(url);
    console.log("Raw response:", response);
    console.log("Response status:", response.status);
    console.log("Response body:", response.body);
    let user_data = response.json();
    return user_data;
}
async function create_post(title, body, user_id) {
    let url = "https://jsonplaceholder.typicode.com/posts";
    let data = { "title": title, "body": body, "userId": user_id };
    let response = await http.post(url, data);
    console.log("POST response status:", response.status);
    console.log("POST response body:", response.body);
    let post_data = response.json();
    return post_data;
}
async function main() {
    console.log("Fetching user data...");
    let user = await fetch_user_data(1);
    console.log("User data:", user);
    console.log("Creating a new post...");
    let post = await create_post("My First Nagari Post", "This post was created using the Nagari programming language!", 1);
    console.log("Created post:", post);
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

