use crate::ast::*;
use crate::error::NagariError;

// Bytecode opcodes
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    LoadConst = 0x01,
    LoadName = 0x02,
    StoreName = 0x03,
    CallFunc = 0x04,
    Return = 0x05,
    JumpIfFalse = 0x06,
    Jump = 0x07,
    Pop = 0x08,
    BinaryAdd = 0x09,
    BinarySubtract = 0x0A,
    BinaryMultiply = 0x0B,
    BinaryDivide = 0x0C,
    BinaryModulo = 0x0D,
    BinaryEqual = 0x0E,
    BinaryNotEqual = 0x0F,
    BinaryLess = 0x10,
    BinaryGreater = 0x11,
    BinaryLessEqual = 0x12,
    BinaryGreaterEqual = 0x13,
    Print = 0x14,
    BuildList = 0x15,
    BuildDict = 0x16,
    GetItem = 0x17,
    SetItem = 0x18,
    ForIter = 0x19,
    BreakLoop = 0x1A,
    ContinueLoop = 0x1B,
    SetupLoop = 0x1C,
    PopBlock = 0x1D,
    Await = 0x1E,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: ConstantValue,
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
}

pub struct CodeGenerator {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
    names: Vec<String>,
    constant_map: std::collections::HashMap<String, usize>,
    name_map: std::collections::HashMap<String, usize>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            constant_map: std::collections::HashMap::new(),
            name_map: std::collections::HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<Vec<u8>, NagariError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }

        // Always end with a return
        self.emit(Opcode::Return, None);

        self.serialize()
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), NagariError> {
        match stmt {
            Statement::FunctionDef(func) => self.compile_function(func),
            Statement::Assignment(assign) => self.compile_assignment(assign),
            Statement::If(if_stmt) => self.compile_if(if_stmt),
            Statement::While(while_loop) => self.compile_while(while_loop),
            Statement::For(for_loop) => self.compile_for(for_loop),
            Statement::Match(match_stmt) => self.compile_match(match_stmt),
            Statement::Return(expr) => self.compile_return(expr),
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::Pop, None); // Pop unused expression result
                Ok(())
            }
            Statement::Import(_) => {
                // TODO: Implement import handling
                Ok(())
            }
            Statement::Break => {
                self.emit(Opcode::BreakLoop, None);
                Ok(())
            }
            Statement::Continue => {
                self.emit(Opcode::ContinueLoop, None);
                Ok(())
            }
        }
    }

    fn compile_function(&mut self, func: &FunctionDef) -> Result<(), NagariError> {
        // TODO: Implement function compilation
        // For now, just compile the body
        for statement in &func.body {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_assignment(&mut self, assign: &Assignment) -> Result<(), NagariError> {
        self.compile_expression(&assign.value)?;
        let name_index = self.add_name(&assign.name);
        self.emit(Opcode::StoreName, Some(name_index as u32));
        Ok(())
    }

    fn compile_if(&mut self, if_stmt: &IfStatement) -> Result<(), NagariError> {
        self.compile_expression(&if_stmt.condition)?;

        let jump_if_false = self.emit_jump(Opcode::JumpIfFalse);

        for statement in &if_stmt.then_branch {
            self.compile_statement(statement)?;
        }

        let jump_end = self.emit_jump(Opcode::Jump);
        self.patch_jump(jump_if_false);

        // Handle elif branches
        let mut elif_jumps = Vec::new();
        for elif_branch in &if_stmt.elif_branches {
            self.compile_expression(&elif_branch.condition)?;
            let elif_jump = self.emit_jump(Opcode::JumpIfFalse);

            for statement in &elif_branch.body {
                self.compile_statement(statement)?;
            }

            elif_jumps.push(self.emit_jump(Opcode::Jump));
            self.patch_jump(elif_jump);
        }

        // Handle else branch
        if let Some(else_branch) = &if_stmt.else_branch {
            for statement in else_branch {
                self.compile_statement(statement)?;
            }
        }

        self.patch_jump(jump_end);
        for jump in elif_jumps {
            self.patch_jump(jump);
        }

        Ok(())
    }

    fn compile_while(&mut self, while_loop: &WhileLoop) -> Result<(), NagariError> {
        let loop_start = self.instructions.len();

        self.compile_expression(&while_loop.condition)?;
        let exit_jump = self.emit_jump(Opcode::JumpIfFalse);

        for statement in &while_loop.body {
            self.compile_statement(statement)?;
        }

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);

        Ok(())
    }

    fn compile_for(&mut self, for_loop: &ForLoop) -> Result<(), NagariError> {
        // TODO: Implement for loop compilation
        self.compile_expression(&for_loop.iterable)?;
        self.emit(Opcode::ForIter, None);

        for statement in &for_loop.body {
            self.compile_statement(statement)?;
        }

        Ok(())
    }

    fn compile_match(&mut self, match_stmt: &MatchStatement) -> Result<(), NagariError> {
        self.compile_expression(&match_stmt.expression)?;

        // TODO: Implement pattern matching compilation
        for case in &match_stmt.cases {
            for statement in &case.body {
                self.compile_statement(statement)?;
            }
        }

        Ok(())
    }

    fn compile_return(&mut self, expr: &Option<Expression>) -> Result<(), NagariError> {
        if let Some(expr) = expr {
            self.compile_expression(expr)?;
        } else {
            let none_index = self.add_constant(ConstantValue::None);
            self.emit(Opcode::LoadConst, Some(none_index as u32));
        }
        self.emit(Opcode::Return, None);
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), NagariError> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Identifier(name) => {
                let name_index = self.add_name(name);
                self.emit(Opcode::LoadName, Some(name_index as u32));
                Ok(())
            }
            Expression::Binary(binary) => self.compile_binary(binary),
            Expression::Call(call) => self.compile_call(call),
            Expression::Await(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::Await, None);
                Ok(())
            }
            Expression::List(elements) => {
                for element in elements {
                    self.compile_expression(element)?;
                }
                self.emit(Opcode::BuildList, Some(elements.len() as u32));
                Ok(())
            }
            Expression::Dict(pairs) => {
                for (key, value) in pairs {
                    self.compile_expression(key)?;
                    self.compile_expression(value)?;
                }
                self.emit(Opcode::BuildDict, Some(pairs.len() as u32));
                Ok(())
            }
        }
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<(), NagariError> {
        let constant_value = match lit {
            Literal::Int(n) => ConstantValue::Int(*n),
            Literal::Float(f) => ConstantValue::Float(*f),
            Literal::String(s) => ConstantValue::String(s.clone()),
            Literal::Bool(b) => ConstantValue::Bool(*b),
            Literal::None => ConstantValue::None,
        };

        let const_index = self.add_constant(constant_value);
        self.emit(Opcode::LoadConst, Some(const_index as u32));
        Ok(())
    }

    fn compile_binary(&mut self, binary: &BinaryExpression) -> Result<(), NagariError> {
        self.compile_expression(&binary.left)?;
        self.compile_expression(&binary.right)?;

        let opcode = match binary.operator {
            BinaryOperator::Add => Opcode::BinaryAdd,
            BinaryOperator::Subtract => Opcode::BinarySubtract,
            BinaryOperator::Multiply => Opcode::BinaryMultiply,
            BinaryOperator::Divide => Opcode::BinaryDivide,
            BinaryOperator::Modulo => Opcode::BinaryModulo,
            BinaryOperator::Equal => Opcode::BinaryEqual,
            BinaryOperator::NotEqual => Opcode::BinaryNotEqual,
            BinaryOperator::Less => Opcode::BinaryLess,
            BinaryOperator::Greater => Opcode::BinaryGreater,
            BinaryOperator::LessEqual => Opcode::BinaryLessEqual,
            BinaryOperator::GreaterEqual => Opcode::BinaryGreaterEqual,
        };

        self.emit(opcode, None);
        Ok(())
    }

    fn compile_call(&mut self, call: &CallExpression) -> Result<(), NagariError> {
        // Special case for print function
        if let Expression::Identifier(name) = &*call.function {
            if name == "print" {
                for arg in &call.arguments {
                    self.compile_expression(arg)?;
                }
                self.emit(Opcode::Print, Some(call.arguments.len() as u32));
                return Ok(());
            }
        }

        self.compile_expression(&call.function)?;
        for arg in &call.arguments {
            self.compile_expression(arg)?;
        }
        self.emit(Opcode::CallFunc, Some(call.arguments.len() as u32));
        Ok(())
    }

    fn emit(&mut self, opcode: Opcode, operand: Option<u32>) -> usize {
        let instruction = Instruction { opcode, operand };
        self.instructions.push(instruction);
        self.instructions.len() - 1
    }

    fn emit_jump(&mut self, opcode: Opcode) -> usize {
        self.emit(opcode, Some(0xFFFF)) // Placeholder for jump target
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit(Opcode::Jump, Some(loop_start as u32));
    }

    fn patch_jump(&mut self, instruction_index: usize) {
        let jump_target = self.instructions.len();
        self.instructions[instruction_index].operand = Some(jump_target as u32);
    }

    fn add_constant(&mut self, value: ConstantValue) -> usize {
        let key = format!("{:?}", value);

        if let Some(&index) = self.constant_map.get(&key) {
            return index;
        }

        let index = self.constants.len();
        self.constants.push(Constant { value });
        self.constant_map.insert(key, index);
        index
    }

    fn add_name(&mut self, name: &str) -> usize {
        if let Some(&index) = self.name_map.get(name) {
            return index;
        }

        let index = self.names.len();
        self.names.push(name.to_string());
        self.name_map.insert(name.to_string(), index);
        index
    }

    fn serialize(&self) -> Result<Vec<u8>, NagariError> {
        let mut bytecode = Vec::new();

        // Magic number
        bytecode.extend_from_slice(b"NAG\x00");

        // Version
        bytecode.extend_from_slice(&[0, 1]); // Version 0.1

        // Constants section
        bytecode.extend_from_slice(&(self.constants.len() as u32).to_le_bytes());
        for constant in &self.constants {
            self.serialize_constant(&mut bytecode, constant)?;
        }

        // Names section
        bytecode.extend_from_slice(&(self.names.len() as u32).to_le_bytes());
        for name in &self.names {
            bytecode.extend_from_slice(&(name.len() as u32).to_le_bytes());
            bytecode.extend_from_slice(name.as_bytes());
        }

        // Instructions section
        bytecode.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());
        for instruction in &self.instructions {
            bytecode.push(instruction.opcode as u8);
            if let Some(operand) = instruction.operand {
                bytecode.extend_from_slice(&operand.to_le_bytes());
            } else {
                bytecode.extend_from_slice(&[0, 0, 0, 0]);
            }
        }

        Ok(bytecode)
    }

    fn serialize_constant(&self, bytecode: &mut Vec<u8>, constant: &Constant) -> Result<(), NagariError> {
        match &constant.value {
            ConstantValue::Int(n) => {
                bytecode.push(0); // Type tag for int
                bytecode.extend_from_slice(&n.to_le_bytes());
            }
            ConstantValue::Float(f) => {
                bytecode.push(1); // Type tag for float
                bytecode.extend_from_slice(&f.to_le_bytes());
            }
            ConstantValue::String(s) => {
                bytecode.push(2); // Type tag for string
                bytecode.extend_from_slice(&(s.len() as u32).to_le_bytes());
                bytecode.extend_from_slice(s.as_bytes());
            }
            ConstantValue::Bool(b) => {
                bytecode.push(3); // Type tag for bool
                bytecode.push(if *b { 1 } else { 0 });
            }
            ConstantValue::None => {
                bytecode.push(4); // Type tag for none
            }
        }
        Ok(())
    }
}

pub fn generate(program: &Program) -> Result<Vec<u8>, NagariError> {
    let mut generator = CodeGenerator::new();
    generator.generate(program)
}
