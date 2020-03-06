// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use move_core_types::identifier::Identifier;
use std::collections::HashMap;
use test_generation::abstract_state::{AbstractState, AbstractValue, CallGraph};
use vm::{
    access::ModuleAccess,
    file_format::{
        empty_module, Bytecode, CompiledModule, CompiledModuleMut, FieldDefinition,
        FieldDefinitionIndex, IdentifierIndex, Kind, LocalsSignatureIndex, MemberCount,
        ModuleHandleIndex, SignatureToken, StructDefinition, StructDefinitionIndex,
        StructFieldInformation, StructHandle, StructHandleIndex, TableIndex, TypeSignature,
        TypeSignatureIndex,
    },
    views::{SignatureTokenView, StructDefinitionView, ViewInternals},
};

mod common;

fn generate_module_with_struct(resource: bool) -> CompiledModuleMut {
    let mut module: CompiledModuleMut = empty_module();
    module.type_signatures = vec![
        SignatureToken::Bool,
        SignatureToken::U64,
        SignatureToken::Vector(Box::new(SignatureToken::U8)),
        SignatureToken::Address,
    ]
    .into_iter()
    .map(TypeSignature)
    .collect();

    let struct_index = 0;
    let num_fields = 5;
    let offset = module.identifiers.len() as TableIndex;
    module.identifiers.push(Identifier::new("struct0").unwrap());

    let field_information = StructFieldInformation::Declared {
        field_count: num_fields as MemberCount,
        fields: FieldDefinitionIndex::new(module.field_defs.len() as TableIndex),
    };
    let struct_def = StructDefinition {
        struct_handle: StructHandleIndex(struct_index),
        field_information,
    };
    module.struct_defs.push(struct_def);

    for i in 0..num_fields {
        module
            .identifiers
            .push(Identifier::new(format!("string{}", i)).unwrap());
        let struct_handle_idx = StructHandleIndex::new(struct_index);
        let typ_idx = TypeSignatureIndex::new(0);
        let str_pool_idx = IdentifierIndex::new(i + 1 as TableIndex);
        let field_def = FieldDefinition {
            struct_: struct_handle_idx,
            name: str_pool_idx,
            signature: typ_idx,
        };
        module.field_defs.push(field_def);
    }
    module.struct_handles = vec![StructHandle {
        module: ModuleHandleIndex::new(0),
        name: IdentifierIndex::new((struct_index + offset) as TableIndex),
        is_nominal_resource: resource,
        type_formals: vec![],
    }];
    module
}

fn create_struct_value(module: &CompiledModule) -> (AbstractValue, Vec<SignatureToken>) {
    let struct_def = module.struct_def_at(StructDefinitionIndex::new(0));
    let struct_def_view = StructDefinitionView::new(module, struct_def);
    let tokens: Vec<SignatureToken> = struct_def_view
        .fields()
        .into_iter()
        .flatten()
        .map(|field| field.type_signature().token().as_inner().clone())
        .collect();
    let struct_kind = if struct_def_view.is_nominal_resource() {
        Kind::Resource
    } else {
        tokens
            .iter()
            .map(|token| SignatureTokenView::new(module, token).kind(&[]))
            .fold(Kind::Unrestricted, |acc_kind, next_kind| {
                match (acc_kind, next_kind) {
                    (Kind::All, _) | (_, Kind::All) => Kind::All,
                    (Kind::Resource, _) | (_, Kind::Resource) => Kind::Resource,
                    (Kind::Unrestricted, Kind::Unrestricted) => Kind::Unrestricted,
                }
            })
    };
    (
        AbstractValue::new_struct(
            SignatureToken::Struct(struct_def.struct_handle, vec![]),
            struct_kind,
        ),
        tokens,
    )
}

