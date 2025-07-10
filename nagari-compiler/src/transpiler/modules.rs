// Module management for JavaScript transpilation

use crate::ast::ImportStatement;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ModuleResolver {
    builtin_modules: HashMap<String, BuiltinModule>,
    target: String,
}

#[derive(Debug, Clone)]
pub struct BuiltinModule {
    pub name: String,
    pub path: PathBuf, // Path to the Nagari source file for the module
    #[allow(dead_code)]
    pub exports: Vec<String>, // List of exported symbols, if known statically
    pub js_path: Option<PathBuf>, // Path to pre-transpiled JS version, if available
    pub interop_required: bool, // Whether the module requires JS interop
    pub js_equivalent: Option<String>, // JS equivalent module name/path
}

#[allow(dead_code)]
impl ModuleResolver {
    pub fn new(target: &str) -> Self {
        let mut resolver = Self {
            builtin_modules: HashMap::new(),
            target: target.to_string(),
        };
        resolver.init_builtin_modules();
        resolver
    }    fn init_builtin_modules(&mut self) {
        // React ecosystem
        self.add_builtin_module(BuiltinModule {
            name: "react".to_string(),
            path: PathBuf::from("react"),
            exports: vec![
                "React".to_string(),
                "useState".to_string(),
                "useEffect".to_string(),
            ],
            js_path: None,
            interop_required: true,
            js_equivalent: Some("react".to_string()),
        });

        // Node.js modules
        self.add_builtin_module(BuiltinModule {
            name: "fs".to_string(),            path: PathBuf::from(if self.target == "node" {
                "fs"
            } else {
                "fs/promises"
            }),
            exports: vec![
                "readFile".to_string(),
                "writeFile".to_string(),
                "mkdir".to_string(),
            ],
            js_path: None,
            interop_required: true,
            js_equivalent: Some("fs".to_string()),
        });

        self.add_builtin_module(BuiltinModule {
            name: "http".to_string(),
            path: PathBuf::from("http"),
            exports: vec![
                "createServer".to_string(),
                "get".to_string(),
                "post".to_string(),
            ],
            js_path: None,
            interop_required: true,
            js_equivalent: Some("http".to_string()),
        });

        self.add_builtin_module(BuiltinModule {
            name: "path".to_string(),
            path: PathBuf::from("path"),
            exports: vec![
                "join".to_string(),
                "resolve".to_string(),
                "dirname".to_string(),
            ],
            js_path: None,
            interop_required: true,
            js_equivalent: Some("path".to_string()),
        });

        self.add_builtin_module(BuiltinModule {
            name: "os".to_string(),
            path: PathBuf::from("os"),
            exports: vec![
                "platform".to_string(),
                "arch".to_string(),
                "cpus".to_string(),
            ],
            js_path: None,
            interop_required: true,
            js_equivalent: Some("os".to_string()),
        });

        // Express framework
        self.add_builtin_module(BuiltinModule {
            name: "express".to_string(),
            path: PathBuf::from("express"),
            exports: vec!["express".to_string()],
            js_path: None,
            interop_required: true,
            js_equivalent: Some("express".to_string()),
        });

        // Built-in globals (available through interop)
        self.add_builtin_module(BuiltinModule {
            name: "console".to_string(),
            path: PathBuf::from("console"),
            exports: vec!["log".to_string(), "error".to_string(), "warn".to_string()],
            js_path: None,
            interop_required: false,
            js_equivalent: None,
        });

        self.add_builtin_module(BuiltinModule {
            name: "Math".to_string(),
            path: PathBuf::from("Math"),
            exports: vec![
                "sin".to_string(),
                "cos".to_string(),
                "max".to_string(),
                "min".to_string(),
            ],
            js_path: None,
            interop_required: false,
            js_equivalent: None,
        });

        self.add_builtin_module(BuiltinModule {
            name: "JSON".to_string(),
            path: PathBuf::from("JSON"),
            exports: vec!["parse".to_string(), "stringify".to_string()],
            js_path: None,
            interop_required: false,
            js_equivalent: None,
        });

        self.add_builtin_module(BuiltinModule {
            name: "Promise".to_string(),
            path: PathBuf::from("Promise"),
            exports: vec![
                "resolve".to_string(),
                "reject".to_string(),
                "all".to_string(),
                "race".to_string(),
            ],
            js_path: None,
            interop_required: false,
            js_equivalent: None,
        });
    }

