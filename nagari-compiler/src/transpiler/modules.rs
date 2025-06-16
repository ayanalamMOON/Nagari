// Module management for JavaScript transpilation

use crate::ast::ImportStatement;
use std::collections::HashMap;

pub struct ModuleResolver {
    builtin_modules: HashMap<String, BuiltinModule>,
    target: String,
}

#[derive(Clone)]
pub struct BuiltinModule {
    pub name: String,
    pub js_equivalent: String,
    pub exports: Vec<String>,
    pub interop_required: bool,
}

impl ModuleResolver {
    pub fn new(target: &str) -> Self {
        let mut resolver = Self {
            builtin_modules: HashMap::new(),
            target: target.to_string(),
        };
        resolver.init_builtin_modules();
        resolver
    }

    fn init_builtin_modules(&mut self) {
        // React ecosystem
        self.add_builtin_module(BuiltinModule {
            name: "react".to_string(),
            js_equivalent: "react".to_string(),
            exports: vec!["React".to_string(), "useState".to_string(), "useEffect".to_string()],
            interop_required: true,
        });

        // Node.js modules
        self.add_builtin_module(BuiltinModule {
            name: "fs".to_string(),
            js_equivalent: if self.target == "node" { "fs" } else { "fs/promises" }.to_string(),
            exports: vec!["readFile".to_string(), "writeFile".to_string(), "mkdir".to_string()],
            interop_required: true,
        });

        self.add_builtin_module(BuiltinModule {
            name: "http".to_string(),
            js_equivalent: "http".to_string(),
            exports: vec!["createServer".to_string(), "get".to_string(), "post".to_string()],
            interop_required: true,
        });

        self.add_builtin_module(BuiltinModule {
            name: "path".to_string(),
            js_equivalent: "path".to_string(),
            exports: vec!["join".to_string(), "resolve".to_string(), "dirname".to_string()],
            interop_required: false,
        });

        self.add_builtin_module(BuiltinModule {
            name: "os".to_string(),
            js_equivalent: "os".to_string(),
            exports: vec!["platform".to_string(), "arch".to_string(), "cpus".to_string()],
            interop_required: false,
        });

        // Express framework
        self.add_builtin_module(BuiltinModule {
            name: "express".to_string(),
            js_equivalent: "express".to_string(),
            exports: vec!["express".to_string()],
            interop_required: true,
        });

        // Built-in globals (available through interop)
        self.add_builtin_module(BuiltinModule {
            name: "console".to_string(),
            js_equivalent: "console".to_string(),
            exports: vec!["log".to_string(), "error".to_string(), "warn".to_string()],
            interop_required: true,
        });

        self.add_builtin_module(BuiltinModule {
            name: "Math".to_string(),
            js_equivalent: "Math".to_string(),
            exports: vec!["sin".to_string(), "cos".to_string(), "max".to_string(), "min".to_string()],
            interop_required: true,
        });

        self.add_builtin_module(BuiltinModule {
            name: "JSON".to_string(),
            js_equivalent: "JSON".to_string(),
            exports: vec!["parse".to_string(), "stringify".to_string()],
            interop_required: true,
        });

        self.add_builtin_module(BuiltinModule {
            name: "Promise".to_string(),
            js_equivalent: "Promise".to_string(),
            exports: vec!["resolve".to_string(), "reject".to_string(), "all".to_string(), "race".to_string()],
            interop_required: true,
        });
    }

    fn add_builtin_module(&mut self, module: BuiltinModule) {
        self.builtin_modules.insert(module.name.clone(), module);
    }

    pub fn is_builtin_module(&self, name: &str) -> bool {
        self.builtin_modules.contains_key(name)
    }

    pub fn get_builtin_module(&self, name: &str) -> Option<&BuiltinModule> {
        self.builtin_modules.get(name)
    }

    pub fn resolve_import(&self, import: &ImportStatement) -> String {
        if let Some(builtin) = self.get_builtin_module(&import.module) {
            if builtin.interop_required {
                self.generate_interop_import(import, builtin)
            } else {
                self.generate_standard_import(import, builtin)
            }
        } else {
            self.generate_external_import(import)
        }
    }

    fn generate_interop_import(&self, import: &ImportStatement, builtin: &BuiltinModule) -> String {
        if let Some(items) = &import.items {
            if import.module == "react" {
                format!(
                    "const {{ {} }} = ReactInterop;",
                    items.join(", ")
                )
            } else {
                format!(
                    "const {{ {} }} = InteropRegistry.getModule(\"{}\") || {{}};",
                    items.join(", "),
                    builtin.name
                )
            }
        } else {
            if import.module == "react" {
                "const React = ReactInterop;".to_string()
            } else {
                format!(
                    "const {} = InteropRegistry.getModule(\"{}\");",
                    import.module,
                    builtin.name
                )
            }
        }
    }

    fn generate_standard_import(&self, import: &ImportStatement, builtin: &BuiltinModule) -> String {
        match self.target.as_str() {
            "esm" | "es6" => {
                if let Some(items) = &import.items {
                    format!(
                        "import {{ {} }} from \"{}\";",
                        items.join(", "),
                        builtin.js_equivalent
                    )
                } else {
                    format!(
                        "import {} from \"{}\";",
                        import.module,
                        builtin.js_equivalent
                    )
                }
            }
            "node" | "cjs" => {
                if let Some(items) = &import.items {
                    format!(
                        "const {{ {} }} = require(\"{}\");",
                        items.join(", "),
                        builtin.js_equivalent
                    )
                } else {
                    format!(
                        "const {} = require(\"{}\");",
                        import.module,
                        builtin.js_equivalent
                    )
                }
            }
            _ => self.generate_external_import(import)
        }
    }

    fn generate_external_import(&self, import: &ImportStatement) -> String {
        match self.target.as_str() {
            "esm" | "es6" => {
                if let Some(items) = &import.items {
                    format!(
                        "import {{ {} }} from \"{}\";",
                        items.join(", "),
                        import.module
                    )
                } else {
                    format!(
                        "import {} from \"{}\";",
                        import.module,
                        import.module
                    )
                }
            }
            "node" | "cjs" => {
                if let Some(items) = &import.items {
                    format!(
                        "const {{ {} }} = require(\"{}\");",
                        items.join(", "),
                        import.module
                    )
                } else {
                    format!(
                        "const {} = require(\"{}\");",
                        import.module,
                        import.module
                    )
                }
            }
            _ => {
                if let Some(items) = &import.items {
                    format!(
                        "import {{ {} }} from \"{}\";",
                        items.join(", "),
                        import.module
                    )
                } else {
                    format!(
                        "import {} from \"{}\";",
                        import.module,
                        import.module
                    )
                }
            }
        }
    }

    pub fn get_runtime_imports(&self, jsx_enabled: bool) -> String {
        let mut imports = vec![
            "jsToNagari",
            "nagariToJS",
            "InteropRegistry"
        ];

        if jsx_enabled {
            imports.extend_from_slice(&["jsx", "Fragment", "jsxToReact", "ReactInterop"]);
        }

        match self.target.as_str() {
            "esm" | "es6" => {
                format!("import {{ {} }} from 'nagari-runtime';", imports.join(", "))
            }
            "node" | "cjs" => {
                format!("const {{ {} }} = require('nagari-runtime');", imports.join(", "))
            }
            _ => {
                format!("import {{ {} }} from 'nagari-runtime';", imports.join(", "))
            }
        }
    }
}
