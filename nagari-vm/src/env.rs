use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    #[allow(dead_code)]
    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn define(&mut self, name: &str, value: Value) {
        if let Some(locals) = self.locals.last_mut() {
            locals.insert(name.to_string(), value);
        } else {
            self.globals.insert(name.to_string(), value);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        // Check local scopes from innermost to outermost
        for locals in self.locals.iter().rev() {
            if let Some(value) = locals.get(name) {
                return Some(value);
            }
        }

        // Check globals
        self.globals.get(name)
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        // Check local scopes from innermost to outermost
        for locals in self.locals.iter_mut().rev() {
            if locals.contains_key(name) {
                locals.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Check globals
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            Ok(())
        } else {
            // If variable doesn't exist, create it in the current scope
            self.define(name, value);
            Ok(())
        }
    }

    pub fn define_global(&mut self, name: &str, value: Value) {
        self.globals.insert(name.to_string(), value);
    }
}
