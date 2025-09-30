// JSX Runtime for Nagari
// Provides React-compatible JSX transformation and utilities
import { jsToNagari, nagariToJS, wrapJSFunction } from './interop.js';
/**
 * JSX factory function - transpiles JSX to React.createElement calls
 */
export function jsx(type, props, ...children) {
    const normalizedProps = props || {};
    // Filter out undefined children and flatten arrays
    const normalizedChildren = children
        .flat()
        .filter(child => child !== null && child !== undefined);
    if (normalizedChildren.length === 1) {
        normalizedProps.children = normalizedChildren[0];
    }
    else if (normalizedChildren.length > 1) {
        normalizedProps.children = normalizedChildren;
    }
    return {
        type,
        props: normalizedProps,
        children: normalizedChildren.length > 0 ? normalizedChildren : null
    };
}
/**
 * JSX Fragment component
 */
export function Fragment(props) {
    return jsx('React.Fragment', null, ...(Array.isArray(props.children) ? props.children : [props.children]));
}
/**
 * Convert Nagari JSX to React elements
 */
export function jsxToReact(element) {
    if (typeof element === 'string') {
        return element;
    }
    if (typeof globalThis !== 'undefined' && globalThis.React) {
        const React = globalThis.React;
        const children = element.children ?
            (Array.isArray(element.children) ?
                element.children.map(jsxToReact) :
                [jsxToReact(element.children)]) : [];
        return React.createElement(element.type, nagariToJS(element.props), ...children);
    }
    // Fallback for environments without React
    return element;
}
/**
 * React interop utilities
 */
export const ReactInterop = {
    // Core React functions
    createElement: wrapJSFunction((type, props, ...children) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return globalThis.React.createElement(typeof type === 'string' ? type : nagariToJS(type), nagariToJS(props), ...children.map(nagariToJS));
        }
        return jsx(type, props, ...children);
    }, 'createElement'),
    // React hooks wrappers
    useState: wrapJSFunction((initialState) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            const [state, setState] = globalThis.React.useState(nagariToJS(initialState));
            return [jsToNagari(state), wrapJSFunction((newState) => setState(nagariToJS(newState)), 'setState')];
        }
        throw new Error('React not available in this environment');
    }, 'useState'),
    useEffect: wrapJSFunction((effect, deps) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return globalThis.React.useEffect(() => {
                const cleanup = effect();
                return cleanup ? () => nagariToJS(cleanup) : undefined;
            }, deps ? deps.map(nagariToJS) : undefined);
        }
        throw new Error('React not available in this environment');
    }, 'useEffect'),
    useCallback: wrapJSFunction((callback, deps) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return globalThis.React.useCallback((...args) => nagariToJS(callback(...args.map(jsToNagari))), deps.map(nagariToJS));
        }
        throw new Error('React not available in this environment');
    }, 'useCallback'),
    useMemo: wrapJSFunction((factory, deps) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return jsToNagari(globalThis.React.useMemo(() => nagariToJS(factory()), deps.map(nagariToJS)));
        }
        throw new Error('React not available in this environment');
    }, 'useMemo'),
    useRef: wrapJSFunction((initialValue) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            const ref = globalThis.React.useRef(nagariToJS(initialValue));
            return jsToNagari({
                get current() { return jsToNagari(ref.current); },
                set current(value) { ref.current = nagariToJS(value); }
            });
        }
        throw new Error('React not available in this environment');
    }, 'useRef'),
    useContext: wrapJSFunction((context) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return jsToNagari(globalThis.React.useContext(nagariToJS(context)));
        }
        throw new Error('React not available in this environment');
    }, 'useContext'),
    // Context API
    createContext: wrapJSFunction((defaultValue) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return jsToNagari(globalThis.React.createContext(nagariToJS(defaultValue)));
        }
        throw new Error('React not available in this environment');
    }, 'createContext'),
    // Component utilities
    memo: wrapJSFunction((component, areEqual) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return globalThis.React.memo((props) => nagariToJS(component(jsToNagari(props))), areEqual ? (prevProps, nextProps) => nagariToJS(areEqual(jsToNagari(prevProps), jsToNagari(nextProps))) : undefined);
        }
        throw new Error('React not available in this environment');
    }, 'memo'),
    forwardRef: wrapJSFunction((render) => {
        if (typeof globalThis !== 'undefined' && globalThis.React) {
            return globalThis.React.forwardRef((props, ref) => nagariToJS(render(jsToNagari(props), jsToNagari(ref))));
        }
        throw new Error('React not available in this environment');
    }, 'forwardRef'),
    // Fragment
    Fragment
};
/**
 * Event handler wrapper for React events
 */
export function wrapEventHandler(handler) {
    return (event) => {
        const nagariEvent = jsToNagari({
            eventType: event.type,
            eventTarget: event.target,
            eventCurrentTarget: event.currentTarget,
            preventDefault: () => event.preventDefault(),
            stopPropagation: () => event.stopPropagation()
        });
        return nagariToJS(handler(nagariEvent));
    };
}
/**
 * Style object converter for CSS-in-JS
 */
export function convertStyles(styles) {
    const converted = {};
    for (const [key, value] of Object.entries(styles)) {
        // Convert kebab-case to camelCase for React
        const camelKey = key.replace(/-([a-z])/g, (_, char) => char.toUpperCase());
        converted[camelKey] = nagariToJS(value);
    }
    return converted;
}
/**
 * HOC (Higher-Order Component) wrapper
 */
export function withNagariProps(WrappedComponent) {
    return (props) => {
        const nagariProps = jsToNagari(props);
        const result = WrappedComponent(nagariProps);
        return nagariToJS(result);
    };
}
//# sourceMappingURL=jsx.js.map