#[test]
#[should_panic]
fn bytecode_pack_signature_not_satisfied() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::Pack(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
fn bytecode_pack() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let (struct_value1, tokens) = create_struct_value(&state1.module.module);
    for token in tokens {
        let abstract_value = AbstractValue {
            token: token.clone(),
            kind: SignatureTokenView::new(&state1.module.module, &token).kind(&[]),
        };
        state1.stack_push(abstract_value);
    }
    let (state2, _) = common::run_instruction(
        Bytecode::Pack(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    let struct_value2 = state2.stack_peek(0).expect("struct not added to stack");
    assert_eq!(
        struct_value1, struct_value2,
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_unpack_signature_not_satisfied() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::Unpack(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
fn bytecode_unpack() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let (struct_value, tokens) = create_struct_value(&state1.module.module);
    state1.stack_push(struct_value);
    let (state2, _) = common::run_instruction(
        Bytecode::Unpack(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    assert_eq!(
        state2.stack_len(),
        tokens.len(),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_exists() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    let (state2, _) = common::run_instruction(
        Bytecode::Exists(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_exists_struct_is_not_resource() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    common::run_instruction(
        Bytecode::Exists(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
#[should_panic]
fn bytecode_exists_no_address_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::Exists(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
fn bytecode_movefrom() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 = AbstractState::from_locals(
        module,
        HashMap::new(),
        vec![],
        vec![StructDefinitionIndex::new(0)],
        CallGraph::new(0),
    );
    let state1_copy = state1.clone();
    let struct_def = state1_copy
        .module
        .module
        .struct_def_at(StructDefinitionIndex::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    let (state2, _) = common::run_instruction(
        Bytecode::MoveFrom(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    let struct_value = state2.stack_peek(0).expect("struct not added to stack");
    assert!(
        match struct_value.token {
            SignatureToken::Struct(struct_handle, _) => struct_handle == struct_def.struct_handle,
            _ => false,
        },
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_movefrom_struct_is_not_resource() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    common::run_instruction(
        Bytecode::MoveFrom(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
#[should_panic]
fn bytecode_movefrom_no_address_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::MoveFrom(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
fn bytecode_movetosender() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(create_struct_value(&state1.module.module).0);
    let (state2, _) = common::run_instruction(
        Bytecode::MoveToSender(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    assert_eq!(state2.stack_len(), 0, "stack type postcondition not met");
}

#[test]
#[should_panic]
fn bytecode_movetosender_struct_is_not_resource() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(create_struct_value(&state1.module.module).0);
    common::run_instruction(
        Bytecode::MoveToSender(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
#[should_panic]
fn bytecode_movetosender_no_struct_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::MoveToSender(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
fn bytecode_mutborrowfield() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::MutableReference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    let field_index = FieldDefinitionIndex::new(0);
    let (state2, _) = common::run_instruction(Bytecode::MutBorrowField(field_index), state1);
    let field_signature = state2
        .module
        .module
        .get_field_signature(field_index)
        .0
        .clone();
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue {
            token: SignatureToken::MutableReference(Box::new(field_signature.clone())),
            kind: SignatureTokenView::new(&state2.module.module, &field_signature).kind(&[]),
        }),
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_mutborrowfield_stack_has_no_reference() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let field_index = FieldDefinitionIndex::new(0);
    common::run_instruction(Bytecode::MutBorrowField(field_index), state1);
}

#[test]
#[should_panic]
fn bytecode_mutborrowfield_ref_is_immutable() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::Reference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    let field_index = FieldDefinitionIndex::new(0);
    common::run_instruction(Bytecode::MutBorrowField(field_index), state1);
}

#[test]
fn bytecode_immborrowfield() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::Reference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    let field_index = FieldDefinitionIndex::new(0);
    let (state2, _) = common::run_instruction(Bytecode::ImmBorrowField(field_index), state1);
    let field_signature = state2
        .module
        .module
        .get_field_signature(field_index)
        .0
        .clone();
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue {
            token: SignatureToken::MutableReference(Box::new(field_signature.clone())),
            kind: SignatureTokenView::new(&state2.module.module, &field_signature).kind(&[]),
        }),
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_immborrowfield_stack_has_no_reference() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let field_index = FieldDefinitionIndex::new(0);
    common::run_instruction(Bytecode::ImmBorrowField(field_index), state1);
}

#[test]
#[should_panic]
fn bytecode_immborrowfield_ref_is_mutable() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::MutableReference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    let field_index = FieldDefinitionIndex::new(0);
    common::run_instruction(Bytecode::ImmBorrowField(field_index), state1);
}

#[test]
fn bytecode_borrowglobal() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    let (state2, _) = common::run_instruction(
        Bytecode::MutBorrowGlobal(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue {
            token: SignatureToken::MutableReference(Box::new(struct_value.token)),
            kind: struct_value.kind,
        }),
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_borrowglobal_struct_is_not_resource() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    common::run_instruction(
        Bytecode::MutBorrowGlobal(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
#[should_panic]
fn bytecode_borrowglobal_no_address_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::MutBorrowGlobal(StructDefinitionIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}
