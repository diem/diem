// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{context::*, labels_to_offsets, remove_fallthrough_jumps};
use crate::shared::unique_map::UniqueMap;
use crate::{
    cfgir::ast as G,
    errors::*,
    naming::ast::{BuiltinTypeName_, TParam, TParamID, TypeName_},
    parser::ast::{
        BinOp, BinOp_, Field, FunctionName, FunctionVisibility, Kind, Kind_, ModuleIdent,
        ModuleIdent_, ModuleName, StructName, UnaryOp, UnaryOp_, Value_, Var,
    },
    shared::*,
};
use move_vm::file_format as F;
use std::{collections::HashMap, convert::TryInto};

//**************************************************************************************************
// Function Frame
//**************************************************************************************************

// Holds information about a function being compiled.
#[derive(Debug)]
struct FunctionFrame {
    locals: HashMap<Var, u8>,
    max_stack_depth: u16,
    cur_stack_depth: u16,
}

impl FunctionFrame {
    fn new(loc: Loc, locals_iter: impl Iterator<Item = Var>) -> Result<FunctionFrame> {
        let mut locals = HashMap::new();

        for (idx, local) in locals_iter.enumerate() {
            if idx >= (F::LocalIndex::max_value() as usize) {
                bail!(
                    loc,
                    "Max number of locals reached. The Move VM only supports up to u8::max_value \
                     locals"
                );
            }
            assert!(locals.insert(local, idx as F::LocalIndex).is_none());
        }
        Ok(Self {
            locals,
            max_stack_depth: 0,
            cur_stack_depth: 0,
        })
    }

    // Manage the stack info for the function
    fn push(&mut self, loc: Loc) -> Result<()> {
        self.pushn(loc, 1)
    }

    fn pushn(&mut self, loc: Loc, n: usize) -> Result<()> {
        let n: u16 = n.try_into().unwrap();
        if self.cur_stack_depth >= u16::max_value() - n {
            bail!(
                loc,
                "ICE Stack depth accounting overflow. The Move VM can only support a maximum stack \
                depth of up to u16::max_value"
            )
        }
        self.cur_stack_depth += n;
        self.max_stack_depth = std::cmp::max(self.max_stack_depth, self.cur_stack_depth);
        Ok(())
    }

    fn pop(&mut self) {
        self.popn(1)
    }

    fn popn(&mut self, n: usize) {
        let n = n.try_into().unwrap();
        if self.cur_stack_depth < n {
            panic!("ICE Stack depth accounting underflow. Type checking failed")
        }
        self.cur_stack_depth -= n;
    }

    fn local(&self, var: &Var) -> u8 {
        *self.locals.get(var).unwrap()
    }
}

//**************************************************************************************************
// Compiled Unit
//**************************************************************************************************

#[derive(Debug)]
pub enum CompiledUnit {
    Module(ModuleName, F::CompiledModule),
    Script(Loc, F::CompiledScript),
}

impl CompiledUnit {
    pub fn name(&self) -> String {
        match self {
            CompiledUnit::Module(m, _) => format!("module_{}", m),
            CompiledUnit::Script(_, _) => "script".into(),
        }
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut serialized = Vec::<u8>::new();
        match self {
            CompiledUnit::Module(_, m) => m.serialize(&mut serialized).unwrap(),
            CompiledUnit::Script(_, s) => s.serialize(&mut serialized).unwrap(),
        };
        serialized
    }

    #[allow(dead_code)]
    pub fn serialize_debug(self) -> Vec<u8> {
        match self {
            CompiledUnit::Module(_, m) => format!("{}", m),
            CompiledUnit::Script(_, s) => format!("{}", s),
        }
        .into()
    }

    pub fn verify(self) -> (Self, Errors) {
        match self {
            CompiledUnit::Module(n, m) => {
                let (m, errors) = verify_module(n.loc(), m);
                (CompiledUnit::Module(n, m), errors)
            }
            CompiledUnit::Script(loc, s) => {
                let (s, errors) = verify_script(loc, s);
                (CompiledUnit::Script(loc, s), errors)
            }
        }
    }
}

fn verify_module(loc: Loc, cm: F::CompiledModule) -> (F::CompiledModule, Errors) {
    match move_bytecode_verifier::verifier::VerifiedModule::new(cm) {
        Ok(v) => (v.into_inner(), vec![]),
        Err((cm, es)) => (
            cm,
            vec![vec![(
                loc,
                format!("ICE failed bytecode verifier: {:#?}", es),
            )]],
        ),
    }
}

