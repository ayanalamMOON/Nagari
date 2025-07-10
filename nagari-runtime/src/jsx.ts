// JSX Runtime for Nagari
// Provides React-compatible JSX transformation and utilities

import { jsToNagari, nagariToJS, wrapJSFunction } from './interop.js';

export interface JSXElement {
    type: string | Function;
    props: { [key: string]: any };
    children: (JSXElement | string)[] | JSXElement | string | null;
}

/**
 * JSX factory function - transpiles JSX to React.createElement calls
 */
export function jsx(
    type: string | Function,
    props: { [key: string]: any } | null,
    ...children: any[]
): JSXElement {
    const normalizedProps = props || {};

    // Filter out undefined children and flatten arrays
    const normalizedChildren = children
        .flat()
        .filter(child => child !== null && child !== undefined);

    if (normalizedChildren.length === 1) {
        normalizedProps.children = normalizedChildren[0];
    } else if (normalizedChildren.length > 1) {
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
export function Fragment(props: { children?: any }): JSXElement {
    return jsx('React.Fragment', null, ...(Array.isArray(props.children) ? props.children : [props.children]));
}

/**
 * Convert Nagari JSX to React elements
 */
export function jsxToReact(element: JSXElement | string): any {
    if (typeof element === 'string') {
        return element;
    }

    if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
        const React = (globalThis as any).React;
        const children = element.children ?
            (Array.isArray(element.children) ?
                element.children.map(jsxToReact) :
                [jsxToReact(element.children)]
            ) : [];

        return React.createElement(
            element.type,
            nagariToJS(element.props),
            ...children
        );
    }

    // Fallback for environments without React
    return element;
}

/**
 * React interop utilities
 */
export const ReactInterop = {
    // Core React functions
    createElement: wrapJSFunction(
        (type: any, props: any, ...children: any[]) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return (globalThis as any).React.createElement(
                    typeof type === 'string' ? type : nagariToJS(type),
                    nagariToJS(props),
                    ...children.map(nagariToJS)
                );
            }
            return jsx(type, props, ...children);
        },
        'createElement'
    ),

    // React hooks wrappers
    useState: wrapJSFunction(
        (initialState: any) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                const [state, setState] = (globalThis as any).React.useState(nagariToJS(initialState));
                return [jsToNagari(state), wrapJSFunction((newState: any) => setState(nagariToJS(newState)), 'setState')];
            }
            throw new Error('React not available in this environment');
        },
        'useState'
    ),

    useEffect: wrapJSFunction(
        (effect: Function, deps?: any[]) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return (globalThis as any).React.useEffect(
                    () => {
                        const cleanup = effect();
                        return cleanup ? () => nagariToJS(cleanup) : undefined;
                    },
                    deps ? deps.map(nagariToJS) : undefined
                );
            }
            throw new Error('React not available in this environment');
        },
        'useEffect'
    ),

    useCallback: wrapJSFunction(
        (callback: Function, deps: any[]) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return (globalThis as any).React.useCallback(
                    (...args: any[]) => nagariToJS(callback(...args.map(jsToNagari))),
                    deps.map(nagariToJS)
                );
            }
            throw new Error('React not available in this environment');
        },
        'useCallback'
    ),

    useMemo: wrapJSFunction(
        (factory: Function, deps: any[]) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return jsToNagari((globalThis as any).React.useMemo(
                    () => nagariToJS(factory()),
                    deps.map(nagariToJS)
                ));
            }
            throw new Error('React not available in this environment');
        },
        'useMemo'
    ),

    useRef: wrapJSFunction(
        (initialValue: any) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                const ref = (globalThis as any).React.useRef(nagariToJS(initialValue));
                return jsToNagari({
                    get current() { return jsToNagari(ref.current); },
                    set current(value: any) { ref.current = nagariToJS(value); }
                });
            }
            throw new Error('React not available in this environment');
        },
        'useRef'
    ),

    useContext: wrapJSFunction(
        (context: any) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return jsToNagari((globalThis as any).React.useContext(nagariToJS(context)));
            }
            throw new Error('React not available in this environment');
        },
        'useContext'
    ),

    // Context API
    createContext: wrapJSFunction(
        (defaultValue: any) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return jsToNagari((globalThis as any).React.createContext(nagariToJS(defaultValue)));
            }
            throw new Error('React not available in this environment');
        },
        'createContext'
    ),

    // Component utilities
    memo: wrapJSFunction(
        (component: Function, areEqual?: Function) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return (globalThis as any).React.memo(
                    (props: any) => nagariToJS(component(jsToNagari(props))),
                    areEqual ? (prevProps: any, nextProps: any) =>
                        nagariToJS(areEqual(jsToNagari(prevProps), jsToNagari(nextProps))) : undefined
                );
            }
            throw new Error('React not available in this environment');
        },
        'memo'
    ),

    forwardRef: wrapJSFunction(
        (render: Function) => {
            if (typeof globalThis !== 'undefined' && (globalThis as any).React) {
                return (globalThis as any).React.forwardRef(
                    (props: any, ref: any) => nagariToJS(render(jsToNagari(props), jsToNagari(ref)))
                );
            }
            throw new Error('React not available in this environment');
        },
        'forwardRef'
    ),

    // Fragment
    Fragment
};

/**
 * Event handler wrapper for React events
 */
export function wrapEventHandler(handler: Function): Function {
    return (event: Event) => {
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
export function convertStyles(styles: { [key: string]: any }): { [key: string]: any } {
    const converted: { [key: string]: any } = {};

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
export function withNagariProps(WrappedComponent: Function): Function {
    return (props: any) => {
        const nagariProps = jsToNagari(props);
        const result = WrappedComponent(nagariProps);
        return nagariToJS(result);
    };
}
