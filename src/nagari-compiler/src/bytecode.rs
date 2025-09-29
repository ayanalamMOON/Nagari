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

    // Import and module operations
    ImportName = 0x1F,
    ImportFrom = 0x20,
    ImportStar = 0x21,

    // Function operations
    MakeFunction = 0x22,
    LoadClosure = 0x23,
    LoadDeref = 0x24,
    StoreDeref = 0x25,

    // Advanced control flow
    SetupExcept = 0x26,
    PopExcept = 0x27,
    RaiseVarargs = 0x28,

    // Pattern matching
    MatchSequence = 0x29,
    MatchMapping = 0x2A,
    MatchClass = 0x2B,
    MatchKeys = 0x2C,

    // Additional operations
    UnpackSequence = 0x2D,
    UnpackEx = 0x2E,
    BuildSlice = 0x2F,
    LoadAttr = 0x30,
    StoreAttr = 0x31,
    DeleteAttr = 0x32,
    LoadGlobal = 0x33,
    StoreGlobal = 0x34,

    // Logical operations
    BinaryAnd = 0x35,
    BinaryOr = 0x36,
    UnaryNot = 0x37,
    UnaryInvert = 0x38,
    UnaryPositive = 0x39,
    UnaryNegative = 0x3A,

    // Missing opcodes
    SetupAsync = 0x3B,
    GetIter = 0x3C,
    DupTop = 0x3D,
    RaiseError = 0x3E,
    CompareOp = 0x3F,
    CompareLength = 0x40,
    Nop = 0x41,
    BuildTuple = 0x42,
    BuildSet = 0x43,
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

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module_name: String,
    pub names: Vec<String>,
    pub aliases: Vec<Option<String>>,
    pub is_star_import: bool,
}

#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub start_addr: usize,
    pub break_addrs: Vec<usize>,
    pub continue_addrs: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ExceptionInfo {
    pub handler_addr: usize,
    pub stack_size: usize,
}

#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub scope_type: ScopeType,
    pub locals: std::collections::HashSet<String>,
    pub parent_locals: std::collections::HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(ConstantValue),
    Identifier(String),
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Dict(Vec<(Pattern, Pattern)>),
    Wildcard,
    Guard {
        pattern: Box<Pattern>,
        condition: Expression,
    },
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct MatchStatement {
    pub expression: Expression,
    pub cases: Vec<MatchCase>,
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Module,
    Function,
    Class,
    Comprehension,
}

pub struct CodeGenerator {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
    names: Vec<String>,
    constant_map: std::collections::HashMap<String, usize>,
    name_map: std::collections::HashMap<String, usize>,

    // Advanced features
    varnames: Vec<String>, // Local variable names
    freevars: Vec<String>, // Free variables (for closures)
    cellvars: Vec<String>, // Cell variables (for closures)
    imports: Vec<ImportInfo>,

    // Control flow tracking
    loop_stack: Vec<LoopInfo>,
    exception_stack: Vec<ExceptionInfo>,

    // Scoping
    scope_stack: Vec<ScopeInfo>,
    current_scope: ScopeType,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            constant_map: std::collections::HashMap::new(),
            name_map: std::collections::HashMap::new(),

            // Advanced features
            varnames: Vec::new(),
            freevars: Vec::new(),
            cellvars: Vec::new(),
            imports: Vec::new(),

            // Control flow tracking
            loop_stack: Vec::new(),
            exception_stack: Vec::new(),

