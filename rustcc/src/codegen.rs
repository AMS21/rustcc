use std::{ffi::CString, ptr};

use libc::c_uint;
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildRet, LLVMConstInt,
        LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilder, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDisposeModule, LLVMDumpModule, LLVMFunctionType,
        LLVMInt1TypeInContext, LLVMInt8TypeInContext, LLVMInt16TypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt128TypeInContext,
        LLVMIntTypeInContext, LLVMModuleCreateWithName, LLVMModuleCreateWithNameInContext,
        LLVMPositionBuilderAtEnd, LLVMSetSourceFileName,
    },
    prelude::{
        LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef,
    },
};

use crate::ast::{
    Expression, ExpressionKind, FunctionDefinition, Statement, StatementKind, TranslationUnit,
};

#[derive(Debug)]
pub struct Codegen {
    builder: LLVMBuilder,
    module: LLVMModule,
    context: LLVMContext,
}

impl Codegen {
    pub fn new(file_path: &str) -> Self {
        let module_name = CString::new(file_path).unwrap();

        let context = LLVMContext::new();
        let module = LLVMModule::new_in_context(module_name.clone(), &context);
        let builder = LLVMBuilder::new_in_context(&context);

        module.set_source_file_name(module_name);

        Codegen {
            builder,
            module,
            context,
        }
    }

    pub fn dump(&self) {
        unsafe { LLVMDumpModule(self.module.0) };
    }

    #[must_use]
    fn int32_type(&self) -> LLVMTypeRef {
        self.context.int32_type()
    }

    #[must_use]
    fn function_type(&self, return_type: LLVMTypeRef) -> LLVMTypeRef {
        unsafe { LLVMFunctionType(return_type, ptr::null_mut(), 0, 0) }
    }

    fn function(&self, name: &str, function_type: LLVMTypeRef) -> LLVMValueRef {
        let Ok(function_name) = CString::new(name) else {
            return ptr::null_mut();
        };

        self.module.add_function(function_name, function_type)
    }

    fn function_basic_block(&self, name: &str, function: LLVMValueRef) -> LLVMBasicBlockRef {
        let Ok(block_name) = CString::new(name) else {
            return ptr::null_mut();
        };

        let basic_block = self
            .context
            .create_basic_block_for_function(function, block_name);

        // Move the builder to the end of the basic block
        self.builder.position_at_end(basic_block);

        basic_block
    }

    #[must_use]
    fn const_int(&self, value: u32) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.int32_type(), value as u64, 0) }
    }

    pub fn codegen(&self, translation_unit: &TranslationUnit) -> Option<()> {
        // Code gen all functions
        for function in &translation_unit.function {
            self.codegen_function(function);
        }

        Some(())
    }

    fn codegen_function(&self, function: &FunctionDefinition) -> Option<()> {
        // Create the function type
        let function_type = self.function_type(self.int32_type());

        // Create the function
        let llvm_function = self.function(&function.name, function_type);
        if llvm_function.is_null() {
            return None;
        }

        // Create a basic block in the function and set our builder to generate
        // code in it.
        self.function_basic_block("entry", llvm_function);

        // Codegen the function body
        self.codegen_statement(&function.body);

        // Verify generated function
        unsafe {
            LLVMVerifyFunction(
                llvm_function,
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
            )
        };

        Some(())
    }

    fn codegen_statement(&self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Return(expression) => {
                let value = self.codegen_expression(expression);

                self.builder.ret(value);
            }
        }
    }

    fn codegen_expression(&self, expression: &Expression) -> LLVMValueRef {
        match expression.kind {
            ExpressionKind::IntegerLiteral(value) => self.const_int(value),
        }
    }
}

// -- LLVM Wrappers --

#[derive(Debug)]
struct LLVMContext(LLVMContextRef);

impl LLVMContext {
    pub fn new() -> Self {
        let context = unsafe { LLVMContextCreate() };
        LLVMContext(context)
    }

    pub fn int1_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt1TypeInContext(self.0) }
    }

    pub fn int8_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt8TypeInContext(self.0) }
    }

    pub fn int16_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt16TypeInContext(self.0) }
    }

    pub fn int32_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt32TypeInContext(self.0) }
    }

    pub fn int64_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt64TypeInContext(self.0) }
    }

    pub fn int128_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt128TypeInContext(self.0) }
    }

    pub fn int_type(&self, num_bits: c_uint) -> LLVMTypeRef {
        unsafe { LLVMIntTypeInContext(self.0, num_bits) }
    }

    pub fn create_basic_block_for_function(
        &self,
        function: LLVMValueRef,
        name: CString,
    ) -> LLVMBasicBlockRef {
        unsafe { LLVMAppendBasicBlockInContext(self.0, function, name.as_ptr()) }
    }
}

impl Drop for LLVMContext {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.0) };
    }
}

#[derive(Debug)]
struct LLVMModule(LLVMModuleRef);

impl LLVMModule {
    pub fn new_global<S: Into<CString>>(name: S) -> Self {
        let module = unsafe { LLVMModuleCreateWithName(name.into().as_ptr()) };
        LLVMModule(module)
    }

    pub fn new_in_context<S: Into<CString>>(name: S, context: &LLVMContext) -> Self {
        let module = unsafe { LLVMModuleCreateWithNameInContext(name.into().as_ptr(), context.0) };
        LLVMModule(module)
    }

    pub fn set_source_file_name(&self, name: CString) {
        unsafe { LLVMSetSourceFileName(self.0, name.as_ptr(), name.as_bytes().len()) };
    }

    pub fn add_function(&self, name: CString, function_type: LLVMTypeRef) -> LLVMValueRef {
        unsafe { LLVMAddFunction(self.0, name.as_ptr(), function_type) }
    }
}

impl Drop for LLVMModule {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.0) };
    }
}

#[derive(Debug)]
struct LLVMBuilder(LLVMBuilderRef);

impl LLVMBuilder {
    pub fn new_global() -> Self {
        let builder = unsafe { LLVMCreateBuilder() };
        LLVMBuilder(builder)
    }

    pub fn new_in_context(context: &LLVMContext) -> Self {
        let builder = unsafe { LLVMCreateBuilderInContext(context.0) };
        LLVMBuilder(builder)
    }

    fn position_at_end(&self, basic_block: LLVMBasicBlockRef) {
        unsafe { LLVMPositionBuilderAtEnd(self.0, basic_block) };
    }

    fn ret(&self, value: LLVMValueRef) {
        unsafe { LLVMBuildRet(self.0, value) };
    }
}

impl Drop for LLVMBuilder {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.0) };
    }
}