fn verify_script(loc: Loc, cs: F::CompiledScript) -> (F::CompiledScript, Errors) {
    match move_bytecode_verifier::verifier::VerifiedScript::new(cs) {
        Ok(v) => (v.into_inner(), vec![]),
        Err((cs, es)) => (
            cs,
            vec![vec![(
                loc,
                format!("ICE failed bytecode verifier: {:#?}", es),
            )]],
        ),
    }
}

pub fn verify_units(units: Vec<CompiledUnit>) -> (Vec<CompiledUnit>, Errors) {
    let mut new_units = vec![];
    let mut errors = vec![];
    for unit in units {
        let (unit, mut es) = unit.verify();
        new_units.push(unit);
        errors.append(&mut es);
    }
    (new_units, errors)
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: G::Program) -> std::result::Result<Vec<CompiledUnit>, Errors> {
    let mut units = vec![];
    let mut errors = vec![];

    let sdecls = prog
        .modules
        .iter()
        .flat_map(|(m, mdef)| {
            mdef.structs.iter().map(move |(s, sdef)| {
                let key = (m.clone(), s);
                let is_nominal_resource = sdef.resource_opt.is_some();
                let kinds = kinds(sdef.type_parameters.iter().map(|tp| &tp.kind));
                (key, (is_nominal_resource, kinds))
            })
        })
        .collect();
    let fdecls = prog
        .modules
        .iter()
        .flat_map(|(m, mdef)| {
            mdef.functions.iter().map(move |(f, fdef)| {
                let key = (m.clone(), f);
                (key, fdef.signature.clone())
            })
        })
        .collect();

    let mut source_modules = prog
        .modules
        .into_iter()
        .filter(|(_, mdef)| mdef.is_source_module.is_some())
        .collect::<Vec<_>>();
    source_modules.sort_by_key(|(_, mdef)| mdef.is_source_module.unwrap());
    for (m, mdef) in source_modules {
        match module(m, mdef, &sdecls, &fdecls) {
            Ok((n, cm)) => units.push(CompiledUnit::Module(n, cm)),
            Err(err) => errors.push(err),
        }
    }
    if let Some((addr, n, fdef)) = prog.main {
        match main(addr, n.clone(), fdef, &sdecls, &fdecls) {
            Ok(cs) => units.push(CompiledUnit::Script(n.loc(), cs)),
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(units)
    } else {
        Err(errors)
    }
}

fn module(
    ident: ModuleIdent,
    mdef: G::ModuleDefinition,
    struct_declarations: &HashMap<(ModuleIdent, StructName), (bool, Vec<F::Kind>)>,
    function_declarations: &HashMap<(ModuleIdent, FunctionName), G::FunctionSignature>,
) -> Result<(ModuleName, F::CompiledModule)> {
    let mut context = Context::new(&ident, struct_declarations, function_declarations)?;

    let mut struct_defs = vec![];
    let mut field_defs = vec![];
    for (s, sdef) in mdef.structs.into_iter() {
        struct_def(
            &mut context,
            &mut struct_defs,
            &mut field_defs,
            &ident,
            s,
            sdef,
        )?;
    }

    let function_defs = mdef
        .functions
        .into_iter()
        .map(|(f, fdef)| function(&mut context, &ident, f, fdef))
        .collect::<Result<Vec<_>>>()?;

    let MaterializedPools {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        identifiers,
        byte_array_pool,
        address_pool,
    } = context.materialize_pools();
    let compiled_module_mut = F::CompiledModuleMut {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        identifiers,
        byte_array_pool,
        address_pool,
        struct_defs,
        field_defs,
        function_defs,
    };
    let compiled_module = compiled_module_mut
        .freeze()
        .map_err(|vm_status| vec![(ident.loc(), format!("VM ERROR: {:#?}", vm_status))])?;
    Ok((ident.0.value.name.clone(), compiled_module))
}

fn main(
    address: Address,
    main_name: FunctionName,
    fdef: G::Function,
    struct_declarations: &HashMap<(ModuleIdent, StructName), (bool, Vec<F::Kind>)>,
    function_declarations: &HashMap<(ModuleIdent, FunctionName), G::FunctionSignature>,
) -> Result<F::CompiledScript> {
    let loc = main_name.loc();
    let current_module_ = ModuleIdent_ {
        address,
        name: ModuleName(sp(loc, F::self_module_name().to_owned().to_string())),
    };
    let current_module = ModuleIdent(sp(loc, current_module_));
    let mut context = Context::new(&current_module, struct_declarations, function_declarations)?;

    let main = function(&mut context, &current_module, main_name, fdef)?;

    let MaterializedPools {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        identifiers,
        byte_array_pool,
        address_pool,
    } = context.materialize_pools();
    let compiled_script = F::CompiledScriptMut {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        identifiers,
        byte_array_pool,
        address_pool,
        main,
    };
    compiled_script
        .freeze()
        .map_err(|vm_status| vec![(loc, format!("VM ERROR: {:#?}", vm_status))])
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(
    context: &mut Context,
    struct_defs: &mut Vec<F::StructDefinition>,
    field_defs: &mut Vec<F::FieldDefinition>,
    m: &ModuleIdent,
    s: StructName,
    sdef: G::StructDefinition,
) -> Result<()> {
    let struct_handle = context.struct_handle_index(m, &s)?;
    let s_loc = s.loc();
    bind_type_parameters(context, s_loc, &sdef.type_parameters)?;
    let idx = context.declare_struct_definition_index(s)?;
    assert!(idx.0 as usize == struct_defs.len());
    let field_information = fields(s_loc, context, field_defs, struct_handle, idx, sdef.fields)?;
    struct_defs.push(F::StructDefinition {
        struct_handle,
        field_information,
    });
    Ok(())
}

fn fields(
    loc: Loc,
    context: &mut Context,
    field_defs: &mut Vec<F::FieldDefinition>,
    struct_handle: F::StructHandleIndex,
    struct_def_idx: F::StructDefinitionIndex,
    gfields: G::StructFields,
) -> Result<F::StructFieldInformation> {
    use F::StructFieldInformation as FF;
    use G::StructFields as GF;
    Ok(match gfields {
        GF::Native(_) => FF::Native,
        GF::Defined(field_vec) if field_vec.is_empty() => {
            // empty fields are not allowed in the bytecode, add a dummy field
            let fake_field = vec![(
                Field(sp(loc, "dummy_field".to_string())),
                G::BaseType_::bool(loc),
            )];
            fields(
                loc,
                context,
                field_defs,
                struct_handle,
                struct_def_idx,
                GF::Defined(fake_field),
            )?
        }

        GF::Defined(field_vec) => {
            let pool_len = field_defs.len();
            let field_count = field_vec.len();

            let field_information = FF::Declared {
                field_count: (field_count as F::MemberCount),
                fields: F::FieldDefinitionIndex(pool_len as F::TableIndex),
            };

            for (field, field_ty) in field_vec {
                let name = context.identifier_index(&field)?;
                let tloc = field_ty.loc;
                let st = base_type(context, field_ty)?;
                let signature = context.type_signature_index(tloc, st)?;
                let idx = context.declare_field(struct_def_idx, field)?;
                assert!(idx.0 as usize == field_defs.len());
                field_defs.push(F::FieldDefinition {
                    struct_: struct_handle,
                    name,
                    signature,
                });
            }

            field_information
        }
    })
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(
    context: &mut Context,
    m: &ModuleIdent,
    f: FunctionName,
    fdef: G::Function,
) -> Result<F::FunctionDefinition> {
    let loc = f.loc();
    bind_type_parameters(context, f.loc(), &fdef.signature.type_parameters)?;
    let parameters = fdef.signature.parameters.clone();
    let idx = match context.function_handle_index(m, &f) {
        Some(idx) => idx,
        None => {
            let signature = function_signature(context, fdef.signature)?;
            context.declare_function(m.clone(), f, signature)?
        }
    };

    let flags = match fdef.visibility {
        FunctionVisibility::Internal => 0,
        FunctionVisibility::Public(_) => F::CodeUnit::PUBLIC,
    } | match &fdef.body.value {
        G::FunctionBody_::Defined { .. } => 0,
        G::FunctionBody_::Native => F::CodeUnit::NATIVE,
    };
    let acquires_global_resources = fdef
        .acquires
        .into_iter()
        .map(|s| context.struct_definition_index(&s))
        .collect();

    let code = match fdef.body.value {
        G::FunctionBody_::Defined {
            locals,
            start,
            blocks,
        } => function_body(context, loc, parameters, locals, start, blocks)?,
        G::FunctionBody_::Native => {
            // TODO this is all sorts of wrong. The file format needs a native code unit
            F::CodeUnit::default()
        }
    };

    Ok(F::FunctionDefinition {
        function: idx,
        flags,
        acquires_global_resources,
        code,
    })
}

fn function_signature(
    context: &mut Context,
    sig: G::FunctionSignature,
) -> Result<F::FunctionSignature> {
    let return_types = types(context, sig.return_type)?;
    let arg_types = sig
        .parameters
        .into_iter()
        .map(|(_, st)| single_type(context, st))
        .collect::<Result<_>>()?;
    let type_formals = kinds(sig.type_parameters.iter().map(|tp| &tp.kind));
    Ok(F::FunctionSignature {
        return_types,
        arg_types,
        type_formals,
    })
}

type Code = Vec<F::Bytecode>;

fn function_body(
    context: &mut Context,
    loc: Loc,
    mut parameters: Vec<(Var, G::SingleType)>,
    mut locals_map: UniqueMap<Var, G::SingleType>,
    start: G::Label,
    blocks: G::Blocks,
) -> Result<F::CodeUnit> {
    parameters
        .iter()
        .for_each(|(var, _)| assert!(locals_map.remove(var).is_some()));
    locals_map
        .into_iter()
        .for_each(|v_ty| parameters.push(v_ty));

    let (local_vars, local_types): (Vec<_>, Vec<_>) = parameters.into_iter().unzip();
    let mut ff = FunctionFrame::new(loc, local_vars.into_iter())?;

    let locals_signature = F::LocalsSignature(
        local_types
            .into_iter()
            .map(|st| single_type(context, st))
            .collect::<Result<_>>()?,
    );
    let locals_idx = context.locals_signature_index(loc, locals_signature)?;

    let mut code_blocks = Vec::new();
    let mut label_map = HashMap::<u16, u16>::new();
    for (idx, (label, basic_block)) in blocks.into_iter().enumerate() {
        // first idx should be the start label
        assert!(idx != 0 || label == start);
        assert!(idx == code_blocks.len());
        assert!(label_map
            .insert(label.0.try_into().unwrap(), idx.try_into().unwrap())
            .is_none());

        let mut code = Code::new();
        for cmd in basic_block {
            command(context, &mut ff, &mut code, cmd)?;
        }
        code_blocks.push(code);
    }

    super::remap_offsets(&mut code_blocks, &label_map);
    remove_fallthrough_jumps::code(&mut code_blocks);
    let code = labels_to_offsets::code(code_blocks);

    Ok(F::CodeUnit {
        max_stack_size: ff.max_stack_depth,
        locals: locals_idx,
        code,
    })
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn kinds<'a>(ks: impl Iterator<Item = &'a Kind>) -> Vec<F::Kind> {
    ks.map(kind).collect()
}

fn kind(sp!(_, k_): &Kind) -> F::Kind {
    use Kind_ as GK;
    use F::Kind as FK;
    match k_ {
        GK::Unknown => FK::All,
        GK::Resource => FK::Resource,
        GK::Affine | GK::Unrestricted => FK::Unrestricted,
    }
}

fn bind_type_parameters(
    context: &mut Context,
    loc: Loc,
    tps: &[TParam],
) -> Result<HashMap<TParamID, F::TableIndex>> {
    let mut m = HashMap::new();
    for (idx, tparam) in tps.iter().enumerate() {
        assert!(m.insert(tparam.id, idx).is_none());
    }
    context.bind_type_parameters(loc, m)
}

fn struct_definition_indices_base(
    context: &mut Context,
    sp!(loc, b_): G::BaseType,
) -> Result<(F::StructDefinitionIndex, F::LocalsSignatureIndex)> {
    use TypeName_ as TN;
    use G::BaseType_ as B;
    match b_ {
        B::Apply(_, sp!(_, TN::ModuleType(m, s)), tys) => {
            assert!(context.module_handle_index(&m).unwrap().0 == 0);
            let def_idx = context.struct_definition_index(&s);
            let local_sig = F::LocalsSignature(base_types(context, tys)?);
            let tys_idx = context.locals_signature_index(loc, local_sig)?;
            Ok((def_idx, tys_idx))
        }
        _ => panic!("ICE expected struct for struct definition index"),
    }
}

fn struct_definition_index_base(
    context: &mut Context,
    sp!(_, b_): &G::BaseType,
) -> F::StructDefinitionIndex {
    use TypeName_ as TN;
    use G::BaseType_ as B;
    match b_ {
        B::Apply(_, sp!(_, TN::ModuleType(m, s)), _) => {
            assert!(context.module_handle_index(&m).unwrap().0 == 0);
            context.struct_definition_index(&s)
        }
        _ => panic!("ICE expected struct for struct definition index"),
    }
}

fn struct_definition_index_single(
    context: &mut Context,
    sp!(_, s_): &G::SingleType,
) -> F::StructDefinitionIndex {
    match s_ {
        G::SingleType_::Base(b) | G::SingleType_::Ref(_, b) => {
            struct_definition_index_base(context, b)
        }
    }
}

fn struct_definition_index(
    context: &mut Context,
    sp!(_, t_): &G::Type,
) -> F::StructDefinitionIndex {
    match t_ {
        G::Type_::Single(s) => struct_definition_index_single(context, s),
        _ => panic!("ICE expected struct for struct definition index"),
    }
}

fn base_types(context: &mut Context, bs: Vec<G::BaseType>) -> Result<Vec<F::SignatureToken>> {
    bs.into_iter().map(|b| base_type(context, b)).collect()
}

fn base_type(context: &mut Context, sp!(_, bt_): G::BaseType) -> Result<F::SignatureToken> {
    use BuiltinTypeName_ as BT;
    use TypeName_ as TN;
    use F::SignatureToken as ST;
    use G::BaseType_ as B;
    Ok(match bt_ {
        B::UnresolvedError => panic!("ICE should not have reached compilation if there are errors"),
        B::Apply(_, sp!(_, TN::Builtin(sp!(_, BT::Address))), _) => ST::Address,
        B::Apply(_, sp!(_, TN::Builtin(sp!(_, BT::U64))), _) => ST::U64,
        B::Apply(_, sp!(_, TN::Builtin(sp!(_, BT::Bool))), _) => ST::Bool,
        B::Apply(_, sp!(_, TN::Builtin(sp!(_, BT::Bytearray))), _) => ST::ByteArray,
        B::Apply(_, sp!(_, TN::ModuleType(m, s)), tys) => {
            let idx = context.struct_handle_index(&m, &s)?;
            let tokens = base_types(context, tys)?;
            ST::Struct(idx, tokens)
        }
        B::Param(TParam { id, .. }) => ST::TypeParameter(context.type_formal_index(id)),
    })
}

fn single_type(context: &mut Context, sp!(_, st_): G::SingleType) -> Result<F::SignatureToken> {
    use F::SignatureToken as ST;
    use G::SingleType_ as S;
    match st_ {
        S::Base(bt) => base_type(context, bt),
        S::Ref(false, bt) => Ok(ST::Reference(Box::new(base_type(context, bt)?))),
        S::Ref(true, bt) => Ok(ST::MutableReference(Box::new(base_type(context, bt)?))),
    }
}

fn types(context: &mut Context, sp!(_, t_): G::Type) -> Result<Vec<F::SignatureToken>> {
    use G::Type_ as T;
    match t_ {
        T::Unit => Ok(vec![]),
        T::Single(st) => Ok(vec![single_type(context, st)?]),
        T::Multiple(ss) => ss.into_iter().map(|st| single_type(context, st)).collect(),
    }
}

fn type_stack_size(sp!(_, t_): &G::Type) -> usize {
    use G::Type_ as T;
    match t_ {
        T::Unit => 0,
        T::Single(_) => 1,
        T::Multiple(ss) => ss.len(),
    }
}

//**************************************************************************************************
// Commands
//**************************************************************************************************

fn command(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Code,
    sp!(_, cmd_): G::Command,
) -> Result<()> {
    use F::Bytecode as B;
    use G::Command_ as C;
    match cmd_ {
        C::Assign(ls, e) => {
            exp(context, function_frame, code, e)?;
            lvalues(context, function_frame, code, ls)?;
        }
        C::Mutate(eref, ervalue) => {
            exp(context, function_frame, code, ervalue)?;
            exp(context, function_frame, code, eref)?;
            code.push(B::WriteRef);
            function_frame.popn(2);
        }
        C::Abort(ecode) => {
            exp_(context, function_frame, code, ecode)?;
            code.push(B::Abort);
            function_frame.pop();
        }
        C::Return(e) => {
            let n = type_stack_size(&e.ty);
            exp_(context, function_frame, code, e)?;
            code.push(B::Ret);
            function_frame.popn(n);
        }
        C::IgnoreAndPop { pop_num, exp: e } => {
            exp_(context, function_frame, code, e)?;
            for _ in 0..pop_num {
                code.push(B::Pop);
            }
            function_frame.popn(pop_num);
        }
        C::Jump(G::Label(lbl)) => {
            code.push(B::Branch(lbl.try_into().unwrap()));
        }
        C::JumpIf {
            cond,
            if_true: G::Label(if_true),
            if_false: G::Label(if_false),
        } => {
            exp_(context, function_frame, code, cond)?;
            code.push(B::BrTrue(if_true.try_into().unwrap()));
            function_frame.pop();
            code.push(B::Branch(if_false.try_into().unwrap()));
        }
    }
    Ok(())
}

fn lvalues(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Code,
    ls: Vec<G::LValue>,
) -> Result<()> {
    lvalues_(context, function_frame, code, ls.into_iter())
}

fn lvalues_(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Code,
    ls: impl std::iter::DoubleEndedIterator<Item = G::LValue>,
) -> Result<()> {
    for l in ls.rev() {
        lvalue(context, function_frame, code, l)?
    }
    Ok(())
}

fn lvalue(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Code,
    sp!(loc, l_): G::LValue,
) -> Result<()> {
    use F::Bytecode as B;
    use G::LValue_ as L;
    match l_ {
        L::Ignore => {
            code.push(B::Pop);
            function_frame.pop();
        }
        L::Var(v, _) => {
            let loc_idx = function_frame.local(&v);
            code.push(B::StLoc(loc_idx));
            function_frame.pop();
        }
        L::Unpack(s, tys, field_ls) if field_ls.is_empty() => {
            // empty fields are not allowed in the bytecode, add a dummy field
            // empty structs have a dummy field of type 'bool' added

            let def_idx = context.struct_definition_index(&s);
            let local_sig = F::LocalsSignature(base_types(context, tys)?);
            let local_idx = context.locals_signature_index(loc, local_sig)?;

            function_frame.pop();
            code.push(B::Unpack(def_idx, local_idx));
            // Pop off false
            function_frame.pushn(loc, 1)?;
            code.push(B::Pop);
            function_frame.pop();
        }

        L::Unpack(s, tys, field_ls) => {
            let def_idx = context.struct_definition_index(&s);
            let local_sig = F::LocalsSignature(base_types(context, tys)?);
            let local_idx = context.locals_signature_index(loc, local_sig)?;

            function_frame.pop();
            code.push(B::Unpack(def_idx, local_idx));
            function_frame.pushn(loc, field_ls.len())?;

            lvalues_(
                context,
                function_frame,
                code,
                field_ls.into_iter().map(|(_, l)| l),
            )?;
        }
    }
    Ok(())
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn exp(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Code,
    e: Box<G::Exp>,
) -> Result<()> {
    exp_(context, function_frame, code, *e)
}

fn exp_(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Code,
    e: G::Exp,
) -> Result<()> {
    use Value_ as V;
    use F::Bytecode as B;
    use G::UnannotatedExp_ as E;
    let ty = e.ty;
    let sp!(eloc, e_) = e.exp;
    match e_ {
        E::Unreachable => panic!("ICE should not compile dead code"),
        E::UnresolvedError => panic!("ICE should not have reached compilation if there are errors"),
        E::Unit => (),
        E::Value(v) => {
            code.push(match v.value {
                V::Address(a) => {
                    let idx = context.address_index(eloc, a)?;
                    B::LdAddr(idx)
                }
                V::Bytearray(bytes) => {
                    let idx = context.byte_array_index(eloc, bytes)?;
                    B::LdByteArray(idx)
                }
                V::U64(u) => B::LdU64(u),
                V::Bool(b) => {
                    if b {
                        B::LdTrue
                    } else {
                        B::LdFalse
                    }
                }
            });
            function_frame.push(eloc)?;
        }
        E::Move { var, .. } => {
            let loc_idx = function_frame.local(&var);
            code.push(B::MoveLoc(loc_idx));
            function_frame.push(eloc)?;
        }
        E::Copy { var, .. } => {
            let loc_idx = function_frame.local(&var);
            code.push(B::CopyLoc(loc_idx));
            function_frame.push(eloc)?;
        }

        E::ModuleCall(mcall) => {
            let num_args = type_stack_size(&mcall.arguments.ty);
            exp(context, function_frame, code, mcall.arguments)?;

            function_frame.popn(num_args);
            call(
                context,
                code,
                mcall.module,
                mcall.name,
                mcall.type_arguments,
            )?;
            function_frame.pushn(eloc, type_stack_size(&ty))?;
        }

        E::Builtin(b, arg) => {
            let num_args = type_stack_size(&arg.ty);
            exp(context, function_frame, code, arg)?;

            function_frame.popn(num_args);
            builtin(context, code, *b)?;
            function_frame.pushn(eloc, type_stack_size(&ty))?;
        }

        E::Freeze(er) => {
            exp(context, function_frame, code, er)?;

            function_frame.pop();
            code.push(B::FreezeRef);
            function_frame.push(eloc)?;
        }

        E::Dereference(er) => {
            exp(context, function_frame, code, er)?;

            function_frame.pop();
            code.push(B::ReadRef);
            function_frame.push(eloc)?;
        }

        E::UnaryExp(op, er) => {
            exp(context, function_frame, code, er)?;

            unary_op(code, op);
            function_frame.push(eloc)?;
        }

        E::BinopExp(el, op, er) => {
            exp(context, function_frame, code, el)?;
            exp(context, function_frame, code, er)?;

            function_frame.popn(2);
            binary_op(code, op);
            function_frame.push(eloc)?;
        }

        E::Pack(s, tys, field_args) if field_args.is_empty() => {
            // empty fields are not allowed in the bytecode, add a dummy field
            // empty structs have a dummy field of type 'bool' added

            let def_idx = context.struct_definition_index(&s);
            let local_sig = F::LocalsSignature(base_types(context, tys)?);
            let local_idx = context.locals_signature_index(eloc, local_sig)?;

            // Push on fake field
            code.push(B::LdFalse);
            function_frame.push(eloc)?;

            function_frame.popn(1);
            code.push(B::Pack(def_idx, local_idx));
            function_frame.push(eloc)?;
        }

        E::Pack(s, tys, field_args) => {
            let num_args = field_args.len();
            for (_, _, earg) in field_args {
                exp_(context, function_frame, code, earg)?;
            }
            let def_idx = context.struct_definition_index(&s);
            let local_sig = F::LocalsSignature(base_types(context, tys)?);
            let local_idx = context.locals_signature_index(eloc, local_sig)?;

            function_frame.popn(num_args);
            code.push(B::Pack(def_idx, local_idx));
            function_frame.push(eloc)?;
        }

        E::ExpList(items) => {
            for item in items {
                let ei = match item {
                    G::ExpListItem::Single(ei, _) | G::ExpListItem::Splat(_, ei, _) => ei,
                };
                exp_(context, function_frame, code, ei)?;
            }
        }

        E::Borrow(mut_, el, field) => {
            let sdef_idx = struct_definition_index(context, &el.ty);
            exp(context, function_frame, code, el)?;
            let f_idx = context.field(sdef_idx, field);

            function_frame.pop();
            code.push(if mut_ {
                B::MutBorrowField(f_idx)
            } else {
                B::ImmBorrowField(f_idx)
            });
            function_frame.push(eloc)?;
        }

        E::BorrowLocal(mut_, v) => {
            let l_idx = function_frame.local(&v);
            code.push(if mut_ {
                B::MutBorrowLoc(l_idx)
            } else {
                B::ImmBorrowLoc(l_idx)
            });
            function_frame.push(eloc)?;
        }
    }
    Ok(())
}

fn call(
    context: &mut Context,
    code: &mut Code,
    m: ModuleIdent,
    f: FunctionName,
    tys: Vec<G::BaseType>,
) -> Result<()> {
    use crate::shared::fake_natives::transaction as TXN;
    use Address as A;
    use F::Bytecode as B;

    match (&m.0.value.address, m.0.value.name.value(), f.value()) {
        (&A::LIBRA_CORE, TXN::MOD, TXN::GAS_PRICE) => code.push(B::GetTxnGasUnitPrice),
        (&A::LIBRA_CORE, TXN::MOD, TXN::MAX_GAS) => code.push(B::GetTxnMaxGasUnits),
        (&A::LIBRA_CORE, TXN::MOD, TXN::GAS_REMAINING) => code.push(B::GetGasRemaining),
        (&A::LIBRA_CORE, TXN::MOD, TXN::SENDER) => code.push(B::GetTxnSenderAddress),
        (&A::LIBRA_CORE, TXN::MOD, TXN::SEQUENCE_NUM) => code.push(B::GetTxnSequenceNumber),
        (&A::LIBRA_CORE, TXN::MOD, TXN::PUBLIC_KEY) => code.push(B::GetTxnPublicKey),
        (&A::LIBRA_CORE, TXN::MOD, TXN::ASSERT) => panic!("ICE should have been covered in hlir"),
        (&A::LIBRA_CORE, TXN::MOD, f) => panic!("ICE unknown magic transaction function {}", f),
        _ => module_call(context, code, m, f, tys)?,
    }
    Ok(())
}

fn module_call(
    context: &mut Context,
    code: &mut Code,
    m: ModuleIdent,
    f: FunctionName,
    tys: Vec<G::BaseType>,
) -> Result<()> {
    use F::Bytecode as B;
    let loc = f.loc();
    let function_handle_index = match context.function_handle_index(&m, &f) {
        Some(i) => i,
        None => {
            let declaration_sig = context.declaration_function_signature(&m, &f);

            let old_type_parameters =
                bind_type_parameters(context, loc, &declaration_sig.type_parameters)?;
            let sig = function_signature(context, declaration_sig)?;
            let old_map = old_type_parameters
                .into_iter()
                .map(|(id, idx)| (id, idx as usize))
                .collect();
            context.bind_type_parameters(loc, old_map)?;

            context.declare_function(m, f, sig)?
        }
    };
    let tokens = F::LocalsSignature(base_types(context, tys)?);
    let type_actuals_id = context.locals_signature_index(loc, tokens)?;
    code.push(B::Call(function_handle_index, type_actuals_id));
    Ok(())
}

fn builtin(context: &mut Context, code: &mut Code, sp!(_, b_): G::BuiltinFunction) -> Result<()> {
    use F::Bytecode as B;
    use G::BuiltinFunction_ as GB;
    code.push(match b_ {
        GB::MoveToSender(bt) => {
            let (def_idx, tys_idx) = struct_definition_indices_base(context, bt)?;
            B::MoveToSender(def_idx, tys_idx)
        }
        GB::MoveFrom(bt) => {
            let (def_idx, tys_idx) = struct_definition_indices_base(context, bt)?;
            B::MoveFrom(def_idx, tys_idx)
        }
        GB::BorrowGlobal(false, bt) => {
            let (def_idx, tys_idx) = struct_definition_indices_base(context, bt)?;
            B::ImmBorrowGlobal(def_idx, tys_idx)
        }
        GB::BorrowGlobal(true, bt) => {
            let (def_idx, tys_idx) = struct_definition_indices_base(context, bt)?;
            B::MutBorrowGlobal(def_idx, tys_idx)
        }
        GB::Exists(bt) => {
            let (def_idx, tys_idx) = struct_definition_indices_base(context, bt)?;
            B::Exists(def_idx, tys_idx)
        }
    });
    Ok(())
}

fn unary_op(code: &mut Code, sp!(_, op_): UnaryOp) {
    use UnaryOp_ as O;
    use F::Bytecode as B;
    code.push(match op_ {
        O::Neg => panic!("ICE not yet supported for any type"),
        O::Not => B::Not,
    });
}

fn binary_op(code: &mut Code, sp!(_, op_): BinOp) {
    use BinOp_ as O;
    use F::Bytecode as B;
    code.push(match op_ {
        O::Add => B::Add,
        O::Sub => B::Sub,
        O::Mul => B::Mul,
        O::Mod => B::Mod,
        O::Div => B::Div,
        O::BitOr => B::BitOr,
        O::BitAnd => B::BitAnd,
        O::Xor => B::Xor,

        O::And => B::And,
        O::Or => B::Or,

        O::Eq => B::Eq,
        O::Neq => B::Neq,

        O::Lt => B::Lt,
        O::Gt => B::Gt,

        O::Le => B::Le,
        O::Ge => B::Ge,
    });
}
