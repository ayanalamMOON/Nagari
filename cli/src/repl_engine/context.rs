#![allow(dead_code)]

use crate::repl_engine::ReplValue;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutionContext {
    pub variables: HashMap<String, Variable>,
    pub imports: HashMap<String, ImportInfo>,
    pub functions: HashMap<String, FunctionInfo>,
    pub classes: HashMap<String, ClassInfo>,
    pub current_scope: ScopeInfo,
    pub global_scope: ScopeInfo,
    pub working_directory: std::path::PathBuf,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Variable {
    pub name: String,
    pub value: ReplValue,
    pub var_type: VariableType,
    pub scope: String,
    pub mutable: bool,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum VariableType {
    Local,
    Global,
    Imported,
    Function,
    Class,
    Constant,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportInfo {
    pub module_name: String,
    pub imported_names: Vec<String>,
    pub alias: Option<String>,
    pub source_path: std::path::PathBuf,
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub body: String,
    pub scope: String,
    pub defined_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<ReplValue>,
    pub optional: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClassInfo {
    pub name: String,
    pub methods: HashMap<String, FunctionInfo>,
    pub properties: HashMap<String, Variable>,
    pub parent_class: Option<String>,
    pub interfaces: Vec<String>,
    pub defined_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScopeInfo {
    pub name: String,
    pub parent: Option<String>,
    pub variables: HashMap<String, String>, // variable name -> variable id
    pub functions: HashMap<String, String>, // function name -> function id
    pub classes: HashMap<String, String>,   // class name -> class id
    pub created_at: DateTime<Utc>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        let now = Utc::now();
        let global_scope = ScopeInfo {
            name: "global".to_string(),
            parent: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            created_at: now,
        };

        Self {
            variables: HashMap::new(),
            imports: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            current_scope: global_scope.clone(),
            global_scope,
            working_directory: std::env::current_dir().unwrap_or_default(),
            environment: std::env::vars().collect(),
        }
    }

    pub fn define_variable(&mut self, name: String, value: ReplValue, mutable: bool) -> String {
        let now = Utc::now();
        let var_id = format!("var_{}_{}", name, now.timestamp_millis());

        let variable = Variable {
            name: name.clone(),
            value,
            var_type: VariableType::Local,
            scope: self.current_scope.name.clone(),
            mutable,
            created_at: now,
            last_modified: now,
        };

        self.variables.insert(var_id.clone(), variable);
        self.current_scope.variables.insert(name, var_id.clone());

        var_id
    }

    pub fn get_variable(&self, name: &str) -> Option<&Variable> {
        // First check current scope
        if let Some(var_id) = self.current_scope.variables.get(name) {
            return self.variables.get(var_id);
        }

        // Then check global scope
        if let Some(var_id) = self.global_scope.variables.get(name) {
            return self.variables.get(var_id);
        }

        None
    }

    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        // First check current scope
        if let Some(var_id) = self.current_scope.variables.get(name).cloned() {
            return self.variables.get_mut(&var_id);
        }

        // Then check global scope
        if let Some(var_id) = self.global_scope.variables.get(name).cloned() {
            return self.variables.get_mut(&var_id);
        }

        None
    }

    pub fn update_variable(&mut self, name: &str, value: ReplValue) -> Result<(), String> {
        if let Some(variable) = self.get_variable_mut(name) {
            if !variable.mutable {
                return Err(format!("Cannot modify immutable variable '{}'", name));
            }

            variable.value = value;
            variable.last_modified = Utc::now();
            Ok(())
        } else {
            Err(format!("Variable '{}' not found", name))
        }
    }

    pub fn define_function(&mut self, name: String, info: FunctionInfo) -> String {
        let func_id = format!("func_{}_{}", name, Utc::now().timestamp_millis());
        self.functions.insert(func_id.clone(), info);
        self.current_scope.functions.insert(name, func_id.clone());
        func_id
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        if let Some(func_id) = self.current_scope.functions.get(name) {
            self.functions.get(func_id)
        } else if let Some(func_id) = self.global_scope.functions.get(name) {
            self.functions.get(func_id)
        } else {
            None
        }
    }

    pub fn define_class(&mut self, name: String, info: ClassInfo) -> String {
        let class_id = format!("class_{}_{}", name, Utc::now().timestamp_millis());
        self.classes.insert(class_id.clone(), info);
        self.current_scope.classes.insert(name, class_id.clone());
        class_id
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        if let Some(class_id) = self.current_scope.classes.get(name) {
            self.classes.get(class_id)
        } else if let Some(class_id) = self.global_scope.classes.get(name) {
            self.classes.get(class_id)
        } else {
            None
        }
    }

    pub fn add_import(&mut self, import_info: ImportInfo) {
        self.imports.insert(import_info.module_name.clone(), import_info);
    }

    pub fn get_import(&self, module_name: &str) -> Option<&ImportInfo> {
        self.imports.get(module_name)
    }

    pub fn list_variables(&self) -> Vec<&Variable> {
        // Return variables from current scope and global scope
        let mut vars = Vec::new();

        for var_id in self.current_scope.variables.values() {
            if let Some(var) = self.variables.get(var_id) {
                vars.push(var);
            }
        }

        for var_id in self.global_scope.variables.values() {
            if let Some(var) = self.variables.get(var_id) {
                if !vars.iter().any(|v| v.name == var.name) {
                    vars.push(var);
                }
            }
        }

        vars
    }

    pub fn list_functions(&self) -> Vec<&FunctionInfo> {
        let mut funcs = Vec::new();

        for func_id in self.current_scope.functions.values() {
            if let Some(func) = self.functions.get(func_id) {
                funcs.push(func);
            }
        }

        for func_id in self.global_scope.functions.values() {
            if let Some(func) = self.functions.get(func_id) {
                if !funcs.iter().any(|f| f.name == func.name) {
                    funcs.push(func);
                }
            }
        }

        funcs
    }

    pub fn list_classes(&self) -> Vec<&ClassInfo> {
        let mut classes = Vec::new();

        for class_id in self.current_scope.classes.values() {
            if let Some(class) = self.classes.get(class_id) {
                classes.push(class);
            }
        }

        for class_id in self.global_scope.classes.values() {
            if let Some(class) = self.classes.get(class_id) {
                if !classes.iter().any(|c| c.name == class.name) {
                    classes.push(class);
                }
            }
        }

        classes
    }

    pub fn list_imports(&self) -> Vec<&ImportInfo> {
        self.imports.values().collect()
    }

    pub fn clear_scope(&mut self) {
        // Clear current scope but keep global scope
        self.current_scope.variables.clear();
        self.current_scope.functions.clear();
        self.current_scope.classes.clear();
    }

    pub fn reset(&mut self) {
        // Reset everything
        self.variables.clear();
        self.imports.clear();
        self.functions.clear();
        self.classes.clear();
        self.current_scope = self.global_scope.clone();
        self.global_scope.variables.clear();
        self.global_scope.functions.clear();
        self.global_scope.classes.clear();
    }

    pub fn get_environment_variable(&self, name: &str) -> Option<&String> {
        self.environment.get(name)
    }

    pub fn set_environment_variable(&mut self, name: String, value: String) {
        self.environment.insert(name, value);
    }

    pub fn get_working_directory(&self) -> &std::path::PathBuf {
        &self.working_directory
    }

    pub fn set_working_directory(&mut self, path: std::path::PathBuf) {
        self.working_directory = path;
    }

    pub fn get_scope_info(&self) -> String {
        format!("Current scope: {}", self.current_scope.name)
    }

    pub fn enter_scope(&mut self, scope_name: String) {
        let now = Utc::now();
        let new_scope = ScopeInfo {
            name: scope_name,
            parent: Some(self.current_scope.name.clone()),
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            created_at: now,
        };

        self.current_scope = new_scope;
    }    pub fn exit_scope(&mut self) {
        if let Some(parent_name) = &self.current_scope.parent {
            // In a real implementation, we'd restore the parent scope
            // For now, just reset to global scope
            if parent_name == "global" {
                self.current_scope = self.global_scope.clone();
            }
        }
    }    // VM integration methods for global variable management
    pub fn sync_with_vm(&mut self, vm: &mut nagari_vm::VM) {
        // Sync all global variables to the VM
        for (_var_id, variable) in &self.variables {
            if matches!(variable.var_type, VariableType::Global) {
                // Convert ReplValue to VM Value and sync
                if let Ok(vm_value) = self.repl_value_to_vm_value(&variable.value) {
                    vm.define_global(&variable.name, vm_value);
                }
            }
        }
    }

    pub fn sync_global_from_vm(&mut self, vm: &nagari_vm::VM, name: &str) -> Option<ReplValue> {
        // Get a global variable from the VM and sync it to our context
        if let Some(vm_value) = vm.get_global(name) {
            let repl_value = self.vm_value_to_repl_value(vm_value);

            // Update or create the variable in our context
            let var_id = format!("var_{}_{}", name, Utc::now().timestamp_millis());
            let variable = Variable {
                name: name.to_string(),
                value: repl_value.clone(),
                var_type: VariableType::Global,
                scope: "global".to_string(),
                mutable: true,
                created_at: Utc::now(),
                last_modified: Utc::now(),
            };

            self.variables.insert(var_id.clone(), variable);
            self.global_scope.variables.insert(name.to_string(), var_id);

            Some(repl_value)
        } else {
            None
        }
    }

    pub fn set_global_in_vm(&mut self, vm: &mut nagari_vm::VM, name: &str, value: ReplValue) -> Result<(), String> {
        // Set a global variable in both the VM and our context
        let vm_value = self.repl_value_to_vm_value(&value)?;
        vm.set_global(name, vm_value)?;

        // Also update in our context
        let var_id = format!("var_{}_{}", name, Utc::now().timestamp_millis());
        let variable = Variable {
            name: name.to_string(),
            value,
            var_type: VariableType::Global,
            scope: "global".to_string(),
            mutable: true,
            created_at: Utc::now(),
            last_modified: Utc::now(),
        };

        self.variables.insert(var_id.clone(), variable);
        self.global_scope.variables.insert(name.to_string(), var_id);

        Ok(())
    }

    pub fn clear_vm_globals(&mut self, vm: &mut nagari_vm::VM) {
        // Clear VM globals and remove global variables from our context
        vm.clear_globals();

        // Remove global variables from our context
        let global_var_ids: Vec<String> = self.global_scope.variables.values().cloned().collect();
        for var_id in global_var_ids {
            self.variables.remove(&var_id);
        }
        self.global_scope.variables.clear();
    }    // Helper methods for value conversion between REPL and VM
    pub fn repl_value_to_vm_value(&self, value: &ReplValue) -> Result<nagari_vm::Value, String> {
        match value {
            ReplValue::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(nagari_vm::Value::Int(*n as i64))
                } else {
                    Ok(nagari_vm::Value::Float(*n))
                }
            }
            ReplValue::String(s) => Ok(nagari_vm::Value::String(s.clone())),
            ReplValue::Boolean(b) => Ok(nagari_vm::Value::Bool(*b)),
            ReplValue::List(items) => {
                let vm_items: Result<Vec<_>, _> = items
                    .iter()
                    .map(|item| self.repl_value_to_vm_value(item))
                    .collect();
                Ok(nagari_vm::Value::List(vm_items?))
            }
            ReplValue::Null | ReplValue::Undefined => Ok(nagari_vm::Value::None),
            ReplValue::Object(_) | ReplValue::Function(_) => {
                Err("Complex types not yet supported for VM sync".to_string())
            }
        }
    }

    pub fn vm_value_to_repl_value(&self, value: &nagari_vm::Value) -> ReplValue {
        match value {
            nagari_vm::Value::Int(i) => ReplValue::Number(*i as f64),
            nagari_vm::Value::Float(f) => ReplValue::Number(*f),
            nagari_vm::Value::String(s) => ReplValue::String(s.clone()),
            nagari_vm::Value::Bool(b) => ReplValue::Boolean(*b),
            nagari_vm::Value::List(items) => {
                let repl_items: Vec<_> = items
                    .iter()
                    .map(|item| self.vm_value_to_repl_value(item))
                    .collect();
                ReplValue::List(repl_items)
            }
            nagari_vm::Value::None => ReplValue::Null,
            _ => ReplValue::Undefined, // For unsupported types
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