    fn add_builtin_module(&mut self, module: BuiltinModule) {
        self.builtin_modules.insert(module.name.clone(), module);
    }

    #[allow(dead_code)]
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
                format!("const {{ {} }} = ReactInterop;", items.join(", "))
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
                    import.module, builtin.name
                )
            }
        }
    }
    fn generate_standard_import(
        &self,
        import: &ImportStatement,
        builtin: &BuiltinModule,
    ) -> String {
        let js_module = builtin.js_equivalent.as_ref().unwrap_or(&builtin.name);
        match self.target.as_str() {
            "esm" | "es6" => {
                if let Some(items) = &import.items {
                    format!("import {{ {} }} from \"{}\";", items.join(", "), js_module)
                } else {
                    format!("import {} from \"{}\";", import.module, js_module)
                }
            }
            "node" | "cjs" => {
                if let Some(items) = &import.items {
                    format!(
                        "const {{ {} }} = require(\"{}\");",
                        items.join(", "),
                        js_module
                    )
                } else {
                    format!("const {} = require(\"{}\");", import.module, js_module)
                }
            }
            _ => self.generate_external_import(import),
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
                    format!("import {} from \"{}\";", import.module, import.module)
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
                    format!("const {} = require(\"{}\");", import.module, import.module)
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
                    format!("import {} from \"{}\";", import.module, import.module)
                }
            }
        }
    }

    pub fn get_runtime_imports(&self, jsx_enabled: bool) -> String {
        let mut imports = vec!["jsToNagari", "nagariToJS", "InteropRegistry"];

        if jsx_enabled {
            imports.extend_from_slice(&["jsx", "Fragment", "jsxToReact", "ReactInterop"]);
        }

        match self.target.as_str() {
            "esm" | "es6" => {
                format!("import {{ {} }} from 'nagari-runtime';", imports.join(", "))
            }
            "node" | "cjs" => {
                format!(
                    "const {{ {} }} = require('nagari-runtime');",
                    imports.join(", ")
                )
            }
            _ => {
                format!("import {{ {} }} from 'nagari-runtime';", imports.join(", "))
            }
        }
    }

    pub fn resolve_module_path(&self, module_name: &str) -> Option<&PathBuf> {
        self.builtin_modules.get(module_name).map(|module| &module.path)
    }

    pub fn get_js_path(&self, module_name: &str) -> Option<&PathBuf> {
        self.builtin_modules.get(module_name)
            .and_then(|module| module.js_path.as_ref())
    }

    pub fn get_module_info(&self, module_name: &str) -> Option<&BuiltinModule> {
        self.builtin_modules.get(module_name)
    }

    pub fn list_builtin_modules(&self) -> Vec<(&String, &PathBuf)> {
        self.builtin_modules.iter()
            .map(|(name, module)| (name, &module.path))
            .collect()
    }
}

impl BuiltinModule {
    #[allow(dead_code)]
    pub fn get_source_path(&self) -> &PathBuf {
        &self.path
    }

    #[allow(dead_code)]
    pub fn get_compiled_path(&self) -> Option<&PathBuf> {
        self.js_path.as_ref()
    }

    #[allow(dead_code)]
    pub fn has_precompiled_js(&self) -> bool {
        self.js_path.is_some()
    }

    #[allow(dead_code)]
    pub fn get_effective_path(&self) -> &PathBuf {
        self.js_path.as_ref().unwrap_or(&self.path)
    }
}
