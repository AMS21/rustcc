use rustcc_ast::{
    binary_operator::BinaryOperator,
    expression::{Expression, ExpressionKind},
    function_definition::FunctionDefinition,
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

    pub fn codegen(&self, translation_unit: &TranslationUnit) {
        // Code gen all functions
        for function in &translation_unit.function {
            self.codegen_function(function);
        }
    }

    fn codegen_function(&self, function: &FunctionDefinition) -> bool {
        // Create the function type
        let function_type = function_type(self.int32_type());

        // Create the function
        let llvm_function = self.function(&function.name, function_type);

        // Create a basic block in the function and set our builder to generate
        // code in it.
        self.function_basic_block("entry", llvm_function);

        // Codegen the function body
        self.codegen_statement(&function.body);

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
        }
    }

    fn codegen_binary_operation(
        &self,
        operator: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValue {
        use BinaryOperator::{
            Add, Assignment, BitwiseAnd, BitwiseLeftShift, BitwiseOr, BitwiseRightShift,
            BitwiseXor, Divide, Equals, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual,
            LogicalAnd, LogicalOr, Multiply, NotEquals, Remainder, Subtract,
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
            Assignment => {
                // TODO: For now, we don't support variables, so just evaluate the right-hand
                // side
                self.codegen_expression(right)
            }
            Equals => self.codegen_binary_equals(left, right),
            NotEquals => self.codegen_binary_not_equals(left, right),
            LessThan => self.codegen_binary_less_than(left, right),
            LessThanOrEqual => self.codegen_binary_less_than_or_equal(left, right),
            GreaterThan => self.codegen_binary_greater_than(left, right),
            GreaterThanOrEqual => self.codegen_binary_greater_than_or_equal(left, right),
        }
    }

    fn codegen_unary_operation(
        &self,
        operator: &UnaryOperator,
        expression: &Expression,
    ) -> LLVMValue {
        use UnaryOperator::{Complement, LogicalNot, Negate, Positive};

        let value = self.codegen_expression(expression);

        match operator {
            Positive => value,
            Negate => self.negate(value),
            Complement => self.complement(value),
            LogicalNot => self.logical_not(value),
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
}
