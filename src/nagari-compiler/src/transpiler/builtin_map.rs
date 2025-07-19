// Nagari builtin to JavaScript mapping

use std::collections::HashMap;

pub struct BuiltinMapper {
    mappings: HashMap<String, BuiltinMapping>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct BuiltinMapping {
    pub js_equivalent: String,
    pub requires_import: Option<String>,
    pub requires_helper: bool,
    pub is_method: bool,
}

impl BuiltinMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            mappings: HashMap::new(),
        };
        mapper.init_mappings();
        mapper
    }

    fn init_mappings(&mut self) {
        // Type constructors
        self.add_mapping(
            "str",
            BuiltinMapping {
                js_equivalent: "String".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "int",
            BuiltinMapping {
                js_equivalent: "parseInt".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "float",
            BuiltinMapping {
                js_equivalent: "parseFloat".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "bool",
            BuiltinMapping {
                js_equivalent: "Boolean".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "list",
            BuiltinMapping {
                js_equivalent: "Array".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        // Built-in functions
        self.add_mapping(
            "print",
            BuiltinMapping {
                js_equivalent: "console.log".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "len",
            BuiltinMapping {
                js_equivalent: ".length".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "range",
            BuiltinMapping {
                js_equivalent: "range".to_string(),
                requires_import: None,
                requires_helper: true,
                is_method: false,
            },
        );

        self.add_mapping(
            "enumerate",
            BuiltinMapping {
                js_equivalent: "enumerate".to_string(),
                requires_import: None,
                requires_helper: true,
                is_method: false,
            },
        );

        self.add_mapping(
            "zip",
            BuiltinMapping {
                js_equivalent: "zip".to_string(),
                requires_import: None,
                requires_helper: true,
                is_method: false,
            },
        );

        self.add_mapping(
            "sum",
            BuiltinMapping {
                js_equivalent: "sum".to_string(),
                requires_import: None,
                requires_helper: true,
                is_method: false,
            },
        );

        // Math functions
        self.add_mapping(
            "abs",
            BuiltinMapping {
                js_equivalent: "Math.abs".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "max",
            BuiltinMapping {
                js_equivalent: "Math.max".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "min",
            BuiltinMapping {
                js_equivalent: "Math.min".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "round",
            BuiltinMapping {
                js_equivalent: "Math.round".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        // Array methods
        self.add_mapping(
            "append",
            BuiltinMapping {
                js_equivalent: ".push".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "extend",
            BuiltinMapping {
                js_equivalent: ".push(...".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "insert",
            BuiltinMapping {
                js_equivalent: ".splice".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "pop",
            BuiltinMapping {
                js_equivalent: ".pop".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "remove",
            BuiltinMapping {
                js_equivalent: ".splice".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "index",
            BuiltinMapping {
                js_equivalent: ".indexOf".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "count",
            BuiltinMapping {
                js_equivalent: ".filter".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "sort",
            BuiltinMapping {
                js_equivalent: ".sort".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "reverse",
            BuiltinMapping {
                js_equivalent: ".reverse".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        // String methods
        self.add_mapping(
            "split",
            BuiltinMapping {
                js_equivalent: ".split".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "join",
            BuiltinMapping {
                js_equivalent: ".join".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "strip",
            BuiltinMapping {
                js_equivalent: ".trim".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "lower",
            BuiltinMapping {
                js_equivalent: ".toLowerCase".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "upper",
            BuiltinMapping {
                js_equivalent: ".toUpperCase".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "replace",
            BuiltinMapping {
                js_equivalent: ".replace".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "startswith",
            BuiltinMapping {
                js_equivalent: ".startsWith".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "endswith",
            BuiltinMapping {
                js_equivalent: ".endsWith".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        // Dict methods
        self.add_mapping(
            "keys",
            BuiltinMapping {
                js_equivalent: "Object.keys".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "values",
            BuiltinMapping {
                js_equivalent: "Object.values".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "items",
            BuiltinMapping {
                js_equivalent: "Object.entries".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        // Type checking
        // isinstance is handled specially in transpile_call
        // self.add_mapping(
        //     "isinstance",
        //     BuiltinMapping {
        //         js_equivalent: "instanceof".to_string(),
        //         requires_import: None,
        //         requires_helper: false,
        //         is_method: false,
        //     },
        // );

        self.add_mapping(
            "hasattr",
            BuiltinMapping {
                js_equivalent: "in".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: false,
            },
        );

        // Iteration
        self.add_mapping(
            "any",
            BuiltinMapping {
                js_equivalent: ".some".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "all",
            BuiltinMapping {
                js_equivalent: ".every".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "map",
            BuiltinMapping {
                js_equivalent: ".map".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "filter",
            BuiltinMapping {
                js_equivalent: ".filter".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        self.add_mapping(
            "reduce",
            BuiltinMapping {
                js_equivalent: ".reduce".to_string(),
                requires_import: None,
                requires_helper: false,
                is_method: true,
            },
        );

        // String manipulation functions
        self.add_mapping(
            "str_capitalize",
            BuiltinMapping {
                js_equivalent: "str_capitalize".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "str_title",
            BuiltinMapping {
                js_equivalent: "str_title".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "str_reverse",
            BuiltinMapping {
                js_equivalent: "str_reverse".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "str_count",
            BuiltinMapping {
                js_equivalent: "str_count".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "str_pad_left",
            BuiltinMapping {
                js_equivalent: "str_pad_left".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "str_pad_right",
            BuiltinMapping {
                js_equivalent: "str_pad_right".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );

        self.add_mapping(
            "str_center",
            BuiltinMapping {
                js_equivalent: "str_center".to_string(),
                requires_import: Some("nagari-runtime".to_string()),
                requires_helper: false,
                is_method: false,
            },
        );
    }

    fn add_mapping(&mut self, name: &str, mapping: BuiltinMapping) {
        self.mappings.insert(name.to_string(), mapping);
    }

    pub fn get_mapping(&self, name: &str) -> Option<&BuiltinMapping> {
        self.mappings.get(name)
    }

    #[allow(dead_code)]
    pub fn is_builtin(&self, name: &str) -> bool {
        self.mappings.contains_key(name)
    }

    #[allow(dead_code)]
    pub fn requires_helper(&self, name: &str) -> bool {
        self.mappings
            .get(name)
            .map(|mapping| mapping.requires_helper)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn get_all_required_helpers(&self) -> Vec<String> {
        self.mappings
            .iter()
            .filter(|(_, mapping)| mapping.requires_helper)
            .map(|(name, _)| name.clone())
            .collect()
    }
}
