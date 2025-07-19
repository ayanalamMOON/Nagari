use crate::builtins::{call_builtin, setup_builtins};
use crate::bytecode::{BytecodeFile, Instruction, Opcode};
use crate::env::Environment;
use crate::value::Value;

pub struct VM {
    stack: Vec<Value>,
    environment: Environment,
    bytecode: Option<BytecodeFile>,
    instruction_pointer: usize,
    debug: bool,
}

impl VM {
    pub fn new(debug: bool) -> Self {
        let mut vm = Self {
            stack: Vec::new(),
            environment: Environment::new(),
            bytecode: None,
            instruction_pointer: 0,
            debug,
        };

        // Setup built-in functions
        for (name, value) in setup_builtins() {
            vm.environment.define_global(name, value);
        }

        vm
    }

    pub fn load_bytecode(&mut self, data: &[u8]) -> Result<(), String> {
        self.bytecode = Some(BytecodeFile::load(data)?);
        self.instruction_pointer = 0;
        Ok(())
    }
    pub async fn run(&mut self) -> Result<(), String> {
        let bytecode_len = if let Some(bytecode) = &self.bytecode {
            if self.debug {
                println!("🐛 Debug mode enabled");
                println!("📊 Constants: {}", bytecode.constants.len());
                println!("📛 Names: {}", bytecode.names.len());
                println!("📋 Instructions: {}", bytecode.instructions.len());
                println!();
            }
            bytecode.instructions.len()
        } else {
            return Err("No bytecode loaded".to_string());
        };

        while self.instruction_pointer < bytecode_len {
            let instruction = if let Some(bytecode) = &self.bytecode {
                bytecode.instructions[self.instruction_pointer].clone()
            } else {
                return Err("Bytecode disappeared during execution".to_string());
            };

            if self.debug {
                self.debug_instruction(&instruction);
            }

            match self.execute_instruction(&instruction).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Runtime error at instruction {}: {}",
                        self.instruction_pointer, e
                    ));
                }
            }

            self.instruction_pointer += 1;
        }

        Ok(())
    }

    async fn execute_instruction(&mut self, instruction: &Instruction) -> Result<bool, String> {
        let bytecode = self.bytecode.as_ref().unwrap();

        match instruction.opcode {
            Opcode::LoadConst => {
                let const_index = instruction.operand as usize;
                if const_index >= bytecode.constants.len() {
                    return Err(format!("Constant index out of bounds: {const_index}"));
                }
                let value = bytecode.constants[const_index].clone();
                self.stack.push(value);
            }

            Opcode::LoadName => {
                let name_index = instruction.operand as usize;
                if name_index >= bytecode.names.len() {
                    return Err(format!("Name index out of bounds: {name_index}"));
                }
                let name = &bytecode.names[name_index];

                if let Some(value) = self.environment.get(name) {
                    self.stack.push(value.clone());
                } else {
                    return Err(format!("Undefined variable: {name}"));
                }
            }

            Opcode::StoreName => {
                let name_index = instruction.operand as usize;
                if name_index >= bytecode.names.len() {
                    return Err(format!("Name index out of bounds: {name_index}"));
                }
                let name = &bytecode.names[name_index];

                if let Some(value) = self.stack.pop() {
                    self.environment.set(name, value)?;
                } else {
                    return Err("Stack underflow in StoreName".to_string());
                }
            }

            Opcode::Pop => {
                if self.stack.pop().is_none() {
                    return Err("Stack underflow in Pop".to_string());
                }
            }

            Opcode::Print => {
                let arg_count = instruction.operand as usize;
                if self.stack.len() < arg_count {
                    return Err("Stack underflow in Print".to_string());
                }

                let args: Vec<Value> = (0..arg_count)
                    .map(|_| self.stack.pop().unwrap())
                    .rev()
                    .collect();

                call_builtin("print", &args).await?;
                self.stack.push(Value::None);
            }

            Opcode::CallFunc => {
                let arg_count = instruction.operand as usize;
                if self.stack.len() < arg_count + 1 {
                    return Err("Stack underflow in CallFunc".to_string());
                }

                let args: Vec<Value> = (0..arg_count)
                    .map(|_| self.stack.pop().unwrap())
                    .rev()
                    .collect();

                let function = self.stack.pop().unwrap();

                match function {
                    Value::Builtin(builtin) => {
                        let result = call_builtin(&builtin.name, &args).await?;
                        self.stack.push(result);
                    }
                    _ => {
                        return Err(format!(
                            "Cannot call non-function value: {}",
                            function.type_name()
                        ));
                    }
                }
            }

            Opcode::Return => {
                return Ok(false); // Stop execution
            }

            Opcode::Jump => {
                self.instruction_pointer = instruction.operand as usize;
                return Ok(true); // Continue without incrementing IP
            }

            Opcode::JumpIfFalse => {
                if let Some(condition) = self.stack.pop() {
                    if !condition.is_truthy() {
                        self.instruction_pointer = instruction.operand as usize;
                        return Ok(true); // Continue without incrementing IP
                    }
                } else {
                    return Err("Stack underflow in JumpIfFalse".to_string());
                }
            }

            // Binary operations
            Opcode::BinaryAdd => self.binary_operation(|a, b| a.add(b))?,
            Opcode::BinarySubtract => self.binary_operation(|a, b| a.subtract(b))?,
            Opcode::BinaryMultiply => self.binary_operation(|a, b| a.multiply(b))?,
            Opcode::BinaryDivide => self.binary_operation(|a, b| a.divide(b))?,
            Opcode::BinaryModulo => self.binary_operation(|a, b| a.modulo(b))?,
            Opcode::BinaryEqual => self.binary_operation(|a, b| Ok(a.equals(b)))?,
            Opcode::BinaryNotEqual => self.binary_operation(|a, b| Ok(a.not_equals(b)))?,
            Opcode::BinaryLess => self.binary_operation(|a, b| a.less(b))?,
            Opcode::BinaryGreater => self.binary_operation(|a, b| a.greater(b))?,
            Opcode::BinaryLessEqual => self.binary_operation(|a, b| a.less_equal(b))?,
            Opcode::BinaryGreaterEqual => self.binary_operation(|a, b| a.greater_equal(b))?,

            Opcode::BuildList => {
                let count = instruction.operand as usize;
                if self.stack.len() < count {
                    return Err("Stack underflow in BuildList".to_string());
                }

                let mut list = Vec::with_capacity(count);
                for _ in 0..count {
                    list.insert(0, self.stack.pop().unwrap());
                }

                self.stack.push(Value::List(list));
            }

            Opcode::BuildDict => {
                let count = instruction.operand as usize;
                if self.stack.len() < count * 2 {
                    return Err("Stack underflow in BuildDict".to_string());
                }

                let mut dict = std::collections::HashMap::new();
                for _ in 0..count {
                    let value = self.stack.pop().unwrap();
                    let key = self.stack.pop().unwrap();

                    if let Value::String(key_str) = key {
                        dict.insert(key_str, value);
                    } else {
                        return Err("Dictionary keys must be strings".to_string());
                    }
                }

                self.stack.push(Value::Dict(dict));
            }

            _ => {
                return Err(format!("Unimplemented opcode: {:?}", instruction.opcode));
            }
        }

        Ok(true)
    }

    fn binary_operation<F>(&mut self, op: F) -> Result<(), String>
    where
        F: FnOnce(&Value, &Value) -> Result<Value, String>,
    {
        if self.stack.len() < 2 {
            return Err("Stack underflow in binary operation".to_string());
        }

        let right = self.stack.pop().unwrap();
        let left = self.stack.pop().unwrap();

        let result = op(&left, &right)?;
        self.stack.push(result);

        Ok(())
    }

    fn debug_instruction(&self, instruction: &Instruction) {
        let bytecode = self.bytecode.as_ref().unwrap();

        print!("{:04} {:?}", self.instruction_pointer, instruction.opcode);

        match instruction.opcode {
            Opcode::LoadConst => {
                if let Some(constant) = bytecode.constants.get(instruction.operand as usize) {
                    print!(" ({constant})");
                }
            }
            Opcode::LoadName | Opcode::StoreName => {
                if let Some(name) = bytecode.names.get(instruction.operand as usize) {
                    print!(" ({name})");
                }
            }
            Opcode::Jump | Opcode::JumpIfFalse => {
                print!(" -> {}", instruction.operand);
            }
            _ => {
                if instruction.operand != 0 {
                    print!(" {}", instruction.operand);
                }
            }
        }

        println!();
        println!("  Stack: {:?}", self.stack);
    } // Public methods for external access
    #[allow(dead_code)] // Used by WASM, embedded, and REPL modules
    pub fn define_global(&mut self, name: &str, value: Value) {
        self.environment.define_global(name, value);
    }

    #[allow(dead_code)] // Used by WASM, embedded, and REPL modules
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.environment.get(name)
    }

    #[allow(dead_code)] // Used by WASM, embedded, and REPL modules
    pub fn set_global(&mut self, name: &str, value: Value) -> Result<(), String> {
        self.environment.set(name, value)
    }

    #[allow(dead_code)] // Used by WASM, embedded, and REPL modules
    pub fn clear_globals(&mut self) {
        self.environment = Environment::new();
        // Re-setup built-ins after clearing
        for (name, value) in setup_builtins() {
            self.environment.define_global(name, value);
        }
    }
}
