use std::cell::RefCell;

use rustc_hash::FxHashMap;
use rustcc_ast::{
    binary_operator::BinaryOperator,
    declaration::Declaration,
    expression::{Expression, ExpressionKind},
    function_definition::{BlockItem, FunctionDefinition},
    statement::{Statement, StatementKind},
    translation_unit::TranslationUnit,
    unary_operator::UnaryOperator,
};
use rustcc_llvm::{
    analysis::{LLVMVerifierFailureAction, verify_function},
    basic_block::LLVMBasicBlock,
    builder::LLVMBuilder,
    context::LLVMContext,
    function::{LLVMFunctionType, LLVMFunctionValue, function_type},
    module::LLVMModule,
    typ::LLVMType,
    value::LLVMValue,
};

#[derive(Debug)]
pub struct Codegen {
    // Note: The order of these fields matters for proper drop order
    builder: LLVMBuilder,
    module: LLVMModule,
    context: LLVMContext,

    // Variable name -> ptr to the variable in LLVM IR
    symbol_table: RefCell<FxHashMap<String, LLVMValue>>,
}

#[derive(Debug)]
pub enum CodegenError {
    FailedContextCreation,
    FailedModuleCreation,
    FailedBuilderCreation,
}

impl std::error::Error for CodegenError {}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedContextCreation => write!(f, "Failed to create LLVM context"),
            Self::FailedModuleCreation => write!(f, "Failed to create LLVM module"),
            Self::FailedBuilderCreation => write!(f, "Failed to create LLVM builder"),
        }
    }
}

impl Codegen {
    pub fn new(file_path: &str) -> Result<Self, CodegenError> {
        let context = LLVMContext::new().ok_or(CodegenError::FailedContextCreation)?;
        let module = LLVMModule::new_in_context(file_path, &context)
            .ok_or(CodegenError::FailedModuleCreation)?;
        let builder =
            LLVMBuilder::new_in_context(&context).ok_or(CodegenError::FailedBuilderCreation)?;

        Ok(Self {
            builder,
            module,
            context,
            symbol_table: RefCell::new(FxHashMap::default()),
        })
    }

    /// Recreate the LLVM module while reusing the same LLVM context and
    /// builder.
    ///
    /// This is useful for hot-looping compiles (e.g., fuzzing) where creating
    /// a fresh context/builder per input is expensive. The old module is
    /// dropped when this method returns and a new one is installed.
    pub fn reset_module(&mut self, file_path: &str) -> Result<(), CodegenError> {
        // Create a fresh module within the existing context.
        let new_module = LLVMModule::new_in_context(file_path, &self.context)
            .ok_or(CodegenError::FailedModuleCreation)?;
        // Replace the module; old module will be disposed via Drop.
        self.module = new_module;
        self.symbol_table.borrow_mut().clear();
        Ok(())
    }

    pub fn dump(&self) {
        self.module.dump();
    }

    #[must_use]
    fn int32_type(&self) -> LLVMType {
        self.context.int32_type()
    }

    #[must_use]
    fn const_int(&self, value: u32) -> LLVMValue {
        self.int32_type().constant_integer(u64::from(value), false)
    }

    fn function(&self, function_name: &str, function_type: LLVMFunctionType) -> LLVMFunctionValue {
        self.module.add_function(function_name, function_type)
    }

    fn function_basic_block(&self, name: &str, function: LLVMFunctionValue) -> LLVMBasicBlock {
        let basic_block = self
            .context
            .create_basic_block_for_function(&function, name);

        // Move the builder to the end of the basic block
        self.builder.position_at_end(basic_block);

        basic_block
    }

    #[must_use]
    fn new_stack_variable(&self, name: &str) -> LLVMValue {
        let ptr = self.builder.alloca(self.int32_type(), name);
        self.symbol_table.borrow_mut().insert(name.to_string(), ptr);

        ptr
    }

    #[must_use]
    #[expect(clippy::expect_used)]
    fn get_current_function(&self) -> LLVMFunctionValue {
        self.builder
            .get_insert_block()
            .expect("No insert block")
            .get_parent()
            .expect("No parent function")
    }

