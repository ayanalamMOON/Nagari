#!/bin/bash
set -e

echo "Building Nagari WebAssembly Runtime..."

# Check for required tools
command -v wasm-pack >/dev/null 2>&1 || {
    echo "wasm-pack is required but not installed. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
}

# Build WebAssembly package
cd nagari-wasm
wasm-pack build --target web --out-dir pkg --release

# Build for different targets
echo "Building for web target..."
wasm-pack build --target web --out-dir pkg/web --release

echo "Building for Node.js target..."
wasm-pack build --target nodejs --out-dir pkg/nodejs --release

echo "Building for bundler target..."
wasm-pack build --target bundler --out-dir pkg/bundler --release

echo "Building for no-modules target..."
wasm-pack build --target no-modules --out-dir pkg/no-modules --release

# Create package.json for npm publishing
cat > pkg/package.json << EOF
{
  "name": "nagari-wasm",
  "version": "0.3.0",
  "description": "WebAssembly runtime for the Nagari programming language",
  "main": "nagari_wasm.js",
  "types": "nagari_wasm.d.ts",
  "files": [
    "nagari_wasm_bg.wasm",
    "nagari_wasm.js",
    "nagari_wasm.d.ts"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/nagari-lang/nagari"
  },
  "keywords": [
    "webassembly",
    "wasm",
    "nagari",
    "programming-language",
    "runtime",
    "interpreter"
  ],
  "author": "Nagari Language Team",
  "license": "MIT",
  "homepage": "https://nagari.dev",
  "bugs": {
    "url": "https://github.com/nagari-lang/nagari/issues"
  }
}
EOF

# Create TypeScript declarations
cat > pkg/nagari_wasm.d.ts << 'EOF'
/* tslint:disable */
/* eslint-disable */
/**
* @returns {Promise<NagariWasmVM>}
*/
export function initNagari(): Promise<NagariWasmVM>;

/**
* @returns {NagariWasmVM}
*/
export function createNagariInstance(): NagariWasmVM;

/**
*/
export class BrowserUtils {
  free(): void;
/**
* @returns {string}
*/
  static getUserAgent(): string;
/**
* @returns {Array<any>}
*/
  static getWindowDimensions(): Array<any>;
/**
* @param {string} key
* @param {string} value
*/
  static localStorageSet(key: string, value: string): void;
/**
* @param {string} key
* @returns {string | undefined}
*/
  static localStorageGet(key: string): string | undefined;
}

/**
*/
export class JSValue {
  free(): void;
/**
* @param {any} value
*/
  constructor(value: any);
/**
* @returns {any}
*/
  readonly value: any;
/**
* @returns {string | undefined}
*/
  asString(): string | undefined;
/**
* @returns {number | undefined}
*/
  asNumber(): number | undefined;
/**
* @returns {boolean | undefined}
*/
  asBool(): boolean | undefined;
/**
* @returns {boolean}
*/
  isNull(): boolean;
/**
* @returns {boolean}
*/
  isUndefined(): boolean;
}

/**
*/
export class NagariWasmVM {
  free(): void;
/**
*/
  constructor();
/**
* @param {string} code
* @returns {JSValue}
*/
  run(code: string): JSValue;
/**
* @param {string} code
* @returns {JSValue}
*/
  eval(code: string): JSValue;
/**
* @param {string} function_name
* @param {Array<any>} args
* @returns {JSValue}
*/
  call(function_name: string, args: Array<any>): JSValue;
/**
* @param {string} module_name
* @param {string} code
*/
  loadModule(module_name: string, code: string): void;
/**
* @param {string} name
* @param {any} value
*/
  setGlobal(name: string, value: any): void;
/**
* @param {string} name
* @returns {JSValue}
*/
  getGlobal(name: string): JSValue;
/**
* @param {string} name
* @param {Function} func
*/
  registerJsFunction(name: string, func: Function): void;
/**
* @returns {any}
*/
  getPerformanceStats(): any;
/**
*/
  reset(): void;
}

/**
*/
export class PerformanceMonitor {
  free(): void;
/**
*/
  constructor();
/**
* @returns {number}
*/
  elapsed(): number;
/**
* @param {string} name
*/
  mark(name: string): void;
/**
* @param {string} name
* @param {string} start_mark
* @param {string} end_mark
* @returns {number}
*/
  measure(name: string, start_mark: string, end_mark: string): number;
}

