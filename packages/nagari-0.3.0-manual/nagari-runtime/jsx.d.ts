export interface JSXElement {
    type: string | Function;
    props: {
        [key: string]: any;
    };
    children: (JSXElement | string)[] | JSXElement | string | null;
}
/**
 * JSX factory function - transpiles JSX to React.createElement calls
 */
export declare function jsx(type: string | Function, props: {
    [key: string]: any;
} | null, ...children: any[]): JSXElement;
/**
 * JSX Fragment component
 */
export declare function Fragment(props: {
    children?: any;
}): JSXElement;
/**
 * Convert Nagari JSX to React elements
 */
export declare function jsxToReact(element: JSXElement | string): any;
/**
 * React interop utilities
 */
export declare const ReactInterop: {
    createElement: import("./types.js").NagariFunction;
    useState: import("./types.js").NagariFunction;
    useEffect: import("./types.js").NagariFunction;
    useCallback: import("./types.js").NagariFunction;
    useMemo: import("./types.js").NagariFunction;
    useRef: import("./types.js").NagariFunction;
    useContext: import("./types.js").NagariFunction;
    createContext: import("./types.js").NagariFunction;
    memo: import("./types.js").NagariFunction;
    forwardRef: import("./types.js").NagariFunction;
    Fragment: typeof Fragment;
};
/**
 * Event handler wrapper for React events
 */
export declare function wrapEventHandler(handler: Function): Function;
/**
 * Style object converter for CSS-in-JS
 */
export declare function convertStyles(styles: {
    [key: string]: any;
}): {
    [key: string]: any;
};
/**
 * HOC (Higher-Order Component) wrapper
 */
export declare function withNagariProps(WrappedComponent: Function): Function;
//# sourceMappingURL=jsx.d.ts.map