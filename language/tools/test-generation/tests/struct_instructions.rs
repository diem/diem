// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use move_core_types::identifier::Identifier;
use std::collections::HashMap;
use test_generation::{
    abstract_state::{AbstractState, AbstractValue, CallGraph},
    kind,
};
use vm::{
    access::ModuleAccess,
    file_format::{
        empty_module, Bytecode, CompiledModule, CompiledModuleMut, FieldDefinition, FieldHandle,
        FieldHandleIndex, IdentifierIndex, Kind, ModuleHandleIndex, SignatureToken,
        StructDefinition, StructDefinitionIndex, StructFieldInformation, StructHandle,
        StructHandleIndex, TableIndex, TypeSignature,
    },
    views::{StructDefinitionView, ViewInternals},
};

mod common;

fn generate_module_with_struct(resource: bool) -> CompiledModuleMut {
    let mut module: CompiledModuleMut = empty_module();

    let struct_index = 0;
    let num_fields = 5;
    let offset = module.identifiers.len() as TableIndex;
    module.identifiers.push(Identifier::new("struct0").unwrap());

    let mut fields = vec![];
    for i in 0..num_fields {
        module
            .identifiers
            .push(Identifier::new(format!("string{}", i)).unwrap());
        let str_pool_idx = IdentifierIndex::new(i + 1 as TableIndex);
        fields.push(FieldDefinition {
            name: str_pool_idx,
            signature: TypeSignature(SignatureToken::Bool),
        });
    }
    let struct_def = StructDefinition {
        struct_handle: StructHandleIndex(struct_index),
        field_information: StructFieldInformation::Declared(fields),
    };
    module.struct_defs.push(struct_def);
    module.struct_handles = vec![StructHandle {
        module: ModuleHandleIndex::new(0),
        name: IdentifierIndex::new((struct_index + offset) as TableIndex),
        is_nominal_resource: resource,
        type_parameters: vec![],
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
        tokens.iter().map(|token| kind(module, token, &[])).fold(
            Kind::Copyable,
            |acc_kind, next_kind| match (acc_kind, next_kind) {
                (Kind::All, _) | (_, Kind::All) => Kind::All,
                (Kind::Resource, _) | (_, Kind::Resource) => Kind::Resource,
                (Kind::Copyable, Kind::Copyable) => Kind::Copyable,
            },
        )
    };
    (
        AbstractValue::new_struct(
            SignatureToken::Struct(struct_def.struct_handle),
            struct_kind,
        ),
        tokens,
    )
}

fn get_field_signature<'a>(
    module: &'a CompiledModuleMut,
    handle: &FieldHandle,
) -> &'a SignatureToken {
    let struct_def = &module.struct_defs[handle.owner.0 as usize];
    match &struct_def.field_information {
        StructFieldInformation::Native => panic!("borrow field on a native struct"),
        StructFieldInformation::Declared(fields) => &fields[handle.field as usize].signature.0,
    }
}

#[test]
#[should_panic]
fn bytecode_pack_signature_not_satisfied() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(Bytecode::Pack(StructDefinitionIndex::new(0)), state1);
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
            kind: kind(&state1.module.module, &token, &[]),
        };
        state1.stack_push(abstract_value);
    }
    let (state2, _) =
        common::run_instruction(Bytecode::Pack(StructDefinitionIndex::new(0)), state1);
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
    common::run_instruction(Bytecode::Unpack(StructDefinitionIndex::new(0)), state1);
}

#[test]
fn bytecode_unpack() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let (struct_value, tokens) = create_struct_value(&state1.module.module);
    state1.stack_push(struct_value);
    let (state2, _) =
        common::run_instruction(Bytecode::Unpack(StructDefinitionIndex::new(0)), state1);
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
    let (state2, _) =
        common::run_instruction(Bytecode::Exists(StructDefinitionIndex::new(0)), state1);
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
    common::run_instruction(Bytecode::Exists(StructDefinitionIndex::new(0)), state1);
}

#[test]
#[should_panic]
fn bytecode_exists_no_address_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(Bytecode::Exists(StructDefinitionIndex::new(0)), state1);
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
    let (state2, _) =
        common::run_instruction(Bytecode::MoveFrom(StructDefinitionIndex::new(0)), state1);
    let struct_value = state2.stack_peek(0).expect("struct not added to stack");
    assert!(
        matches!(struct_value.token, SignatureToken::Struct(struct_handle) if struct_handle == struct_def.struct_handle),
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
    common::run_instruction(Bytecode::MoveFrom(StructDefinitionIndex::new(0)), state1);
}

#[test]
#[should_panic]
fn bytecode_movefrom_no_address_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(Bytecode::MoveFrom(StructDefinitionIndex::new(0)), state1);
}

#[test]
fn bytecode_moveto() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_reference(
        SignatureToken::Reference(Box::new(SignatureToken::Signer)),
        Kind::Copyable,
    ));
    state1.stack_push(create_struct_value(&state1.module.module).0);
    let (state2, _) =
        common::run_instruction(Bytecode::MoveTo(StructDefinitionIndex::new(0)), state1);
    assert_eq!(state2.stack_len(), 0, "stack type postcondition not met");
}

