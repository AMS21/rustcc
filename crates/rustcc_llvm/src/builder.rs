use std::ffi::CString;

use crate::{
    basic_block::LLVMBasicBlock,
    context::LLVMContext,
    ffi::{
        LLVMBuildAShr, LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildBr, LLVMBuildCondBr,
        LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr,
        LLVMBuildPhi, LLVMBuildRet, LLVMBuildSDiv, LLVMBuildSExt, LLVMBuildSRem, LLVMBuildShl,
        LLVMBuildStore, LLVMBuildSub, LLVMBuildUnreachable, LLVMBuildXor, LLVMBuildZExt,
        LLVMBuilderRef, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMGetInsertBlock,
        LLVMIntPredicate, LLVMPositionBuilderAtEnd,
    },
    phi::LLVMPhi,
    to_cstring,
    typ::LLVMType,
    value::LLVMValue,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct LLVMBuilder(LLVMBuilderRef);

impl LLVMBuilder {
    #[must_use]
    pub fn new_in_context(context: &LLVMContext) -> Option<Self> {
        // Safety: LLVMCreateBuilderInContext is safe to call with a valid
        // LLVMContextRef.
        let builder = unsafe { LLVMCreateBuilderInContext(context.as_raw()) };

        if builder.is_null() {
            None
        } else {
            Some(Self(builder))
        }
    }

    pub fn position_at_end(&self, basic_block: LLVMBasicBlock) {
        // Safety: LLVMPositionBuilderAtEnd is safe to call with valid parameters.
        unsafe { LLVMPositionBuilderAtEnd(self.0, basic_block.as_raw()) };
    }

    #[must_use]
    pub fn get_insert_block(&self) -> Option<LLVMBasicBlock> {
        // Safety: LLVMGetInsertBlock is safe to call with a valid LLVMBuilderRef.
        let block = unsafe { LLVMGetInsertBlock(self.0) };
        if block.is_null() {
            None
        } else {
            Some(LLVMBasicBlock::from_raw(block))
        }
    }

    /// Returns true if the current insert block has a terminator instruction.
    #[must_use]
    pub fn has_insert_block_terminator(&self) -> bool {
        self.get_insert_block()
            .and_then(|basic_block| basic_block.terminator())
            .is_some()
    }

    pub fn ret(&self, value: LLVMValue) {
        // Safety: LLVMBuildRet is safe to call with valid parameters.
        unsafe { LLVMBuildRet(self.0, value.as_raw()) };
    }

    #[must_use]
    pub fn alloca(&self, typ: LLVMType, name: &str) -> LLVMValue {
        let name = to_cstring(name);
        // Safety: LLVMBuildAlloca is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe { LLVMBuildAlloca(self.0, typ.as_raw(), name.as_ptr()) })
    }

    #[expect(
        clippy::must_use_candidate,
        reason = "The store instruction does not produce a value that can be used in further \
                  computations."
    )]
    pub fn store(&self, value: LLVMValue, ptr: LLVMValue) -> LLVMValue {
        // Safety: LLVMBuildStore is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe { LLVMBuildStore(self.0, value.as_raw(), ptr.as_raw()) })
    }

    #[must_use]
    pub fn load(&self, typ: LLVMType, ptr: LLVMValue, name: &str) -> LLVMValue {
        let load_name = format!("{name}.load");
        let name = to_cstring(&*load_name);
        // Safety: LLVMBuildLoad2 is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildLoad2(self.0, typ.as_raw(), ptr.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn negate(&self, value: LLVMValue) -> LLVMValue {
        let name = to_cstring("neg");
        // Safety: LLVMBuildNeg is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe { LLVMBuildNeg(self.0, value.as_raw(), name.as_ptr()) })
    }

    #[must_use]
    pub fn add(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("add");
        // Safety: LLVMBuildAdd is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildAdd(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn subtract(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("sub");
        // Safety: LLVMBuildSub is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildSub(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn multiply(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("mul");
        // Safety: LLVMBuildMul is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildMul(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn signed_divide(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("sdiv");
        // Safety: LLVMBuildSDiv is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildSDiv(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn unsigned_divide(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("udiv");
        // Safety: LLVMBuildUDiv is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            crate::ffi::LLVMBuildUDiv(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn signed_remainder(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("srem");
        // Safety: LLVMBuildSRem is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildSRem(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn unsigned_remainder(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("urem");
        // Safety: LLVMBuildURem is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            crate::ffi::LLVMBuildURem(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    // --- bitwise operations ---

    #[must_use]
    pub fn bitwise_and(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("and");
        // Safety: LLVMBuildAnd is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildAnd(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn bitwise_left_shift(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("shl");
        // Safety: LLVMBuildShl is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildShl(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn bitwise_arithmetic_right_shift(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("ashr");
        // Safety: LLVMBuildAShr is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildAShr(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn bitwise_logical_right_shift(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("lshr");
        // Safety: LLVMBuildLShr is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            crate::ffi::LLVMBuildLShr(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn bitwise_xor(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("xor");
        // Safety: LLVMBuildXor is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildXor(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn bitwise_or(&self, left: LLVMValue, right: LLVMValue) -> LLVMValue {
        let name = to_cstring("or");
        // Safety: LLVMBuildOr is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildOr(self.0, left.as_raw(), right.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn bitwise_complement(&self, value: LLVMValue) -> LLVMValue {
        let name = to_cstring("not");
        // Safety: LLVMBuildNot is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe { LLVMBuildNot(self.0, value.as_raw(), name.as_ptr()) })
    }

    // --- integer comparison ---

    #[must_use]
    fn integer_compare(
        &self,
        predicate: LLVMIntPredicate,
        lhs: LLVMValue,
        rhs: LLVMValue,
        name: &CString,
    ) -> LLVMValue {
        // Safety: LLVMBuildICmp is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildICmp(self.0, predicate, lhs.as_raw(), rhs.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn integer_equal(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_eq");
        self.integer_compare(LLVMIntPredicate::LLVMIntEQ, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_not_equal(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_ne");
        self.integer_compare(LLVMIntPredicate::LLVMIntNE, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_unsigned_greater_than(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_ugt");
        self.integer_compare(LLVMIntPredicate::LLVMIntUGT, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_unsigned_greater_than_or_equal(
        &self,
        lhs: LLVMValue,
        rhs: LLVMValue,
    ) -> LLVMValue {
        let name = to_cstring("icmp_uge");
        self.integer_compare(LLVMIntPredicate::LLVMIntUGE, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_unsigned_less_than(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_ult");
        self.integer_compare(LLVMIntPredicate::LLVMIntULT, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_unsigned_less_than_or_equal(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_ule");
        self.integer_compare(LLVMIntPredicate::LLVMIntULE, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_signed_greater_than(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_sgt");
        self.integer_compare(LLVMIntPredicate::LLVMIntSGT, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_signed_greater_than_or_equal(
        &self,
        lhs: LLVMValue,
        rhs: LLVMValue,
    ) -> LLVMValue {
        let name = to_cstring("icmp_sge");
        self.integer_compare(LLVMIntPredicate::LLVMIntSGE, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_signed_less_than(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_slt");
        self.integer_compare(LLVMIntPredicate::LLVMIntSLT, lhs, rhs, &name)
    }

    #[must_use]
    pub fn integer_signed_less_than_or_equal(&self, lhs: LLVMValue, rhs: LLVMValue) -> LLVMValue {
        let name = to_cstring("icmp_sle");
        self.integer_compare(LLVMIntPredicate::LLVMIntSLE, lhs, rhs, &name)
    }

    // --- type conversion ---

    #[must_use]
    pub fn zero_extend(&self, value: LLVMValue, to_type: LLVMType) -> LLVMValue {
        let name = to_cstring("zext");
        // Safety: LLVMBuildZExt is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildZExt(self.0, value.as_raw(), to_type.as_raw(), name.as_ptr())
        })
    }

    #[must_use]
    pub fn sign_extend(&self, value: LLVMValue, to_type: LLVMType) -> LLVMValue {
        let name = to_cstring("sext");
        // Safety: LLVMBuildSExt is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildSExt(self.0, value.as_raw(), to_type.as_raw(), name.as_ptr())
        })
    }

    // --- branching and control flow ---

    #[expect(clippy::must_use_candidate)]
    pub fn conditional_branch(
        &self,
        condition: LLVMValue,
        then_bb: LLVMBasicBlock,
        else_bb: LLVMBasicBlock,
    ) -> LLVMValue {
        // Safety: LLVMBuildCondBr is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe {
            LLVMBuildCondBr(
                self.0,
                condition.as_raw(),
                then_bb.as_raw(),
                else_bb.as_raw(),
            )
        })
    }

    #[expect(clippy::must_use_candidate)]
    pub fn unconditional_branch(&self, target_bb: LLVMBasicBlock) -> LLVMValue {
        // Safety: LLVMBuildBr is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe { LLVMBuildBr(self.0, target_bb.as_raw()) })
    }

    #[must_use]
    pub fn phi<'a, S: Into<&'a str>>(&self, typ: LLVMType, name: S) -> LLVMPhi {
        let name = to_cstring(name.into());
        // Safety: LLVMBuildPhi is safe to call with valid parameters.
        LLVMPhi::from_raw(unsafe { LLVMBuildPhi(self.0, typ.as_raw(), name.as_ptr()) })
    }

    #[expect(clippy::must_use_candidate)]
    pub fn unreachable(&self) -> LLVMValue {
        // Safety: LLVMBuildUnreachable is safe to call with valid parameters.
        LLVMValue::from_raw(unsafe { LLVMBuildUnreachable(self.0) })
    }
}

impl Drop for LLVMBuilder {
    fn drop(&mut self) {
        // Safety: LLVMDisposeBuilder is safe to call with a valid LLVMBuilderRef.
        unsafe { LLVMDisposeBuilder(self.0) };
    }
}
