use chrn_utils::id_types::TypeId;
use lang::types::builtins::BuiltinType;

use crate::{
    id_tag_decls::FuncTag,
    parser::ast::ast_concepts::BinaryOp,
    script_compiler::{ScriptCompiler, compiler_consts},
    semantic::{
        hir::hir_concepts::{BuiltinTypeInfo, Type, TypeInfo},
        values::Value,
    },
};

/// Attempts to infer a type using `Value` directly
pub(crate) fn infer_type_from_val(compiler: &mut ScriptCompiler, val: &Value) -> Option<TypeId> {
    let mut val = val;
    let mut depth = 0;
    while let Value::List(elements) = val {
        let val_id = *elements.first()?;
        val = &compiler.vals[val_id];
        depth += 1;
    }

    let mut type_id = match val {
        Value::ArbitraryInt(kind) => kind.type_id(),
        Value::ArbitraryFloat(kind) => kind.type_id(),
        Value::Bool(_) => TypeId::new(compiler_consts::CORE_BOOL),
        Value::Char(_) => TypeId::new(compiler_consts::CORE_CHAR),
        Value::Func(func_sym) => {
            compiler
                .get_func(func_sym.into_tagged::<FuncTag>())
                .ret_type
        }
        Value::InternedStr(_) => TypeId::new(compiler_consts::CORE_STR),
        // Tuples only express type constraints; runtime values cannot result from constant evaluation.
        Value::Tuple(_) | Value::RuntimeStr(_) => unreachable!(),
        Value::Unknown => return None,
        // Previous loop makes this impossible
        Value::List(_) => unreachable!(),
    };

    // Preserving every array layer. Allocating types after.
    for _ in 0..depth {
        type_id = compiler.push_ty(TypeInfo::new(
            Type::BuiltinTypeInfo(BuiltinTypeInfo::new(BuiltinType::List(type_id))),
            compiler.intrinsic_registry.core_mod_id,
        ));
    }
    Some(type_id)
}

/// Attempts to infer a type using known binary-op semantics
pub(crate) fn infer_type_from_binary_op(
    lhs_type_id: TypeId,
    rhs_type_id: TypeId,
    lhs_is_unknown: bool,
    op: BinaryOp,
    rhs_is_unknown: bool,
) -> Option<TypeId> {
    match op {
        // Maybe this can still point to unknown?
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mult | BinaryOp::Div | BinaryOp::Mod => {
            if lhs_is_unknown && rhs_is_unknown {
                None
            } else if rhs_is_unknown {
                Some(lhs_type_id)
            } else {
                Some(rhs_type_id)
            }
        }
        BinaryOp::Greater
        | BinaryOp::Less
        | BinaryOp::GreaterOrEq
        | BinaryOp::And
        | BinaryOp::Or
        | BinaryOp::EqTo
        | BinaryOp::NotEq
        | BinaryOp::LessOrEq => Some(TypeId::new(compiler_consts::CORE_BOOL)),
        BinaryOp::BitOr
        | BinaryOp::BitAnd
        | BinaryOp::BitRightShift
        | BinaryOp::BitLeftShift
        | BinaryOp::BitXor => Some(TypeId::new(compiler_consts::CORE_I64)),
    }
}