    #[must_use]
    fn negate(&self, value: LLVMValue) -> LLVMValue {
        self.builder.negate(value)
    }

    #[must_use]
    fn complement(&self, value: LLVMValue) -> LLVMValue {
        self.builder.bitwise_complement(value)
    }

    #[must_use]
    fn logical_not(&self, value: LLVMValue) -> LLVMValue {
        // 1. Check if the value == 0
        let zero = self.const_int(0);
        let value = self.builder.integer_equal(value, zero);

        // 2. Zero-extend the i1 result to i32
        self.builder.zero_extend(value, self.int32_type())
    }

    #[expect(clippy::panic)]
    fn load_variable(&self, name: &str) -> LLVMValue {
        let symbol_table = self.symbol_table.borrow();
        let ptr = symbol_table
            .get(name)
            .unwrap_or_else(|| panic!("Variable {name} not found in symbol table"));

        self.builder.load(self.int32_type(), *ptr, name)
    }

    #[expect(clippy::panic)]
    fn store_variable(&self, name: &str, value: LLVMValue) {
        let symbol_table = self.symbol_table.borrow();
        let ptr = symbol_table
            .get(name)
            .unwrap_or_else(|| panic!("Variable {name} not found in symbol table"));

        self.builder.store(value, *ptr);
    }

    #[must_use]
    pub fn codegen(&self, translation_unit: &TranslationUnit) -> bool {
        // Codegen all functions
        for function in &translation_unit.function {
            if !self.codegen_function(function) {
                return false;
            }
        }

        true
    }

    #[must_use]
    fn codegen_function(&self, function: &FunctionDefinition) -> bool {
        // Clear the symbol table for the new function scope
        self.symbol_table.borrow_mut().clear();

        // Create the function type
        let function_type = function_type(self.int32_type());

        // Create the function
        let llvm_function = self.function(&function.name, function_type);

        // Create a basic block in the function and set our builder to generate
        // code in it.
        self.function_basic_block("entry", llvm_function);

        // Codegen the function body
        for statement in &function.body {
            if self.builder.has_insert_block_terminator() {
                // if the current block has a terminator, don't generate anything since it's
                // unreachable
                // TODO: Warn about unreachable code
                break;
            }

            match statement {
                BlockItem::Statement(statement) => self.codegen_statement(statement),
                BlockItem::Declaration(declaration) => {
                    self.codegen_declaration(declaration);
                }
            }
        }

        // Check if the last block has a terminator
        #[expect(clippy::expect_used)]
        let basic_block = self.builder.get_insert_block().expect("No insert block");

        if basic_block.terminator().is_none() {
            // TODO: What about void function
            if function.name == "main" {
                // If main is missing a terminator add return 0
                let zero = self.const_int(0);
                self.builder.ret(zero);
            } else {
                // For all other functions add unreachable
                self.builder.unreachable();
            }
        }

        // Verify generated function and when fuzzing abort on failure
        #[cfg(not(fuzzing))]
        if !verify_function(
            llvm_function,
            LLVMVerifierFailureAction::LLVMPrintMessageAction,
        ) {
            println!("Function verification failed");
            return false;
        }
        #[cfg(fuzzing)]
        let _ = verify_function(
            llvm_function,
            LLVMVerifierFailureAction::LLVMAbortProcessAction,
        );

        true
    }