/**
*/
export class ReactHooks {
  free(): void;
/**
*/
  constructor();
/**
* @param {string} initial_code
* @returns {JSValue}
*/
  useNagariState(initial_code: string): JSValue;
/**
* @param {string} effect_code
* @param {Array<any>} dependencies
*/
  useNagariEffect(effect_code: string, dependencies: Array<any>): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_jSValue_free: (a: number) => void;
  readonly __wbg_get_jSValue_inner: (a: number) => number;
  readonly jSValue_new: (a: number) => number;
  readonly jSValue_value: (a: number) => number;
  readonly jSValue_as_string: (a: number, b: number) => void;
  readonly jSValue_as_number: (a: number, b: number) => void;
  readonly jSValue_as_bool: (a: number, b: number) => void;
  readonly jSValue_is_null: (a: number) => number;
  readonly jSValue_is_undefined: (a: number) => number;
  readonly __wbg_nagariwasm vm_free: (a: number) => void;
  readonly nagariwasm vm_new: () => number;
  readonly nagariwasm vm_run: (a: number, b: number, c: number, d: number) => void;
  readonly nagariwasm vm_eval: (a: number, b: number, c: number, d: number) => void;
  readonly nagariwasm vm_call: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly nagariwasm vm_load_module: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly nagariwasm vm_set_global: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly nagariwasm vm_get_global: (a: number, b: number, c: number, d: number) => void;
  readonly nagariwasm vm_register_js_function: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly nagariwasm vm_get_performance_stats: (a: number) => number;
  readonly nagariwasm vm_reset: (a: number, b: number) => void;
  readonly greet: () => void;
  readonly main: () => void;
  readonly initNagari: () => number;
  readonly createNagariInstance: () => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c4c1a7a0c9d5f9c: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h1e2c4e9c3a1f8b7d: (a: number, b: number, c: number, d: number) => void;
}

export function __wbg_set_wasm(val: InitOutput): void;
EOF

echo "Creating React integration package..."
mkdir -p pkg/react
cat > pkg/react/package.json << EOF
{
  "name": "nagari-react",
  "version": "0.3.0",
  "description": "React integration for Nagari WebAssembly runtime",
  "main": "index.js",
  "types": "index.d.ts",
  "peerDependencies": {
    "react": ">=16.8.0"
  },
  "dependencies": {
    "nagari-wasm": "^0.3.0"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/nagari-lang/nagari"
  },
  "keywords": [
    "react",
    "nagari",
    "webassembly",
    "hooks"
  ],
  "author": "Nagari Language Team",
  "license": "MIT"
}
EOF

cat > pkg/react/index.js << 'EOF'
import { useEffect, useState, useCallback, useRef } from 'react';
import { initNagari } from 'nagari-wasm';

export function useNagari(initialCode) {
  const [vm, setVm] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const vmRef = useRef(null);

  useEffect(() => {
    let mounted = true;

    async function initializeVM() {
      try {
        const nagariVM = await initNagari();

        if (initialCode) {
          nagariVM.run(initialCode);
        }

        if (mounted) {
          setVm(nagariVM);
          vmRef.current = nagariVM;
          setLoading(false);
        }
      } catch (err) {
        if (mounted) {
          setError(err);
          setLoading(false);
        }
      }
    }

    initializeVM();

    return () => {
      mounted = false;
      if (vmRef.current) {
        vmRef.current.reset();
      }
    };
  }, [initialCode]);

  const run = useCallback((code) => {
    if (vmRef.current) {
      return vmRef.current.run(code);
    }
    return null;
  }, []);

  const call = useCallback((functionName, ...args) => {
    if (vmRef.current) {
      return vmRef.current.call(functionName, args);
    }
    return null;
  }, []);

  const setGlobal = useCallback((name, value) => {
    if (vmRef.current) {
      vmRef.current.setGlobal(name, value);
    }
  }, []);

  const getGlobal = useCallback((name) => {
    if (vmRef.current) {
      return vmRef.current.getGlobal(name);
    }
    return null;
  }, []);

  return {
    vm,
    loading,
    error,
    run,
    call,
    setGlobal,
    getGlobal
  };
}

export function useNagariState(initialCode) {
  const { vm, loading, error } = useNagari();
  const [state, setState] = useState(null);

  useEffect(() => {
    if (vm && initialCode) {
      const result = vm.run(initialCode);
      setState(result.value);
    }
  }, [vm, initialCode]);

  const updateState = useCallback((newCode) => {
    if (vm) {
      const result = vm.run(newCode);
      setState(result.value);
      return result.value;
    }
    return null;
  }, [vm]);

  return [state, updateState, { loading, error }];
}

export function useNagariEffect(effectCode, dependencies = []) {
  const { vm } = useNagari();

  useEffect(() => {
    if (vm && effectCode) {
      vm.run(effectCode);
    }
  }, [vm, effectCode, ...dependencies]);
}
EOF

cat > pkg/react/index.d.ts << 'EOF'
import { JSValue } from 'nagari-wasm';

export interface NagariHookResult {
  vm: any | null;
  loading: boolean;
  error: Error | null;
  run: (code: string) => JSValue | null;
  call: (functionName: string, ...args: any[]) => JSValue | null;
  setGlobal: (name: string, value: any) => void;
  getGlobal: (name: string) => JSValue | null;
}

export function useNagari(initialCode?: string): NagariHookResult;

export function useNagariState(initialCode: string): [any, (newCode: string) => any, { loading: boolean; error: Error | null }];

export function useNagariEffect(effectCode: string, dependencies?: any[]): void;
EOF

echo "WebAssembly runtime build completed!"
echo "Files generated:"
echo "  - nagari-wasm/pkg/ - Main WebAssembly package"
echo "  - nagari-wasm/pkg/web/ - Web target"
echo "  - nagari-wasm/pkg/nodejs/ - Node.js target"
echo "  - nagari-wasm/pkg/bundler/ - Bundler target"
echo "  - nagari-wasm/pkg/no-modules/ - No modules target"
echo "  - nagari-wasm/pkg/react/ - React integration"

cd ..