#[test]
#[should_panic]
fn bytecode_moveto_struct_is_not_resource() {
    let module: CompiledModuleMut = generate_module_with_struct(false);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_reference(
        SignatureToken::Reference(Box::new(SignatureToken::Signer)),
        Kind::Copyable,
    ));
    state1.stack_push(create_struct_value(&state1.module.module).0);
    common::run_instruction(Bytecode::MoveTo(StructDefinitionIndex::new(0)), state1);
}

#[test]
#[should_panic]
fn bytecode_moveto_no_struct_on_stack() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_reference(
        SignatureToken::Reference(Box::new(SignatureToken::Signer)),
        Kind::Copyable,
    ));
    common::run_instruction(Bytecode::MoveTo(StructDefinitionIndex::new(0)), state1);
}

#[test]
fn bytecode_mutborrowfield() {
    let mut module: CompiledModuleMut = generate_module_with_struct(false);
    let struct_def_idx = StructDefinitionIndex((module.struct_defs.len() - 1) as u16);
    module.field_handles.push(FieldHandle {
        owner: struct_def_idx,
        field: 0,
    });
    let field_handle_idx = FieldHandleIndex((module.field_handles.len() - 1) as u16);
    let field_signature =
        get_field_signature(&module, &module.field_handles[field_handle_idx.0 as usize]).clone();

    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::MutableReference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    let (state2, _) = common::run_instruction(Bytecode::MutBorrowField(field_handle_idx), state1);
    let kind = kind(&state2.module.module, &field_signature, &[]);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue {
            token: SignatureToken::MutableReference(Box::new(field_signature)),
            kind,
        }),
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_mutborrowfield_stack_has_no_reference() {
    let mut module: CompiledModuleMut = generate_module_with_struct(false);
    let struct_def_idx = StructDefinitionIndex((module.struct_defs.len() - 1) as u16);
    module.field_handles.push(FieldHandle {
        owner: struct_def_idx,
        field: 0,
    });
    let field_handle_idx = FieldHandleIndex((module.field_handles.len() - 1) as u16);

    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(Bytecode::MutBorrowField(field_handle_idx), state1);
}

#[test]
#[should_panic]
fn bytecode_mutborrowfield_ref_is_immutable() {
    let mut module: CompiledModuleMut = generate_module_with_struct(false);
    let struct_def_idx = StructDefinitionIndex((module.struct_defs.len() - 1) as u16);
    module.field_handles.push(FieldHandle {
        owner: struct_def_idx,
        field: 0,
    });
    let field_handle_idx = FieldHandleIndex((module.field_handles.len() - 1) as u16);

    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::Reference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    common::run_instruction(Bytecode::MutBorrowField(field_handle_idx), state1);
}

#[test]
fn bytecode_immborrowfield() {
    let mut module: CompiledModuleMut = generate_module_with_struct(false);
    let struct_def_idx = StructDefinitionIndex((module.struct_defs.len() - 1) as u16);
    module.field_handles.push(FieldHandle {
        owner: struct_def_idx,
        field: 0,
    });
    let field_handle_idx = FieldHandleIndex((module.field_handles.len() - 1) as u16);
    let field_signature =
        get_field_signature(&module, &module.field_handles[field_handle_idx.0 as usize]).clone();

    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::Reference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    let (state2, _) = common::run_instruction(Bytecode::ImmBorrowField(field_handle_idx), state1);
    let kind = kind(&state2.module.module, &field_signature, &[]);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue {
            token: SignatureToken::MutableReference(Box::new(field_signature)),
            kind,
        }),
        "stack type postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_immborrowfield_stack_has_no_reference() {
    let mut module: CompiledModuleMut = generate_module_with_struct(false);
    let struct_def_idx = StructDefinitionIndex((module.struct_defs.len() - 1) as u16);
    module.field_handles.push(FieldHandle {
        owner: struct_def_idx,
        field: 0,
    });
    let field_handle_idx = FieldHandleIndex((module.field_handles.len() - 1) as u16);

    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(Bytecode::ImmBorrowField(field_handle_idx), state1);
}

#[test]
#[should_panic]
fn bytecode_immborrowfield_ref_is_mutable() {
    let mut module: CompiledModuleMut = generate_module_with_struct(false);
    let struct_def_idx = StructDefinitionIndex((module.struct_defs.len() - 1) as u16);
    module.field_handles.push(FieldHandle {
        owner: struct_def_idx,
        field: 0,
    });
    let field_handle_idx = FieldHandleIndex((module.field_handles.len() - 1) as u16);

    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue {
        token: SignatureToken::MutableReference(Box::new(struct_value.token)),
        kind: struct_value.kind,
    });
    common::run_instruction(Bytecode::ImmBorrowField(field_handle_idx), state1);
}

#[test]
fn bytecode_borrowglobal() {
    let module: CompiledModuleMut = generate_module_with_struct(true);
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    let struct_value = create_struct_value(&state1.module.module).0;
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Address));
    let (state2, _) = common::run_instruction(
        Bytecode::MutBorrowGlobal(StructDefinitionIndex::new(0)),
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
        Bytecode::MutBorrowGlobal(StructDefinitionIndex::new(0)),
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
        Bytecode::MutBorrowGlobal(StructDefinitionIndex::new(0)),
        state1,
    );
}