            // Scoping
            scope_stack: Vec::new(),
            current_scope: ScopeType::Module,
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
            Statement::FunctionDef(func_def) => {
                self.compile_function_def(func_def)?;
                Ok(())
            }
            Statement::Assignment(assign) => self.compile_assignment(assign),
            Statement::If(if_stmt) => self.compile_if(if_stmt),
            Statement::While(while_loop) => self.compile_while(while_loop),
            Statement::For(for_loop) => {
                self.compile_for_loop(for_loop)?;
                Ok(())
            }
            Statement::Match(match_stmt) => self.compile_match(match_stmt),
            Statement::Return(expr) => self.compile_return(expr),
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::Pop, None); // Pop unused expression result
                Ok(())
            }
            Statement::Import(import_stmt) => {
                self.compile_import_statement(import_stmt)?;
                Ok(())
            }
            Statement::Break => {
                if self.loop_stack.is_empty() {
                    return Err(NagariError::SemanticError("break outside loop".to_string()));
                }
                let break_jump = self.emit_jump(Opcode::Jump);
                if let Some(loop_info) = self.loop_stack.last_mut() {
                    loop_info.break_addrs.push(break_jump);
                }
                Ok(())
            }
            Statement::Continue => {
                if self.loop_stack.is_empty() {
                    return Err(NagariError::SemanticError(
                        "continue outside loop".to_string(),
                    ));
                }
                let continue_jump = self.emit_jump(Opcode::Jump);
                if let Some(loop_info) = self.loop_stack.last_mut() {
                    loop_info.continue_addrs.push(continue_jump);
                }
                Ok(())
            }
            // Placeholder implementations for the remaining variants
            Statement::AttributeAssignment(_) => Ok(()),
            Statement::TupleAssignment(_) => Ok(()),
            Statement::Del(_) => Ok(()),
            Statement::With(_) => Ok(()),
            Statement::Try(_) => Ok(()),
            Statement::Raise(_) => Ok(()),
            Statement::TypeAlias(_) => Ok(()),
            Statement::Yield(_) => Ok(()),
            Statement::YieldFrom(_) => Ok(()),
            Statement::ClassDef(_) => Ok(()),
            Statement::DestructuringAssignment(_) => Ok(()),
            Statement::ArrayDestructuringAssignment(_) => Ok(()),
            Statement::ImportDefault(_) => Ok(()),
            Statement::ImportNamed(_) => Ok(()),
            Statement::ImportNamespace(_) => Ok(()),
            Statement::ImportSideEffect(_) => Ok(()),
            Statement::ExportDefault(_) => Ok(()),
            Statement::ExportNamed(_) => Ok(()),
            Statement::ExportAll(_) => Ok(()),
            Statement::ExportDeclaration(_) => Ok(()),
            Statement::Pass => {
                self.emit_opcode(Opcode::Nop);
                Ok(())
            }
        }
    }

    fn compile_function_def(&mut self, func_def: &FunctionDef) -> Result<(), NagariError> {
        // Create a new scope for the function
        let mut function_locals = std::collections::HashSet::new();

        // Add parameters to the function's local variables
        for param in &func_def.parameters {
            function_locals.insert(param.name.clone());
        }

        // Save current state
        let saved_varnames = self.varnames.clone();
        let saved_freevars = self.freevars.clone();
        let saved_cellvars = self.cellvars.clone();

        // Enter function scope
        let scope_info = ScopeInfo {
            scope_type: ScopeType::Function,
            locals: function_locals.clone(),
            parent_locals: self.get_current_locals(),
        };
        self.scope_stack.push(scope_info);

        // Reset function-specific state
        self.varnames.clear();
        self.freevars.clear();
        self.cellvars.clear();

        // Add parameters to varnames (local variables)
        for param in &func_def.parameters {
            self.add_varname(param.name.clone());
        }

        // Create function code object
        let _func_start = self.instructions.len();

        // Compile default parameter values if any
        let mut defaults_count = 0;
        for param in func_def.parameters.iter().rev() {
            if let Some(default) = &param.default_value {
                self.compile_expression(default)?;
                defaults_count += 1;
            } else {
                break;
            }
        }

        // Mark function as async if needed
        // Note: FunctionDef doesn't have is_async field in current AST
        // if func_def.is_async {
        //     self.emit_opcode(Opcode::SetupAsync);
        // }

        // Compile function body
        for statement in &func_def.body {
            self.compile_statement(statement)?;
        }

        // Ensure function returns something (None if no explicit return)
        self.emit_opcode(Opcode::LoadConst);
        let none_const = self.add_constant(ConstantValue::None);
        self.emit_arg(none_const);
        self.emit_opcode(Opcode::Return);

        // Create function object
        let code_idx = self.create_code_object(
            &func_def.name,
            func_def.parameters.len(),
            defaults_count,
            self.varnames.clone(),
            self.freevars.clone(),
            self.cellvars.clone(),
        );

        // Restore previous state
        self.scope_stack.pop();
        self.varnames = saved_varnames;
        self.freevars = saved_freevars;
        self.cellvars = saved_cellvars;

        // Load the function code and create function object
        self.emit_opcode_with_arg(Opcode::LoadConst, code_idx as u32);
        let name_const = self.add_constant(ConstantValue::String(func_def.name.clone()));
        self.emit_opcode_with_arg(Opcode::LoadConst, name_const);

        // Handle closures - load free variables
        let freevars_clone = self.freevars.clone();
        for freevar in &freevars_clone {
            let deref_idx = self.add_name(freevar);
            self.emit_opcode_with_arg(Opcode::LoadDeref, deref_idx);
        }

        // Create function with appropriate flags
        let mut flags = 0;
        // Note: FunctionDef doesn't have is_async field in current AST
        // if func_def.is_async {
        //     flags |= 0x80; // CO_COROUTINE
        // }
        if defaults_count > 0 {
            flags |= 0x01; // Has defaults
        }
        if !self.freevars.is_empty() {
            flags |= 0x10; // Has free variables
        }

        self.emit_opcode_with_arg(Opcode::MakeFunction, flags);

        // Store the function in the appropriate scope
        let func_name_idx = self.add_varname(func_def.name.clone());
        self.emit_opcode_with_arg(Opcode::StoreName, func_name_idx);

        Ok(())
    }

    fn create_code_object(
        &mut self,
        name: &str,
        arg_count: usize,
        defaults_count: usize,
        varnames: Vec<String>,
        freevars: Vec<String>,
        cellvars: Vec<String>,
    ) -> usize {
        // This would create a code object in a real implementation
        // For now, we'll store the necessary information as a constant
        let code_info = format!(
            "CODE:{}:{}:{}:{}:{}:{}",
            name,
            arg_count,
            defaults_count,
            varnames.join(","),
            freevars.join(","),
            cellvars.join(",")
        );
        self.add_constant(ConstantValue::String(code_info)) as usize
    }

    fn patch_jump_to(&mut self, jump_addr: usize, target_addr: usize) {
        // Calculate the relative offset
        let _offset = target_addr as i32 - jump_addr as i32;

        // Update the instruction at jump_addr with the target
        if jump_addr < self.instructions.len() {
            // Store the target address in the instruction's argument
            self.instructions[jump_addr].operand = Some(target_addr as u32);
        }
    }

    fn get_current_locals(&self) -> std::collections::HashSet<String> {
        if let Some(scope) = self.scope_stack.last() {
            scope.locals.clone()
        } else {
            std::collections::HashSet::new()
        }
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

    fn compile_for(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &[Statement],
    ) -> Result<(), NagariError> {
        // Compile the iterable expression
        self.compile_expression(iterable)?;

        // Get the iterator
        self.emit_opcode(Opcode::GetIter);

        // Setup loop: this is the start of the loop body
        let loop_start = self.instructions.len();

        // Create loop info for break/continue tracking
        let loop_info = LoopInfo {
            start_addr: loop_start,
            break_addrs: Vec::new(),
            continue_addrs: Vec::new(),
        };
        self.loop_stack.push(loop_info);

        // Try to get the next item from iterator
        self.emit_opcode(Opcode::ForIter);
        let for_iter_jump = self.instructions.len() - 1; // Remember this position to patch later

        // Store the current item in the loop variable
        let var_idx = self.add_varname(variable.to_string());
        self.emit_opcode_with_arg(Opcode::StoreName, var_idx);

        // Compile the loop body
        for statement in body {
            self.compile_statement(statement)?;
        }

        // Handle continue statements (jump back to start of loop)
        let continue_addrs = {
            let loop_info = self.loop_stack.last_mut().unwrap();
            loop_info.continue_addrs.clone()
        };
        for &continue_addr in &continue_addrs {
            self.patch_jump_to(continue_addr, loop_start);
        }

        // Jump back to the beginning of the loop
        self.emit_opcode_with_arg(Opcode::Jump, loop_start as u32);

        // This is where ForIter jumps when the iterator is exhausted
        let loop_end = self.instructions.len();

        // Patch the ForIter instruction to jump here when done
        self.patch_jump_to(for_iter_jump, loop_end);

        // Handle break statements (jump to end of loop)
        let loop_info = self.loop_stack.pop().unwrap();
        for &break_addr in &loop_info.break_addrs {
            self.patch_jump_to(break_addr, loop_end);
        }

        // Pop the iterator from the stack (ForIter leaves it there when exhausted)
        self.emit_opcode(Opcode::Pop);

        Ok(())
    }

    fn compile_match(
        &mut self,
        match_stmt: &crate::ast::MatchStatement,
    ) -> Result<(), NagariError> {
        // Compile the match expression - this is what we're matching against
        self.compile_expression(&match_stmt.expression)?;

        // We'll need to keep a copy of the match value on the stack for each pattern
        let mut case_jumps = Vec::new();
        let mut end_jumps = Vec::new();

        for (case_idx, case) in match_stmt.cases.iter().enumerate() {
            // Duplicate the match value for this case (except for the last case)
            if case_idx < match_stmt.cases.len() - 1 {
                self.emit_opcode(Opcode::DupTop);
            }

            // Compile the pattern matching
            let pattern_match_jump = self.compile_pattern(&case.pattern)?;

            // Note: AST MatchCase doesn't have guard field in current implementation
            // Guards are handled as Pattern::Guard within the pattern itself

            // Pattern matched (and guard passed if present), execute the case body
            for statement in &case.body {
                self.compile_statement(statement)?;
            }

            // Jump to end after executing this case
            let end_jump = self.emit_jump(Opcode::Jump);
            end_jumps.push(end_jump);

            // Patch the pattern match jump to skip this case
            self.patch_jump(pattern_match_jump);

            // Patch guard jumps if any
            for jump in case_jumps.drain(..) {
                self.patch_jump(jump);
            }
        }

        // Clean up the match value from the stack if no pattern matched
        self.emit_opcode(Opcode::Pop);

        // Raise a MatchError if no pattern matched
        let error_const =
            self.add_constant(ConstantValue::String("No pattern matched".to_string()));
        self.emit_opcode_with_arg(Opcode::LoadConst, error_const);
        self.emit_opcode(Opcode::RaiseError);

        // Patch all end jumps to here
        let end_addr = self.instructions.len();
        for jump in end_jumps {
            self.patch_jump_to(jump, end_addr);
        }

        Ok(())
    }

    fn compile_pattern(&mut self, pattern: &crate::ast::Pattern) -> Result<usize, NagariError> {
        match pattern {
            crate::ast::Pattern::Literal(value) => {
                // Compare the match value with the literal
                let const_idx = match value {
                    Literal::Int(n) => self.add_constant(ConstantValue::Int(*n)),
                    Literal::Float(f) => self.add_constant(ConstantValue::Float(*f)),
                    Literal::String(s) => self.add_constant(ConstantValue::String(s.clone())),
                    Literal::Bool(b) => self.add_constant(ConstantValue::Bool(*b)),
                    Literal::None => self.add_constant(ConstantValue::None),
                };
                self.emit_opcode_with_arg(Opcode::LoadConst, const_idx);
                self.emit_opcode(Opcode::CompareOp); // Equal comparison
                let jump_if_false = self.emit_jump(Opcode::JumpIfFalse);
                Ok(jump_if_false)
            }

            crate::ast::Pattern::Identifier(name) => {
                // Bind the match value to the identifier (always succeeds)
                let var_idx = self.add_varname(name.clone());
                self.emit_opcode_with_arg(Opcode::StoreName, var_idx);
                // Create a dummy jump that never triggers (always matches)
                let const_idx = self.add_constant(ConstantValue::Bool(true));
                self.emit_opcode_with_arg(Opcode::LoadConst, const_idx);
                let dummy_jump = self.emit_jump(Opcode::JumpIfFalse);
                Ok(dummy_jump)
            }

            crate::ast::Pattern::Wildcard => {
                // Wildcard always matches, just consume the value
                self.emit_opcode(Opcode::Pop);
                // Create a dummy jump that never triggers
                let const_idx = self.add_constant(ConstantValue::Bool(true));
                self.emit_opcode_with_arg(Opcode::LoadConst, const_idx);
                let dummy_jump = self.emit_jump(Opcode::JumpIfFalse);
                Ok(dummy_jump)
            }

            crate::ast::Pattern::Tuple(patterns) => self.compile_sequence_pattern(patterns, true),

            crate::ast::Pattern::List(patterns) => self.compile_sequence_pattern(patterns, false),

            crate::ast::Pattern::Dict(pairs) => {
                // Check if the match value is a dict and has the required keys
                self.emit_opcode(Opcode::MatchMapping);
                let fail_jump = self.emit_jump(Opcode::JumpIfFalse);

                // Match each key-value pair
                for (key_pattern, value_pattern) in pairs {
                    // Get the value for this key
                    self.compile_pattern(key_pattern)?;
                    self.emit_opcode(Opcode::GetItem);

                    // Match the value pattern
                    self.compile_pattern(value_pattern)?;
                }

                Ok(fail_jump)
            }

            crate::ast::Pattern::Guard(pattern, condition) => {
                // First match the inner pattern
                let pattern_jump = self.compile_pattern(pattern)?;

                // Then check the guard condition
                self.compile_expression(condition)?;
                let _guard_jump = self.emit_jump(Opcode::JumpIfFalse);

                // Return the pattern jump (guard jump will be handled separately)
                Ok(pattern_jump)
            }

            crate::ast::Pattern::Constructor(_name, _patterns) => {
                // TODO: Implement constructor pattern matching
                Ok(self.emit_jump(Opcode::JumpIfFalse))
            }

            crate::ast::Pattern::Range(start, end) => {
                // TODO: Implement range pattern matching
                self.compile_expression(start)?;
                self.compile_expression(end)?;
                Ok(self.emit_jump(Opcode::JumpIfFalse))
            }
        }
    }

    fn compile_sequence_pattern(
        &mut self,
        patterns: &[crate::ast::Pattern],
        is_tuple: bool,
    ) -> Result<usize, NagariError> {
        // Check if the match value is the right type and length
        if is_tuple {
            self.emit_opcode(Opcode::MatchSequence);
        } else {
            self.emit_opcode(Opcode::MatchSequence);
        }

        // Check length
        let len_const = self.add_constant(ConstantValue::Int(patterns.len() as i64));
        self.emit_opcode_with_arg(Opcode::LoadConst, len_const);
        self.emit_opcode(Opcode::CompareLength);
        let fail_jump = self.emit_jump(Opcode::JumpIfFalse);

        // Unpack and match each element
        for (i, pattern) in patterns.iter().enumerate() {
            // Get the i-th element
            let idx_const = self.add_constant(ConstantValue::Int(i as i64));
            self.emit_opcode_with_arg(Opcode::LoadConst, idx_const);
            self.emit_opcode(Opcode::GetItem);

            // Match the pattern
            self.compile_pattern(pattern)?;
        }

        Ok(fail_jump)
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

    fn compile_import_statement(
        &mut self,
        import_stmt: &ImportStatement,
    ) -> Result<(), NagariError> {
        if let Some(ref items) = import_stmt.items {
            // from module import items
            let module_idx = self.add_constant(ConstantValue::String(import_stmt.module.clone()));
            self.emit_opcode_with_arg(Opcode::ImportName, module_idx);

            for item in items {
                let name_idx = self.add_constant(ConstantValue::String(item.clone()));
                self.emit_opcode_with_arg(Opcode::ImportFrom, name_idx);
                let var_idx = self.add_varname(item.clone());
                self.emit_opcode_with_arg(Opcode::StoreName, var_idx);
            }
        } else {
            // import module
            let module_idx = self.add_constant(ConstantValue::String(import_stmt.module.clone()));
            self.emit_opcode_with_arg(Opcode::ImportName, module_idx);
            let var_idx = self.add_varname(import_stmt.module.clone());
            self.emit_opcode_with_arg(Opcode::StoreName, var_idx);
        }
        Ok(())
    }

    fn compile_for_loop(&mut self, for_loop: &ForLoop) -> Result<(), NagariError> {
        // Compile the iterable expression
        self.compile_expression(&for_loop.iterable)?;

        // Get an iterator
        self.emit_opcode(Opcode::GetIter);

        let loop_start = self.instructions.len();
        let break_jump = self.emit_jump(Opcode::ForIter);

        // Store loop variable
        let var_idx = self.add_varname(for_loop.variable.clone());
        self.emit_opcode_with_arg(Opcode::StoreName, var_idx);

        // Compile loop body
        for stmt in &for_loop.body {
            self.compile_statement(stmt)?;
        }

        // Jump back to loop start
        self.emit_opcode_with_arg(Opcode::Jump, loop_start as u32);

        // Patch the break jump
        self.patch_jump(break_jump);

        Ok(())
    }

    fn compile_import(&mut self, source: &str, items: &[String]) -> Result<(), NagariError> {
        // Add import info to tracking
        let mut names = Vec::new();
        let mut aliases = Vec::new();
        let is_star_import = items.is_empty(); // Empty items means import * or module import

        for item in items {
            names.push(item.clone());
            aliases.push(None);
        }

        let import_info = ImportInfo {
            module_name: source.to_string(),
            names: names.clone(),
            aliases: aliases.clone(),
            is_star_import,
        };
        self.imports.push(import_info);

        // Generate bytecode based on import type
        if items.is_empty() {
            // Simple module import: import module
            let module_idx = self.add_constant(ConstantValue::String(source.to_string()));
            self.emit_opcode_with_arg(Opcode::ImportName, module_idx);

            // Store the module with its name (extract from path)
            let module_name = source.split('/').last().unwrap_or(source).to_string();
            let var_idx = self.add_varname(module_name);
            self.emit_opcode_with_arg(Opcode::StoreName, var_idx);
        } else {
            // From import: from module import name1, name2, ...
            let module_idx = self.add_constant(ConstantValue::String(source.to_string()));
            self.emit_opcode_with_arg(Opcode::ImportName, module_idx);

            // Import each specified item
            for item in items {
                let name_idx = self.add_constant(ConstantValue::String(item.clone()));
                self.emit_opcode_with_arg(Opcode::ImportFrom, name_idx);

                // Store with original name (no alias support for String items)
                let var_idx = self.add_varname(item.clone());
                self.emit_opcode_with_arg(Opcode::StoreName, var_idx);
            }
        }

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
            // Placeholder implementations for missing Expression variants
            Expression::JSXElement(_) => {
                // TODO: Implement JSX element compilation
                Ok(())
            }
            Expression::Lambda(_) => {
                // TODO: Implement lambda compilation
                Ok(())
            }
            Expression::ListComprehension(_) => {
                // TODO: Implement list comprehension compilation
                Ok(())
            }
            Expression::DictComprehension(_) => {
                // TODO: Implement dict comprehension compilation
                Ok(())
            }
            Expression::SetComprehension(_) => {
                // TODO: Implement set comprehension compilation
                Ok(())
            }
            Expression::Generator(_) => {
                // TODO: Implement generator compilation
                Ok(())
            }
            Expression::Ternary(_) => {
                // TODO: Implement ternary compilation
                Ok(())
            }
            Expression::Attribute(_) => {
                // TODO: Implement attribute access compilation
                Ok(())
            }
            Expression::Index(_) => {
                // TODO: Implement index access compilation
                Ok(())
            }
            Expression::Slice(_) => {
                // TODO: Implement slice compilation
                Ok(())
            }
            Expression::Tuple(elements) => {
                for element in elements {
                    self.compile_expression(element)?;
                }
                self.emit(Opcode::BuildTuple, Some(elements.len() as u32));
                Ok(())
            }
            Expression::Set(elements) => {
                for element in elements {
                    self.compile_expression(element)?;
                }
                self.emit(Opcode::BuildSet, Some(elements.len() as u32));
                Ok(())
            }
            Expression::Unary(_) => {
                // TODO: Implement unary expression compilation
                Ok(())
            }
            Expression::NamedExpr(_) => {
                // TODO: Implement named expression (walrus operator) compilation
                Ok(())
            }
            Expression::Dictionary(pairs) => {
                for (key, value) in pairs {
                    self.compile_expression(key)?;
                    self.compile_expression(value)?;
                }
                self.emit(Opcode::BuildDict, Some(pairs.len() as u32));
                Ok(())
            }
            Expression::Subscript(_) => {
                // TODO: Implement subscript compilation
                Ok(())
            }
            Expression::FunctionExpr(_) => {
                // TODO: Implement function expression compilation
                Ok(())
            }
            Expression::Async(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::SetupAsync, None);
                Ok(())
            }
            Expression::Spread(_) => {
                // TODO: Implement spread operator compilation
                Ok(())
            }
            Expression::TemplateLiteral(_) => {
                // TODO: Implement template literal compilation
                Ok(())
            }
            Expression::FString(_) => {
                // TODO: Implement f-string compilation
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
            BinaryOperator::And => Opcode::BinaryAnd,
            BinaryOperator::Or => Opcode::BinaryOr,
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

    fn emit_opcode(&mut self, opcode: Opcode) -> usize {
        self.emit(opcode, None)
    }

    fn emit_opcode_with_arg(&mut self, opcode: Opcode, arg: u32) -> usize {
        self.emit(opcode, Some(arg))
    }

    fn emit_arg(&mut self, arg: u32) -> usize {
        let instruction = Instruction {
            opcode: Opcode::Nop, // Use Nop as placeholder for arg-only instructions
            operand: Some(arg),
        };
        self.instructions.push(instruction);
        self.instructions.len() - 1
    }

    fn add_varname(&mut self, name: String) -> u32 {
        // Check if the variable name already exists
        for (index, var) in self.varnames.iter().enumerate() {
            if var == &name {
                return index as u32;
            }
        }

        // Add new variable name
        self.varnames.push(name);
        (self.varnames.len() - 1) as u32
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

    fn add_constant(&mut self, value: ConstantValue) -> u32 {
        let key = format!("{:?}", value);

        if let Some(&index) = self.constant_map.get(&key) {
            return index as u32;
        }

        let index = self.constants.len();
        self.constants.push(Constant { value });
        self.constant_map.insert(key, index);
        index as u32
    }

    fn add_name(&mut self, name: &str) -> u32 {
        if let Some(&index) = self.name_map.get(name) {
            return index as u32;
        }

        let index = self.names.len();
        self.names.push(name.to_string());
        self.name_map.insert(name.to_string(), index);
        index as u32
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
            self.serialize_constant(&constant.value, &mut bytecode)?;
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

    fn serialize_constant(
        &self,
        constant: &ConstantValue,
        bytecode: &mut Vec<u8>,
    ) -> Result<(), NagariError> {
        match constant {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_generator() -> CodeGenerator {
        CodeGenerator::new()
    }

    fn create_simple_program(statements: Vec<Statement>) -> Program {
        Program { statements }
    }

    #[test]
    fn test_import_simple() {
        let mut generator = create_test_generator();
        let import_stmt = Statement::Import {
            source: "math".to_string(),
            items: vec![],
        };

        assert!(generator.compile_statement(&import_stmt).is_ok());

        // Verify that import info was recorded
        assert_eq!(generator.imports.len(), 1);
        assert_eq!(generator.imports[0].module_name, "math");
        assert!(!generator.imports[0].is_star_import);

        // Verify bytecode was generated
        assert!(!generator.instructions.is_empty());
    }

    #[test]
    fn test_import_from() {
        let mut generator = create_test_generator();
        let import_stmt = Statement::Import {
            source: "math".to_string(),
            items: vec![
                ImportItem {
                    name: "sqrt".to_string(),
                    alias: None,
                },
                ImportItem {
                    name: "pi".to_string(),
                    alias: Some("PI".to_string()),
                },
            ],
        };

        assert!(generator.compile_statement(&import_stmt).is_ok());

        // Verify import info
        assert_eq!(generator.imports.len(), 1);
        assert_eq!(generator.imports[0].names, vec!["sqrt", "pi"]);
        assert_eq!(
            generator.imports[0].aliases,
            vec![None, Some("PI".to_string())]
        );
    }

    #[test]
    fn test_function_compilation() {
        let mut generator = create_test_generator();
        let func_stmt = Statement::Function {
            name: "add".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "a".to_string(),
                    type_annotation: None,
                    default_value: None,
                },
                FunctionParameter {
                    name: "b".to_string(),
                    type_annotation: None,
                    default_value: Some(Expression::Literal(Literal::Number(0.0))),
                },
            ],
            body: vec![Statement::Return(Some(Expression::Binary {
                left: Box::new(Expression::Identifier("a".to_string())),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Identifier("b".to_string())),
            }))],
            is_async: false,
            return_type: None,
        };

        assert!(generator.compile_statement(&func_stmt).is_ok());

        // Verify function was created
        assert!(generator.varnames.contains(&"add".to_string()));
        assert!(!generator.instructions.is_empty());
    }

    #[test]
    fn test_for_loop_compilation() {
        let mut generator = create_test_generator();
        let for_stmt = Statement::For {
            variable: "i".to_string(),
            iterable: Expression::List(vec![
                Expression::Literal(Literal::Number(1.0)),
                Expression::Literal(Literal::Number(2.0)),
                Expression::Literal(Literal::Number(3.0)),
            ]),
            body: vec![Statement::Expression(Expression::Call {
                function: Box::new(Expression::Identifier("print".to_string())),
                arguments: vec![Expression::Identifier("i".to_string())],
            })],
        };

        assert!(generator.compile_statement(&for_stmt).is_ok());

        // Verify loop opcodes were generated
        let has_get_iter = generator
            .instructions
            .iter()
            .any(|inst| matches!(inst.opcode, Opcode::GetIter));
        let has_for_iter = generator
            .instructions
            .iter()
            .any(|inst| matches!(inst.opcode, Opcode::ForIter));

        assert!(has_get_iter);
        assert!(has_for_iter);
    }

    #[test]
    fn test_break_outside_loop_error() {
        let mut generator = create_test_generator();
        let break_stmt = Statement::Break;

        assert!(generator.compile_statement(&break_stmt).is_err());
    }

    #[test]
    fn test_continue_outside_loop_error() {
        let mut generator = create_test_generator();
        let continue_stmt = Statement::Continue;

        assert!(generator.compile_statement(&continue_stmt).is_err());
    }

    #[test]
    fn test_pattern_matching_literal() {
        let mut generator = create_test_generator();
        let match_stmt = MatchStatement {
            expression: Expression::Literal(Literal::Number(42.0)),
            cases: vec![
                MatchCase {
                    pattern: Pattern::Literal(ConstantValue::Int(42)),
                    guard: None,
                    body: vec![Statement::Return(Some(Expression::Literal(
                        Literal::String("matched".to_string()),
                    )))],
                },
                MatchCase {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: vec![Statement::Return(Some(Expression::Literal(
                        Literal::String("default".to_string()),
                    )))],
                },
            ],
        };

        assert!(generator.compile_match(&match_stmt).is_ok());
    }

    #[test]
    fn test_constant_serialization() {
        let mut generator = create_test_generator();

        // Add various constants
        let int_idx = generator.add_constant(ConstantValue::Int(42));
        let float_idx = generator.add_constant(ConstantValue::Float(3.14));
        let string_idx = generator.add_constant(ConstantValue::String("hello".to_string()));
        let bool_idx = generator.add_constant(ConstantValue::Bool(true));
        let none_idx = generator.add_constant(ConstantValue::None);

        // Verify constants were added
        assert_eq!(generator.constants.len(), 5);
        assert_ne!(int_idx, float_idx);
        assert_ne!(float_idx, string_idx);

        // Test serialization
        let mut bytecode = Vec::new();
        for constant in &generator.constants {
            assert!(generator
                .serialize_constant(constant, &mut bytecode)
                .is_ok());
        }

        // Verify bytecode was generated
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_variable_name_management() {
        let mut generator = create_test_generator();

        let var1_idx = generator.add_varname("variable1".to_string());
        let var2_idx = generator.add_varname("variable2".to_string());
        let var1_idx_duplicate = generator.add_varname("variable1".to_string());

        // First two should be different
        assert_ne!(var1_idx, var2_idx);

        // Adding same variable again should return same index
        assert_eq!(var1_idx, var1_idx_duplicate);

        // Verify variables are stored
        assert!(generator.varnames.contains(&"variable1".to_string()));
        assert!(generator.varnames.contains(&"variable2".to_string()));
    }
}
