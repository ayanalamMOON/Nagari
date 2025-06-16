// JavaScript runtime helpers and builtin mappings

use std::collections::HashMap;

pub struct JSRuntime {
    target: String,
    builtin_mappings: HashMap<String, String>,
}

impl JSRuntime {
    pub fn new(target: &str) -> Self {
        let mut runtime = Self {
            target: target.to_string(),
            builtin_mappings: HashMap::new(),
        };
        runtime.init_builtin_mappings();
        runtime
    }

    fn init_builtin_mappings(&mut self) {
        // Nagari builtin -> JavaScript equivalent
        self.builtin_mappings.insert("print".to_string(), "console.log".to_string());
        self.builtin_mappings.insert("input".to_string(), "prompt".to_string());
        self.builtin_mappings.insert("len".to_string(), "length".to_string());
        self.builtin_mappings.insert("str".to_string(), "String".to_string());
        self.builtin_mappings.insert("int".to_string(), "parseInt".to_string());
        self.builtin_mappings.insert("float".to_string(), "parseFloat".to_string());
        self.builtin_mappings.insert("bool".to_string(), "Boolean".to_string());
        self.builtin_mappings.insert("list".to_string(), "Array".to_string());
        self.builtin_mappings.insert("dict".to_string(), "Object".to_string());
        self.builtin_mappings.insert("range".to_string(), "Array.from".to_string());
        self.builtin_mappings.insert("enumerate".to_string(), "entries".to_string());
        self.builtin_mappings.insert("zip".to_string(), "zip".to_string());
        self.builtin_mappings.insert("max".to_string(), "Math.max".to_string());
        self.builtin_mappings.insert("min".to_string(), "Math.min".to_string());
        self.builtin_mappings.insert("abs".to_string(), "Math.abs".to_string());
        self.builtin_mappings.insert("round".to_string(), "Math.round".to_string());
        self.builtin_mappings.insert("sum".to_string(), "sum".to_string());
        self.builtin_mappings.insert("any".to_string(), "some".to_string());
        self.builtin_mappings.insert("all".to_string(), "every".to_string());
    }

    pub fn get_builtin_mapping(&self, name: &str) -> Option<&String> {
        self.builtin_mappings.get(name)
    }

    pub fn generate_runtime_helpers(&self) -> String {
        let mut helpers = String::new();

        // Helper functions that don't exist in JS
        helpers.push_str(&self.generate_range_helper());
        helpers.push_str(&self.generate_zip_helper());
        helpers.push_str(&self.generate_sum_helper());
        helpers.push_str(&self.generate_enumerate_helper());

        // Python-style string formatting
        helpers.push_str(&self.generate_string_format_helper());

        // List comprehension helpers
        helpers.push_str(&self.generate_list_comprehension_helpers());

        helpers
    }

    fn generate_range_helper(&self) -> String {
        r#"
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

"#.to_string()
    }

    fn generate_zip_helper(&self) -> String {
        r#"
// Python-style zip function
function zip(...iterables) {
    const length = Math.min(...iterables.map(arr => arr.length));
    const result = [];
    for (let i = 0; i < length; i++) {
        result.push(iterables.map(arr => arr[i]));
    }
    return result;
}

"#.to_string()
    }

    fn generate_sum_helper(&self) -> String {
        r#"
// Python-style sum function
function sum(iterable, start = 0) {
    return iterable.reduce((acc, val) => acc + val, start);
}

"#.to_string()
    }

    fn generate_enumerate_helper(&self) -> String {
        r#"
// Python-style enumerate function
function enumerate(iterable, start = 0) {
    return iterable.map((item, index) => [index + start, item]);
}

"#.to_string()
    }

    fn generate_string_format_helper(&self) -> String {
        r#"
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

"#.to_string()
    }

    fn generate_list_comprehension_helpers(&self) -> String {
        r#"
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

"#.to_string()
    }

    pub fn wrap_async_function(&self, function_name: &str, is_async: bool) -> String {
        if is_async {
            match self.target.as_str() {
                "node" => {
                    format!("async function {}(...args) {{ return nagariToJS(await jsToNagari({}(...args.map(jsToNagari)))); }}", function_name, function_name)
                }
                _ => {
                    format!("async function {}(...args) {{ return nagariToJS(await jsToNagari({}(...args.map(jsToNagari)))); }}", function_name, function_name)
                }
            }
        } else {
            format!("function {}(...args) {{ return nagariToJS({}(...args.map(jsToNagari))); }}", function_name, function_name)
        }
    }

    pub fn generate_polyfills(&self) -> String {
        match self.target.as_str() {
            "node" => self.generate_node_polyfills(),
            _ => self.generate_browser_polyfills(),
        }
    }

    fn generate_node_polyfills(&self) -> String {
        r#"
// Node.js polyfills
if (typeof globalThis === 'undefined') {
    global.globalThis = global;
}

// DOM-like APIs for server-side rendering
if (typeof document === 'undefined') {
    globalThis.document = {
        getElementById: () => null,
        querySelector: () => null,
        querySelectorAll: () => [],
        createElement: (tag) => ({ tagName: tag, children: [] })
    };
}

if (typeof window === 'undefined') {
    globalThis.window = globalThis;
}

"#.to_string()
    }

    fn generate_browser_polyfills(&self) -> String {
        r#"
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

"#.to_string()
    }
}