    fn codegen_statement(&self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Return(expression) => {
                let value = self.codegen_expression(expression);

                self.builder.ret(value);
            }
            StatementKind::Expression(expression) => {
                self.codegen_expression(expression);
            }
            StatementKind::Null => {
                // Literally nothing todo
            }
        }
    }

    fn codegen_declaration(&self, declaration: &Declaration) {
        // Allocate space for the variable on the stack
        let ptr = self.new_stack_variable(&declaration.name);

        // If the declaration has an initializer, codegen the initializer and store it
        // in the variable
        if let Some(initializer) = &declaration.initializer {
            let value = self.codegen_expression(initializer);
            self.builder.store(value, ptr);
        }
    }

    fn codegen_expression(&self, expression: &Expression) -> LLVMValue {
        use ExpressionKind::{BinaryOperation, IntegerLiteral, Parenthesis, UnaryOperation};

        match &expression.kind {
            IntegerLiteral(value) => self.const_int(*value),
            UnaryOperation {
                operator,
                expression,
            } => self.codegen_unary_operation(operator, expression.as_ref()),
            Parenthesis(expression) => self.codegen_expression(expression),
            BinaryOperation {
                operator,
                left,
                right,
            } => self.codegen_binary_operation(operator, left, right),
            ExpressionKind::Variable(name) => self.load_variable(name),
        }
    }

    // -- Binary operations --

    fn codegen_binary_operation(
        &self,
        operator: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        use BinaryOperator::{
            Add, AddAssign, Assignment, BitwiseAnd, BitwiseAndAssign, BitwiseLeftShift,
            BitwiseLeftShiftAssign, BitwiseOr, BitwiseOrAssign, BitwiseRightShift,
            BitwiseRightShiftAssign, BitwiseXor, BitwiseXorAssign, Divide, DivideAssign, Equals,
            GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, LogicalAnd, LogicalOr,
            Multiply, MultiplyAssign, NotEquals, Remainder, RemainderAssign, Subtract,
            SubtractAssign,
        };

        match operator {
            Add => self.codegen_binary_add(left, right),
            Subtract => self.codegen_binary_subtract(left, right),
            Multiply => self.codegen_binary_multiply(left, right),
            Divide => self.codegen_binary_divide(left, right),
            Remainder => self.codegen_binary_remainder(left, right),
            BitwiseAnd => self.codegen_binary_bitwise_and(left, right),
            BitwiseLeftShift => self.codegen_binary_bitwise_left_shift(left, right),
            BitwiseOr => self.codegen_binary_bitwise_or(left, right),
            BitwiseRightShift => self.codegen_binary_bitwise_right_shift(left, right),
            BitwiseXor => self.codegen_binary_bitwise_xor(left, right),
            LogicalAnd => self.codegen_binary_logical_and(left, right),
            LogicalOr => self.codegen_binary_logical_or(left, right),
            Assignment => self.codegen_binary_assignment(left, self.codegen_expression(right)),
            Equals => self.codegen_binary_equals(left, right),
            NotEquals => self.codegen_binary_not_equals(left, right),
            LessThan => self.codegen_binary_less_than(left, right),
            LessThanOrEqual => self.codegen_binary_less_than_or_equal(left, right),
            GreaterThan => self.codegen_binary_greater_than(left, right),
            GreaterThanOrEqual => self.codegen_binary_greater_than_or_equal(left, right),
            AddAssign => self.codegen_binary_add_assign(left, right),
            SubtractAssign => self.codegen_binary_subtract_assign(left, right),
            MultiplyAssign => self.codegen_binary_multiply_assign(left, right),
            DivideAssign => self.codegen_binary_divide_assign(left, right),
            RemainderAssign => self.codegen_binary_remainder_assign(left, right),
            BitwiseLeftShiftAssign => self.codegen_binary_bitwise_left_shift_assign(left, right),
            BitwiseRightShiftAssign => self.codegen_binary_bitwise_right_shift_assign(left, right),
            BitwiseAndAssign => self.codegen_binary_bitwise_and_assign(left, right),
            BitwiseXorAssign => self.codegen_binary_bitwise_xor_assign(left, right),
            BitwiseOrAssign => self.codegen_binary_bitwise_or_assign(left, right),
        }
    }

    fn codegen_binary_add(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.add(left_value, right_value)
    }

    fn codegen_binary_subtract(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.subtract(left_value, right_value)
    }

    fn codegen_binary_multiply(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.multiply(left_value, right_value)
    }

    fn codegen_binary_divide(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.signed_divide(left_value, right_value)
    }

    fn codegen_binary_remainder(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.signed_remainder(left_value, right_value)
    }

    fn codegen_binary_bitwise_and(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.bitwise_and(left_value, right_value)
    }

    fn codegen_binary_bitwise_left_shift(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.bitwise_left_shift(left_value, right_value)
    }

    fn codegen_binary_bitwise_or(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.bitwise_or(left_value, right_value)
    }

    fn codegen_binary_bitwise_right_shift(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder
            .bitwise_arithmetic_right_shift(left_value, right_value)
    }

    fn codegen_binary_bitwise_xor(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        self.builder.bitwise_xor(left_value, right_value)
    }

    fn codegen_binary_logical_and(&self, left: &Expression, right: &Expression) -> LLVMValue {
        // NOTE: Logical AND is a short-circuiting operator.
        // This means that if the left operand is false, the right operand is not
        // evaluated.
        let left_value = self.codegen_expression(left);

        // Check if left_value is zero
        let zero = self.const_int(0);
        let left_is_true = self.builder.integer_not_equal(left_value, zero);

        // Get the current function to create a new basic blocks
        let function = self.get_current_function();

        // Create basic blocks for the right-hand side and the false case
        let rhs_basic_block = self.context.create_basic_block("land.rhs");
        let false_basic_block = self.context.create_basic_block("land.false");
        let true_basic_block = self.context.create_basic_block("land.true");
        let end_basic_block = self.context.create_basic_block("land.end");

        // Conditionally branch to the right-hand side or the false case if left is
        // false
        self.builder
            .conditional_branch(left_is_true, rhs_basic_block, false_basic_block);

        // Build the rhs basic block
        function.append_existing_basic_block(rhs_basic_block);
        self.builder.position_at_end(rhs_basic_block);

        let right_value = self.codegen_expression(right);

        // Check if right_value is not zero
        let right_is_true = self.builder.integer_not_equal(right_value, zero);

        // Branch to the true or false basic block based on right_is_false
        self.builder
            .conditional_branch(right_is_true, true_basic_block, false_basic_block);

        // Build the true basic block
        self.builder.position_at_end(true_basic_block);
        self.builder.unconditional_branch(end_basic_block);

        // Build the false basic block
        self.builder.position_at_end(false_basic_block);
        self.builder.unconditional_branch(end_basic_block);

        // Build the end merger basic block
        self.builder.position_at_end(end_basic_block);

        // Create a PHI node to merge the results
        let phi = self.builder.phi(self.int32_type(), "land.phi");
        let true_value = self.const_int(1);
        let false_value = self.const_int(0);

        // Add incoming values to the PHI node
        phi.add_incoming(
            &[true_value, false_value],
            &[true_basic_block, false_basic_block],
        );

        // Append all our new basic blocks to the function
        function.append_existing_basic_block(false_basic_block);
        function.append_existing_basic_block(true_basic_block);
        function.append_existing_basic_block(end_basic_block);

        phi.value()
    }

    fn codegen_binary_logical_or(&self, left: &Expression, right: &Expression) -> LLVMValue {
        // NOTE: Logical OR is a short-circuiting operator.
        // This means that if the left operand is true, the right operand is not
        // evaluated.
        let left_value = self.codegen_expression(left);

        // Check if left_value is not zero (true)
        let zero = self.const_int(0);
        let left_is_true = self.builder.integer_not_equal(left_value, zero);

        // Get the current function to create a new basic blocks
        let function = self.get_current_function();

        // Create basic blocks for the right-hand side and the true case
        let rhs_basic_block = self.context.create_basic_block("lor.rhs");
        let true_basic_block = self.context.create_basic_block("lor.true");
        let false_basic_block = self.context.create_basic_block("lor.false");
        let end_basic_block = self.context.create_basic_block("lor.end");

        // Conditionally branch to the true case or the right-hand side if left is false
        self.builder
            .conditional_branch(left_is_true, true_basic_block, rhs_basic_block);

        // Build the rhs basic block
        self.builder.position_at_end(rhs_basic_block);
        function.append_existing_basic_block(rhs_basic_block);

        let right_value = self.codegen_expression(right);

        // Check if right_value is not zero
        let right_is_true = self.builder.integer_not_equal(right_value, zero);

        // Branch to the true or false basic block based on right_is_true
        self.builder
            .conditional_branch(right_is_true, true_basic_block, false_basic_block);

        // Build the true basic block
        self.builder.position_at_end(true_basic_block);
        self.builder.unconditional_branch(end_basic_block);

        // Build the false basic block
        self.builder.position_at_end(false_basic_block);
        self.builder.unconditional_branch(end_basic_block);

        // Build the end merger basic block
        self.builder.position_at_end(end_basic_block);

        // Create a PHI node to merge the results
        let phi = self.builder.phi(self.int32_type(), "lor.phi");
        let true_value = self.const_int(1);
        let false_value = self.const_int(0);

        // Add incoming values to the PHI node
        phi.add_incoming(
            &[true_value, false_value],
            &[true_basic_block, false_basic_block],
        );

        // Append all our new basic blocks to the function
        function.append_existing_basic_block(true_basic_block);
        function.append_existing_basic_block(false_basic_block);
        function.append_existing_basic_block(end_basic_block);

        phi.value()
    }

    #[expect(clippy::panic)]
    fn codegen_binary_assignment(&self, left: &Expression, right_value: LLVMValue) -> LLVMValue {
        // The left-hand side of an assignment must be an l-value.
        let Some(name) = left.as_variable_name() else {
            panic!("left-hand side of assignment must be an l-value");
        };

        // Store the right-hand side value into the existing variable
        self.store_variable(name, right_value);

        // The value of an assignment expression is the value that was assigned, so
        // return the codegened right value
        right_value
    }

    fn codegen_binary_equals(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        let value = self.builder.integer_equal(left_value, right_value);
        self.builder.zero_extend(value, self.int32_type())
    }

    fn codegen_binary_not_equals(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        let value = self.builder.integer_not_equal(left_value, right_value);
        self.builder.zero_extend(value, self.int32_type())
    }

    fn codegen_binary_less_than(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        let value = self
            .builder
            .integer_signed_less_than(left_value, right_value);
        self.builder.zero_extend(value, self.int32_type())
    }

    fn codegen_binary_less_than_or_equal(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        let value = self
            .builder
            .integer_signed_less_than_or_equal(left_value, right_value);
        self.builder.zero_extend(value, self.int32_type())
    }

    fn codegen_binary_greater_than(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        let value = self
            .builder
            .integer_signed_greater_than(left_value, right_value);
        self.builder.zero_extend(value, self.int32_type())
    }

    fn codegen_binary_greater_than_or_equal(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        let value = self
            .builder
            .integer_signed_greater_than_or_equal(left_value, right_value);
        self.builder.zero_extend(value, self.int32_type())
    }

    fn codegen_binary_add_assign(&self, left: &Expression, right: &Expression) -> LLVMValue {
        // An add assignment is just an addition followed by an assignment, so we
        // can reuse the codegen for those operations.
        let add = self.codegen_binary_add(left, right);
        self.codegen_binary_assignment(left, add)
    }

    fn codegen_binary_subtract_assign(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let subtract = self.codegen_binary_subtract(left, right);
        self.codegen_binary_assignment(left, subtract)
    }

    fn codegen_binary_multiply_assign(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let multiply = self.codegen_binary_multiply(left, right);
        self.codegen_binary_assignment(left, multiply)
    }

    fn codegen_binary_divide_assign(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let divide = self.codegen_binary_divide(left, right);
        self.codegen_binary_assignment(left, divide)
    }

    fn codegen_binary_remainder_assign(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let remainder = self.codegen_binary_remainder(left, right);
        self.codegen_binary_assignment(left, remainder)
    }

    fn codegen_binary_bitwise_left_shift_assign(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let left_shift = self.codegen_binary_bitwise_left_shift(left, right);
        self.codegen_binary_assignment(left, left_shift)
    }

    fn codegen_binary_bitwise_right_shift_assign(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let right_shift = self.codegen_binary_bitwise_right_shift(left, right);
        self.codegen_binary_assignment(left, right_shift)
    }

    fn codegen_binary_bitwise_and_assign(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let bitwise_and = self.codegen_binary_bitwise_and(left, right);
        self.codegen_binary_assignment(left, bitwise_and)
    }

    fn codegen_binary_bitwise_xor_assign(
        &self,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        let bitwise_xor = self.codegen_binary_bitwise_xor(left, right);
        self.codegen_binary_assignment(left, bitwise_xor)
    }

    fn codegen_binary_bitwise_or_assign(&self, left: &Expression, right: &Expression) -> LLVMValue {
        let bitwise_or = self.codegen_binary_bitwise_or(left, right);
        self.codegen_binary_assignment(left, bitwise_or)
    }

    // -- Unary operations --

    fn codegen_unary_operation(
        &self,
        operator: &UnaryOperator,
        expression: &Expression,
    ) -> LLVMValue {
        use UnaryOperator::{
            Complement, LogicalNot, Negate, Positive, PostDecrement, PostIncrement, PreDecrement,
            PreIncrement,
        };

        match operator {
            Positive => self.codegen_unary_positive(expression),
            Negate => self.codegen_unary_negate(expression),
            Complement => self.codegen_unary_complement(expression),
            LogicalNot => self.codegen_unary_logical_not(expression),
            PreIncrement => self.codegen_unary_prefix_increment(expression),
            PreDecrement => self.codegen_unary_prefix_decrement(expression),
            PostIncrement => self.codegen_unary_postfix_increment(expression),
            PostDecrement => self.codegen_unary_postfix_decrement(expression),
        }
    }

    fn codegen_unary_positive(&self, expression: &Expression) -> LLVMValue {
        self.codegen_expression(expression)
    }

    fn codegen_unary_negate(&self, expression: &Expression) -> LLVMValue {
        let value = self.codegen_expression(expression);
        self.negate(value)
    }

    fn codegen_unary_complement(&self, expression: &Expression) -> LLVMValue {
        let value = self.codegen_expression(expression);
        self.complement(value)
    }

    fn codegen_unary_logical_not(&self, expression: &Expression) -> LLVMValue {
        let value = self.codegen_expression(expression);
        self.logical_not(value)
    }

    #[expect(clippy::panic)]
    fn codegen_unary_prefix_increment(&self, expression: &Expression) -> LLVMValue {
        // The operand of a prefix increment must be an l-value.
        let Some(name) = expression.as_variable_name() else {
            panic!("operand of prefix increment must be an l-value");
        };

        // First load the current value of the variable
        let current_value = self.load_variable(name);

        // Then add 1 to it
        let one = self.const_int(1);
        let new_value = self.builder.add(current_value, one);

        // Then store the new value back in the variable
        self.store_variable(name, new_value);

        // The value of a prefix increment expression is the new value, so return it
        new_value
    }

    #[expect(clippy::panic)]
    fn codegen_unary_prefix_decrement(&self, expression: &Expression) -> LLVMValue {
        // The operand of a prefix decrement must be an l-value.
        let Some(name) = expression.as_variable_name() else {
            panic!("operand of prefix decrement must be an l-value");
        };

        // First load the current value of the variable
        let current_value = self.load_variable(name);

        // Then subtract 1 from it
        let one = self.const_int(1);
        let new_value = self.builder.subtract(current_value, one);

        // Then store the new value back in the variable
        self.store_variable(name, new_value);

        // The value of a prefix decrement expression is the new value, so return it
        new_value
    }

    #[expect(clippy::panic)]
    fn codegen_unary_postfix_increment(&self, expression: &Expression) -> LLVMValue {
        // The operand of a postfix increment must be an l-value.
        let Some(name) = expression.as_variable_name() else {
            panic!("operand of postfix increment must be an l-value");
        };

        // First load the current value of the variable
        let current_value = self.load_variable(name);

        // Then add 1 to it
        let one = self.const_int(1);
        let new_value = self.builder.add(current_value, one);

        // Then store the new value back in the variable
        self.store_variable(name, new_value);

        // The value of a postfix increment expression is the old value, so return the
        // current value
        current_value
    }

    #[expect(clippy::panic)]
    fn codegen_unary_postfix_decrement(&self, expression: &Expression) -> LLVMValue {
        // The operand of a postfix decrement must be an l-value.
        let Some(name) = expression.as_variable_name() else {
            panic!("operand of postfix decrement must be an l-value");
        };

        // First load the current value of the variable
        let current_value = self.load_variable(name);

        // Then subtract 1 from it
        let one = self.const_int(1);
        let new_value = self.builder.subtract(current_value, one);

        // Then store the new value back in the variable
        self.store_variable(name, new_value);

        // The value of a postfix decrement expression is the old value, so return the
        // current value
        current_value
    }
}
