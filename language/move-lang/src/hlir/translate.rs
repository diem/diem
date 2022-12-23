// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diag,
    expansion::ast::{AbilitySet, Fields, ModuleIdent, Value_},
    hlir::ast::{self as H, Block},
    naming::ast as N,
    parser::ast::{BinOp_, ConstantName, Field, FunctionName, StructName, Var},
    shared::{unique_map::UniqueMap, *},
    typing::ast as T,
    FullyCompiledProgram,
};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// Vars
//**************************************************************************************************

const NEW_NAME_DELIM: &str = "#";

fn new_name(context: &mut Context, n: Symbol) -> Symbol {
    format!("{}{}{}", n, NEW_NAME_DELIM, context.counter_next()).into()
}

const TEMP_PREFIX: &str = "%";
static TEMP_PREFIX_SYMBOL: Lazy<Symbol> = Lazy::new(|| TEMP_PREFIX.into());

fn new_temp_name(context: &mut Context) -> Symbol {
    new_name(context, *TEMP_PREFIX_SYMBOL)
}

pub fn is_temp_name(s: Symbol) -> bool {
    s.starts_with(TEMP_PREFIX)
}

pub enum DisplayVar {
    Orig(String),
    Tmp,
}

pub fn display_var(s: Symbol) -> DisplayVar {
    if is_temp_name(s) {
        DisplayVar::Tmp
    } else {
        let mut orig = s.as_str().to_string();
        orig.truncate(orig.find('#').unwrap_or_else(|| s.len()));
        DisplayVar::Orig(orig)
    }
}

//**************************************************************************************************
// Context
//**************************************************************************************************

struct Context<'env> {
    env: &'env mut CompilationEnv,
    structs: UniqueMap<StructName, UniqueMap<Field, usize>>,
    function_locals: UniqueMap<Var, H::SingleType>,
    local_scope: UniqueMap<Var, Var>,
    used_locals: BTreeSet<Var>,
    signature: Option<H::FunctionSignature>,
    tmp_counter: usize,
}

impl<'env> Context<'env> {
    pub fn new(env: &'env mut CompilationEnv) -> Self {
        Context {
            env,
            structs: UniqueMap::new(),
            function_locals: UniqueMap::new(),
            local_scope: UniqueMap::new(),
            used_locals: BTreeSet::new(),
            signature: None,
            tmp_counter: 0,
        }
    }

    pub fn has_empty_locals(&self) -> bool {
        self.function_locals.is_empty() && self.local_scope.is_empty()
    }

    pub fn extract_function_locals(&mut self) -> (UniqueMap<Var, H::SingleType>, BTreeSet<Var>) {
        self.local_scope = UniqueMap::new();
        self.tmp_counter = 0;
        let locals = std::mem::replace(&mut self.function_locals, UniqueMap::new());
        let used = std::mem::take(&mut self.used_locals);
        (locals, used)
    }

    pub fn new_temp(&mut self, loc: Loc, t: H::SingleType) -> Var {
        let new_var = Var(sp(loc, new_temp_name(self)));
        self.function_locals.add(new_var, t).unwrap();
        self.local_scope.add(new_var, new_var).unwrap();
        self.used_locals.insert(new_var);
        new_var
    }

    pub fn bind_local(&mut self, v: Var, t: H::SingleType) {
        let new_var = if !self.function_locals.contains_key(&v) {
            v
        } else {
            Var(sp(v.loc(), new_name(self, v.value())))
        };
        self.function_locals.add(new_var, t).unwrap();
        self.local_scope.remove(&v);
        assert!(!self.local_scope.contains_key(&new_var));
        self.local_scope.add(v, new_var).unwrap();
    }

    pub fn remapped_local(&mut self, v: Var) -> Var {
        let remapped = *self.local_scope.get(&v).unwrap();
        self.used_locals.insert(remapped);
        remapped
    }

    pub fn add_struct_fields(&mut self, structs: &UniqueMap<StructName, H::StructDefinition>) {
        assert!(self.structs.is_empty());
        for (sname, sdef) in structs.key_cloned_iter() {
            let mut fields = UniqueMap::new();
            let field_map = match &sdef.fields {
                H::StructFields::Native(_) => continue,
                H::StructFields::Defined(m) => m,
            };
            for (idx, (field, _)) in field_map.iter().enumerate() {
                fields.add(*field, idx).unwrap();
            }
            self.structs.add(sname, fields).unwrap();
        }
    }

    pub fn fields(&self, struct_name: &StructName) -> Option<&UniqueMap<Field, usize>> {
        let fields = self.structs.get(struct_name);
        // if fields are none, the struct must be defined in another module,
        // in that case, there should be errors
        assert!(fields.is_some() || self.env.has_diags());
        fields
    }

    fn counter_next(&mut self) -> usize {
        self.tmp_counter += 1;
        self.tmp_counter
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(
    compilation_env: &mut CompilationEnv,
    _pre_compiled_lib: Option<&FullyCompiledProgram>,
    prog: T::Program,
) -> H::Program {
    let mut context = Context::new(compilation_env);
    let T::Program {
        modules: tmodules,
        scripts: tscripts,
    } = prog;
    let modules = modules(&mut context, tmodules);
    let scripts = scripts(&mut context, tscripts);

    H::Program { modules, scripts }
}

fn modules(
    context: &mut Context,
    modules: UniqueMap<ModuleIdent, T::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, H::ModuleDefinition> {
    let hlir_modules = modules
        .into_iter()
        .map(|(mname, m)| module(context, mname, m));
    UniqueMap::maybe_from_iter(hlir_modules).unwrap()
}

fn module(
    context: &mut Context,
    module_ident: ModuleIdent,
    mdef: T::ModuleDefinition,
) -> (ModuleIdent, H::ModuleDefinition) {
    let T::ModuleDefinition {
        attributes,
        is_source_module,
        dependency_order,
        friends,
        structs: tstructs,
        functions: tfunctions,
        constants: tconstants,
    } = mdef;

    let structs = tstructs.map(|name, s| struct_def(context, name, s));
    context.add_struct_fields(&structs);

    let constants = tconstants.map(|name, c| constant(context, name, c));
    let functions = tfunctions.map(|name, f| function(context, name, f));

    context.structs = UniqueMap::new();
    (
        module_ident,
        H::ModuleDefinition {
            attributes,
            is_source_module,
            dependency_order,
            friends,
            structs,
            constants,
            functions,
        },
    )
}

fn scripts(
    context: &mut Context,
    tscripts: BTreeMap<Symbol, T::Script>,
) -> BTreeMap<Symbol, H::Script> {
    tscripts
        .into_iter()
        .map(|(n, s)| (n, script(context, s)))
        .collect()
}

fn script(context: &mut Context, tscript: T::Script) -> H::Script {
    let T::Script {
        attributes,
        loc,
        constants: tconstants,
        function_name,
        function: tfunction,
    } = tscript;
    let constants = tconstants.map(|name, c| constant(context, name, c));
    let function = function(context, function_name, tfunction);
    H::Script {
        attributes,

        loc,
        constants,
        function_name,
        function,
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: T::Function) -> H::Function {
    assert!(context.has_empty_locals());
    assert!(context.tmp_counter == 0);
    let attributes = f.attributes;
    let visibility = f.visibility;
    let signature = function_signature(context, f.signature);
    let acquires = f.acquires;
    let body = function_body(context, &signature, f.body);
    H::Function {
        attributes,
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_signature(context: &mut Context, sig: N::FunctionSignature) -> H::FunctionSignature {
    let type_parameters = sig.type_parameters;
    let parameters = sig
        .parameters
        .into_iter()
        .map(|(v, tty)| {
            let ty = single_type(context, tty);
            context.bind_local(v, ty.clone());
            (v, ty)
        })
        .collect();
    let return_type = type_(context, sig.return_type);
    H::FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    }
}

fn function_body(
    context: &mut Context,
    sig: &H::FunctionSignature,
    sp!(loc, tb_): T::FunctionBody,
) -> H::FunctionBody {
    use H::FunctionBody_ as HB;
    use T::FunctionBody_ as TB;
    let b_ = match tb_ {
        TB::Native => {
            context.extract_function_locals();
            HB::Native
        }
        TB::Defined(seq) => {
            let (locals, body) = function_body_defined(context, sig, loc, seq);
            HB::Defined { locals, body }
        }
    };
    sp(loc, b_)
}

fn function_body_defined(
    context: &mut Context,
    signature: &H::FunctionSignature,
    loc: Loc,
    seq: T::Sequence,
) -> (UniqueMap<Var, H::SingleType>, Block) {
    let mut body = VecDeque::new();
    context.signature = Some(signature.clone());
    let final_exp = block(context, &mut body, loc, Some(&signature.return_type), seq);
    match &final_exp.exp.value {
        H::UnannotatedExp_::Unreachable => (),
        _ => {
            use H::{Command_ as C, Statement_ as S};
            let eloc = final_exp.exp.loc;
            let ret = sp(
                eloc,
                C::Return {
                    from_user: false,
                    exp: final_exp,
                },
            );
            body.push_back(sp(eloc, S::Command(ret)))
        }
    }
    let (mut locals, used) = context.extract_function_locals();
    let unused = check_unused_locals(context, &mut locals, used);
    check_trailing_unit(context, &mut body);
    remove_unused_bindings(&unused, &mut body);
    context.signature = None;
    (locals, body)
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

fn constant(context: &mut Context, _name: ConstantName, cdef: T::Constant) -> H::Constant {
    let T::Constant {
        attributes,
        loc,
        signature: tsignature,
        value: tvalue,
    } = cdef;
    let signature = base_type(context, tsignature);
    let eloc = tvalue.exp.loc;
    let tseq = {
        let mut v = T::Sequence::new();
        v.push_back(sp(eloc, T::SequenceItem_::Seq(Box::new(tvalue))));
        v
    };
    let function_signature = H::FunctionSignature {
        type_parameters: vec![],
        parameters: vec![],
        return_type: H::Type_::base(signature.clone()),
    };
    let (locals, body) = function_body_defined(context, &function_signature, eloc, tseq);
    H::Constant {
        attributes,
        loc,
        signature,
        value: (locals, body),
    }
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(
    context: &mut Context,
    _name: StructName,
    sdef: N::StructDefinition,
) -> H::StructDefinition {
    let attributes = sdef.attributes;
    let abilities = sdef.abilities;
    let type_parameters = sdef.type_parameters;
    let fields = struct_fields(context, sdef.fields);
    H::StructDefinition {
        attributes,
        abilities,
        type_parameters,
        fields,
    }
}

fn struct_fields(context: &mut Context, tfields: N::StructFields) -> H::StructFields {
    let tfields_map = match tfields {
        N::StructFields::Native(loc) => return H::StructFields::Native(loc),
        N::StructFields::Defined(m) => m,
    };
    let mut indexed_fields = tfields_map
        .into_iter()
        .map(|(f, (idx, t))| (idx, (f, base_type(context, t))))
        .collect::<Vec<_>>();
    indexed_fields.sort_by(|(idx1, _), (idx2, _)| idx1.cmp(idx2));
    H::StructFields::Defined(indexed_fields.into_iter().map(|(_, f_ty)| f_ty).collect())
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_name(_context: &Context, sp!(loc, ntn_): N::TypeName) -> H::TypeName {
    use H::TypeName_ as HT;
    use N::TypeName_ as NT;
    let tn_ = match ntn_ {
        NT::Multiple(_) => panic!(
            "ICE type constraints failed {}:{}-{}",
            loc.file(),
            loc.start(),
            loc.end()
        ),
        NT::Builtin(bt) => HT::Builtin(bt),
        NT::ModuleType(m, s) => HT::ModuleType(m, s),
    };
    sp(loc, tn_)
}

fn base_types<R: std::iter::FromIterator<H::BaseType>>(
    context: &Context,
    tys: impl IntoIterator<Item = N::Type>,
) -> R {
    tys.into_iter().map(|t| base_type(context, t)).collect()
}

fn base_type(context: &Context, sp!(loc, nb_): N::Type) -> H::BaseType {
    use H::BaseType_ as HB;
    use N::Type_ as NT;
    let b_ = match nb_ {
        NT::Var(_) => panic!(
            "ICE tvar not expanded: {}:{}-{}",
            loc.file(),
            loc.start(),
            loc.end()
        ),
        NT::Apply(None, n, tys) => {
            crate::shared::ast_debug::print_verbose(&NT::Apply(None, n, tys));
            panic!("ICE kind not expanded: {:#?}", loc)
        }
        NT::Apply(Some(k), n, nbs) => HB::Apply(k, type_name(context, n), base_types(context, nbs)),
        NT::Param(tp) => HB::Param(tp),
        NT::UnresolvedError => HB::UnresolvedError,
        NT::Anything => HB::Unreachable,
        NT::Ref(_, _) | NT::Unit => {
            panic!(
                "ICE type constraints failed {}:{}-{}",
                loc.file(),
                loc.start(),
                loc.end()
            )
        }
    };
    sp(loc, b_)
}

fn expected_types(context: &Context, loc: Loc, nss: Vec<Option<N::Type>>) -> H::Type {
    let any = || {
        sp(
            loc,
            H::SingleType_::Base(sp(loc, H::BaseType_::UnresolvedError)),
        )
    };
    let ss = nss
        .into_iter()
        .map(|sopt| sopt.map(|s| single_type(context, s)).unwrap_or_else(any))
        .collect::<Vec<_>>();
    H::Type_::from_vec(loc, ss)
}

fn single_types(context: &Context, ss: Vec<N::Type>) -> Vec<H::SingleType> {
    ss.into_iter().map(|s| single_type(context, s)).collect()
}

fn single_type(context: &Context, sp!(loc, ty_): N::Type) -> H::SingleType {
    use H::SingleType_ as HS;
    use N::Type_ as NT;
    let s_ = match ty_ {
        NT::Ref(mut_, nb) => HS::Ref(mut_, base_type(context, *nb)),
        _ => HS::Base(base_type(context, sp(loc, ty_))),
    };
    sp(loc, s_)
}

fn type_(context: &Context, sp!(loc, ty_): N::Type) -> H::Type {
    use H::Type_ as HT;
    use N::{TypeName_ as TN, Type_ as NT};
    let t_ = match ty_ {
        NT::Unit => HT::Unit,
        NT::Apply(None, n, tys) => {
            crate::shared::ast_debug::print_verbose(&NT::Apply(None, n, tys));
            panic!("ICE kind not expanded: {:#?}", loc)
        }
        NT::Apply(Some(_), sp!(_, TN::Multiple(_)), ss) => HT::Multiple(single_types(context, ss)),
        _ => HT::Single(single_type(context, sp(loc, ty_))),
    };
    sp(loc, t_)
}

//**************************************************************************************************
// Statements
//**************************************************************************************************

fn block(
    context: &mut Context,
    result: &mut Block,
    loc: Loc,
    expected_type_opt: Option<&H::Type>,
    mut seq: T::Sequence,
) -> H::Exp {
    use T::SequenceItem_ as S;
    let last = match seq.pop_back() {
        None => {
            return H::exp(
                sp(loc, H::Type_::Unit),
                sp(
                    loc,
                    H::UnannotatedExp_::Unit {
                        case: H::UnitCase::FromUser,
                    },
                ),
            )
        }
        Some(sp!(_, S::Seq(last))) => last,
        Some(_) => panic!("ICE last sequence item should be exp"),
    };

    let old_scope = context.local_scope.clone();
    for sp!(sloc, seq_item_) in seq {
        match seq_item_ {
            S::Seq(te) => statement(context, result, *te),
            S::Declare(binds) => declare_bind_list(context, &binds),
            S::Bind(binds, ty, e) => {
                let expected_tys = expected_types(context, sloc, ty);
                let res = exp_(context, result, Some(&expected_tys), *e);
                declare_bind_list(context, &binds);
                assign_command(context, result, sloc, binds, res);
            }
        }
    }
    let res = exp_(context, result, expected_type_opt, *last);
    context.local_scope = old_scope;
    res
}

fn statement(context: &mut Context, result: &mut Block, e: T::Exp) {
    use H::Statement_ as S;
    use T::UnannotatedExp_ as TE;

    let ty = e.ty;
    let sp!(eloc, e_) = e.exp;
    let stmt_ = match e_ {
        TE::IfElse(tb, tt, tf) => {
            let cond = exp(context, result, None, *tb);

            let mut if_block = Block::new();
            let et = exp_(context, &mut if_block, None, *tt);
            ignore_and_pop(&mut if_block, et);

            let mut else_block = Block::new();
            let ef = exp_(context, &mut else_block, None, *tf);
            ignore_and_pop(&mut else_block, ef);

            S::IfElse {
                cond,
                if_block,
                else_block,
            }
        }
        TE::While(tb, loop_body) => {
            let mut cond_block = Block::new();
            let cond_exp = exp(context, &mut cond_block, None, *tb);

            let mut loop_block = Block::new();
            let el = exp_(context, &mut loop_block, None, *loop_body);
            ignore_and_pop(&mut loop_block, el);

            S::While {
                cond: (cond_block, cond_exp),
                block: loop_block,
            }
        }
        TE::Loop {
            body: loop_body,
            has_break,
        } => {
            let loop_block = statement_loop_body(context, *loop_body);

            S::Loop {
                block: loop_block,
                has_break,
            }
        }
        TE::Block(seq) => {
            let res = block(context, result, eloc, None, seq);
            ignore_and_pop(result, res);
            return;
        }
        e_ => {
            let te = T::exp(ty, sp(eloc, e_));
            let e = exp_(context, result, None, te);
            ignore_and_pop(result, e);
            return;
        }
    };
    result.push_back(sp(eloc, stmt_))
}

fn statement_loop_body(context: &mut Context, body: T::Exp) -> Block {
    let mut loop_block = Block::new();
    let el = exp_(context, &mut loop_block, None, body);
    ignore_and_pop(&mut loop_block, el);
    loop_block
}

//**************************************************************************************************
// LValue
//**************************************************************************************************

fn declare_bind_list(context: &mut Context, sp!(_, binds): &T::LValueList) {
    binds.iter().for_each(|b| declare_bind(context, b))
}

fn declare_bind(context: &mut Context, sp!(_, bind_): &T::LValue) {
    use T::LValue_ as L;
    match bind_ {
        L::Ignore => (),
        L::Var(v, ty) => {
            let st = single_type(context, *ty.clone());
            context.bind_local(*v, st)
        }
        L::Unpack(_, _, _, fields) | L::BorrowUnpack(_, _, _, _, fields) => fields
            .iter()
            .for_each(|(_, _, (_, (_, b)))| declare_bind(context, b)),
    }
}

fn assign_command(
    context: &mut Context,
    result: &mut Block,
    loc: Loc,
    sp!(_, assigns): T::LValueList,
    rvalue: H::Exp,
) {
    use H::{Command_ as C, Statement_ as S};
    let mut lvalues = vec![];
    let mut after = Block::new();
    for (idx, a) in assigns.into_iter().enumerate() {
        let a_ty = rvalue.ty.value.type_at_index(idx);
        let (ls, mut af) = assign(context, result, a, a_ty);

        lvalues.push(ls);
        after.append(&mut af);
    }
    result.push_back(sp(
        loc,
        S::Command(sp(loc, C::Assign(lvalues, Box::new(rvalue)))),
    ));
    result.append(&mut after);
}

fn assign(
    context: &mut Context,
    result: &mut Block,
    sp!(loc, ta_): T::LValue,
    rvalue_ty: &H::SingleType,
) -> (H::LValue, Block) {
    use H::{LValue_ as L, UnannotatedExp_ as E};
    use T::LValue_ as A;
    let mut after = Block::new();
    let l_ = match ta_ {
        A::Ignore => L::Ignore,
        A::Var(v, st) => L::Var(
            context.remapped_local(v),
            Box::new(single_type(context, *st)),
        ),
        A::Unpack(_m, s, tbs, tfields) => {
            let bs = base_types(context, tbs);

            let mut fields = vec![];
            for (decl_idx, f, bt, tfa) in assign_fields(context, &s, tfields) {
                assert!(fields.len() == decl_idx);
                let st = &H::SingleType_::base(bt);
                let (fa, mut fafter) = assign(context, result, tfa, st);
                after.append(&mut fafter);
                fields.push((f, fa))
            }
            L::Unpack(s, bs, fields)
        }
        A::BorrowUnpack(mut_, _m, s, _tss, tfields) => {
            let tmp = context.new_temp(loc, rvalue_ty.clone());
            let copy_tmp = || {
                let copy_tmp_ = E::Copy {
                    from_user: false,
                    var: tmp,
                };
                H::exp(H::Type_::single(rvalue_ty.clone()), sp(loc, copy_tmp_))
            };
            let fields = assign_fields(context, &s, tfields).into_iter().enumerate();
            for (idx, (decl_idx, f, bt, tfa)) in fields {
                assert!(idx == decl_idx);
                let floc = tfa.loc;
                let borrow_ = E::Borrow(mut_, Box::new(copy_tmp()), f);
                let borrow_ty = H::Type_::single(sp(floc, H::SingleType_::Ref(mut_, bt)));
                let borrow = H::exp(borrow_ty, sp(floc, borrow_));
                assign_command(context, &mut after, floc, sp(floc, vec![tfa]), borrow);
            }
            L::Var(tmp, Box::new(rvalue_ty.clone()))
        }
    };
    (sp(loc, l_), after)
}

fn assign_fields(
    context: &Context,
    s: &StructName,
    tfields: Fields<(N::Type, T::LValue)>,
) -> Vec<(usize, Field, H::BaseType, T::LValue)> {
    let decl_fields = context.fields(s);
    let mut count = 0;
    let mut decl_field = |f: &Field| -> usize {
        match decl_fields {
            Some(m) => *m.get(f).unwrap(),
            None => {
                // none can occur with errors in typing
                let i = count;
                count += 1;
                i
            }
        }
    };
    let mut tfields_vec = tfields
        .into_iter()
        .map(|(f, (_idx, (tbt, tfa)))| (decl_field(&f), f, base_type(context, tbt), tfa))
        .collect::<Vec<_>>();
    tfields_vec.sort_by(|(idx1, _, _, _), (idx2, _, _, _)| idx1.cmp(idx2));
    tfields_vec
}

//**************************************************************************************************
// Commands
//**************************************************************************************************

fn ignore_and_pop(result: &mut Block, e: H::Exp) {
    match &e.exp.value {
        H::UnannotatedExp_::Unreachable => (),
        _ => {
            let pop_num = match &e.ty.value {
                H::Type_::Unit => 0,
                H::Type_::Single(_) => 1,
                H::Type_::Multiple(tys) => tys.len(),
            };
            let loc = e.exp.loc;
            let c = sp(loc, H::Command_::IgnoreAndPop { pop_num, exp: e });
            result.push_back(sp(loc, H::Statement_::Command(c)))
        }
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn exp(
    context: &mut Context,
    result: &mut Block,
    expected_type_opt: Option<&H::Type>,
    te: T::Exp,
) -> Box<H::Exp> {
    Box::new(exp_(context, result, expected_type_opt, te))
}

fn exp_<'env>(
    context: &mut Context<'env>,
    result: &mut Block,
    initial_expected_type_opt: Option<&H::Type>,
    initial_e: T::Exp,
) -> H::Exp {
    use std::{cell::RefCell, rc::Rc};

    struct Stack<'a, 'env> {
        frames: Vec<Box<dyn FnOnce(&mut Self)>>,
        operands: Vec<H::Exp>,
        context: &'a mut Context<'env>,
    }

    macro_rules! inner {
        ($block:expr, $exp_ty_opt:expr, $e:expr) => {{
            let e_result = $block.clone();
            let e_exp_ty_opt = $exp_ty_opt;
            move |s: &mut Stack| exp_loop(s, e_result, e_exp_ty_opt, $e)
        }};
    }

    fn maybe_freeze(
        context: &mut Context,
        result: &mut Block,
        expected_type_opt: Option<H::Type>,
        e: H::Exp,
    ) -> H::Exp {
        match (&e.exp.value, expected_type_opt.as_ref()) {
            (H::UnannotatedExp_::Unreachable, _) => e,
            (_, Some(exty)) if needs_freeze(context, &e.ty, exty) != Freeze::NotNeeded => {
                freeze(context, result, exty, e)
            }
            _ => e,
        }
    }

    fn exp_loop(
        stack: &mut Stack,
        result: Rc<RefCell<Block>>,
        cur_expected_type_opt: Option<H::Type>,
        cur: Box<T::Exp>,
    ) {
        use H::{Statement_ as S, UnannotatedExp_ as HE};
        use T::UnannotatedExp_ as TE;

        let (tty, sp!(loc, cur_)) = (cur.ty, cur.exp);
        let ty = type_(stack.context, tty);
        match cur_ {
            //***********************************************
            // Stack-ified traversal
            //***********************************************
            TE::IfElse(cond, if_true, if_false) => {
                let f_cond = inner!(result, None, cond);

                let if_block = Rc::new(RefCell::new(Block::new()));
                let f_if = inner!(if_block, Some(ty.clone()), if_true);

                let else_block = Rc::new(RefCell::new(Block::new()));
                let f_else = inner!(else_block, Some(ty.clone()), if_false);

                let f_if_else = move |s: &mut Stack| {
                    let ef = s.operands.pop().unwrap();
                    let et = s.operands.pop().unwrap();
                    let cond = Box::new(s.operands.pop().unwrap());

                    let mut if_block = Rc::try_unwrap(if_block).unwrap().into_inner();
                    let mut else_block = Rc::try_unwrap(else_block).unwrap().into_inner();
                    let result = &mut *result.borrow_mut();

                    let e_ = match (&et.exp.value, &ef.exp.value) {
                        (HE::Unreachable, HE::Unreachable) => {
                            let s_ = S::IfElse {
                                cond,
                                if_block,
                                else_block,
                            };
                            result.push_back(sp(loc, s_));
                            HE::Unreachable
                        }
                        _ => {
                            let tmps = make_temps(s.context, loc, ty.clone());
                            let tres = bind_exp_(&mut if_block, loc, tmps.clone(), et);
                            let fres = bind_exp_(&mut else_block, loc, tmps, ef);
                            let s_ = S::IfElse {
                                cond,
                                if_block,
                                else_block,
                            };
                            result.push_back(sp(loc, s_));
                            match (tres, fres) {
                                (HE::Unreachable, HE::Unreachable) => unreachable!(),
                                (HE::Unreachable, res) | (res, HE::Unreachable) | (res, _) => res,
                            }
                        }
                    };
                    let e_res = H::exp(ty, sp(loc, e_));
                    // each branch is frozen so no need to freeze
                    s.operands.push(e_res)
                };

                stack.frames.push(Box::new(f_if_else));
                stack.frames.push(Box::new(f_else));
                stack.frames.push(Box::new(f_if));
                stack.frames.push(Box::new(f_cond));
            }
            TE::BinopExp(lhs, op, toperand_ty, rhs) => {
                let operand_exp_ty_opt = match &op.value {
                    BinOp_::And if bind_for_short_circuit(&rhs) => {
                        let tfalse_ = sp(loc, TE::Value(sp(loc, Value_::Bool(false))));
                        let tfalse = Box::new(T::exp(N::Type_::bool(loc), tfalse_));
                        let if_else_ = sp(loc, TE::IfElse(lhs, rhs, tfalse));
                        let if_else = Box::new(T::exp(N::Type_::bool(ty.loc), if_else_));
                        return exp_loop(stack, result, cur_expected_type_opt, if_else);
                    }
                    BinOp_::Or if bind_for_short_circuit(&rhs) => {
                        let ttrue_ = sp(loc, TE::Value(sp(loc, Value_::Bool(true))));
                        let ttrue = Box::new(T::exp(N::Type_::bool(loc), ttrue_));
                        let if_else_ = sp(loc, TE::IfElse(lhs, ttrue, rhs));
                        let if_else = Box::new(T::exp(N::Type_::bool(ty.loc), if_else_));
                        return exp_loop(stack, result, cur_expected_type_opt, if_else);
                    }
                    BinOp_::Eq | BinOp_::Neq => {
                        let operand_ty = type_(stack.context, *toperand_ty);
                        Some(freeze_ty(operand_ty))
                    }
                    _ => None,
                };

                let f_lhs = inner!(result, operand_exp_ty_opt.clone(), lhs);
                let f_rhs = inner!(result, operand_exp_ty_opt, rhs);
                let f_binop = move |s: &mut Stack| {
                    let rhs = Box::new(s.operands.pop().unwrap());
                    let lhs = Box::new(s.operands.pop().unwrap());

                    let result = &mut *result.borrow_mut();

                    let e_res = H::exp(ty, sp(loc, HE::BinopExp(lhs, op, rhs)));
                    let e_res = maybe_freeze(s.context, result, cur_expected_type_opt, e_res);
                    s.operands.push(e_res)
                };
                stack.frames.push(Box::new(f_binop));
                stack.frames.push(Box::new(f_rhs));
                stack.frames.push(Box::new(f_lhs));
            }
            TE::Builtin(bt, arguments) if matches!(&*bt, sp!(_, T::BuiltinFunction_::Assert)) => {
                let tbool = N::Type_::bool(loc);
                let tu64 = N::Type_::u64(loc);
                let tunit = sp(loc, N::Type_::Unit);
                let vcond = Var(sp(loc, new_temp_name(stack.context)));
                let vcode = Var(sp(loc, new_temp_name(stack.context)));

                let mut stmts = VecDeque::new();

                let bvar = |v, st| sp(loc, T::LValue_::Var(v, st));
                let bind_list = sp(
                    loc,
                    vec![
                        bvar(vcond, Box::new(tbool.clone())),
                        bvar(vcode, Box::new(tu64.clone())),
                    ],
                );
                let tys = vec![Some(tbool.clone()), Some(tu64.clone())];
                let bind = sp(loc, T::SequenceItem_::Bind(bind_list, tys, arguments));
                stmts.push_back(bind);

                let mvar = |var, st| {
                    let from_user = false;
                    let mv = TE::Move { from_user, var };
                    T::exp(st, sp(loc, mv))
                };
                let econd = mvar(vcond, tu64);
                let ecode = mvar(vcode, tbool);
                let eabort = T::exp(tunit.clone(), sp(loc, TE::Abort(Box::new(ecode))));
                let eunit = T::exp(tunit.clone(), sp(loc, TE::Unit { trailing: false }));
                let inlined_ = TE::IfElse(Box::new(econd), Box::new(eunit), Box::new(eabort));
                let inlined = T::exp(tunit.clone(), sp(loc, inlined_));
                stmts.push_back(sp(loc, T::SequenceItem_::Seq(Box::new(inlined))));

                let block = T::exp(tunit, sp(loc, TE::Block(stmts)));
                exp_loop(stack, result, cur_expected_type_opt, Box::new(block));
            }
            te_ => {
                let result = &mut *result.borrow_mut();
                let e_res = exp_impl(stack.context, result, ty, loc, te_);
                let e_res = maybe_freeze(stack.context, result, cur_expected_type_opt, e_res);
                stack.operands.push(e_res)
            }
        }
    }

    let mut stack = Stack {
        frames: vec![],
        operands: vec![],
        context,
    };
    let rc_result = Rc::new(RefCell::new(std::mem::take(result)));
    exp_loop(
        &mut stack,
        rc_result.clone(),
        initial_expected_type_opt.cloned(),
        Box::new(initial_e),
    );
    while let Some(f) = stack.frames.pop() {
        f(&mut stack)
    }
    let e_res = stack.operands.pop().unwrap();
    assert!(stack.frames.is_empty());
    assert!(stack.operands.is_empty());
    *result = Rc::try_unwrap(rc_result).unwrap().into_inner();
    e_res
}

enum TmpItem {
    Single(Box<H::SingleType>),
    Splat(Loc, Vec<H::SingleType>),
}

fn exp_impl(
    context: &mut Context,
    result: &mut Block,
    ty: H::Type,
    eloc: Loc,
    e_: T::UnannotatedExp_,
) -> H::Exp {
    use H::{Command_ as C, Statement_ as S, UnannotatedExp_ as HE};
    use T::UnannotatedExp_ as TE;

    let res = match e_ {
        // Statement-like expressions
        TE::While(tb, loop_body) => {
            let mut cond_block = Block::new();
            let cond_exp = exp(context, &mut cond_block, None, *tb);

            let mut loop_block = Block::new();
            let el = exp_(context, &mut loop_block, None, *loop_body);
            ignore_and_pop(&mut loop_block, el);

            let s_ = S::While {
                cond: (cond_block, cond_exp),
                block: loop_block,
            };
            result.push_back(sp(eloc, s_));
            HE::Unit {
                case: H::UnitCase::Implicit,
            }
        }
        TE::Loop {
            has_break,
            body: loop_body,
        } => {
            let loop_block = statement_loop_body(context, *loop_body);

            let s_ = S::Loop {
                block: loop_block,
                has_break,
            };
            result.push_back(sp(eloc, s_));
            if !has_break {
                HE::Unreachable
            } else {
                HE::Unit {
                    case: H::UnitCase::Implicit,
                }
            }
        }
        TE::Block(seq) => return block(context, result, eloc, None, seq),

        // Command-like expressions
        TE::Return(te) => {
            let expected_type = context.signature.as_ref().map(|s| s.return_type.clone());
            let e = exp_(context, result, expected_type.as_ref(), *te);
            let c = sp(
                eloc,
                C::Return {
                    from_user: true,
                    exp: e,
                },
            );
            result.push_back(sp(eloc, S::Command(c)));
            HE::Unreachable
        }
        TE::Abort(te) => {
            let e = exp_(context, result, None, *te);
            let c = sp(eloc, C::Abort(e));
            result.push_back(sp(eloc, S::Command(c)));
            HE::Unreachable
        }
        TE::Break => {
            let c = sp(eloc, C::Break);
            result.push_back(sp(eloc, S::Command(c)));
            HE::Unreachable
        }
        TE::Continue => {
            let c = sp(eloc, C::Continue);
            result.push_back(sp(eloc, S::Command(c)));
            HE::Unreachable
        }
        TE::Assign(assigns, lvalue_ty, te) => {
            let expected_type = expected_types(context, eloc, lvalue_ty);
            let e = exp_(context, result, Some(&expected_type), *te);
            assign_command(context, result, eloc, assigns, e);
            HE::Unit {
                case: H::UnitCase::Implicit,
            }
        }
        TE::Mutate(tl, tr) => {
            let er = exp(context, result, None, *tr);
            let el = exp(context, result, None, *tl);
            let c = sp(eloc, C::Mutate(el, er));
            result.push_back(sp(eloc, S::Command(c)));
            HE::Unit {
                case: H::UnitCase::Implicit,
            }
        }
        // All other expressiosn
        TE::Unit { trailing } => HE::Unit {
            case: if trailing {
                H::UnitCase::Trailing
            } else {
                H::UnitCase::FromUser
            },
        },
        TE::Value(v) => HE::Value(v),
        TE::Constant(_m, c) => {
            // Currently only private constants exist
            HE::Constant(c)
        }
        TE::Move { from_user, var } => HE::Move {
            from_user,
            var: context.remapped_local(var),
        },
        TE::Copy { from_user, var } => HE::Copy {
            from_user,
            var: context.remapped_local(var),
        },
        TE::BorrowLocal(mut_, v) => HE::BorrowLocal(mut_, context.remapped_local(v)),

        TE::Use(_) => panic!("ICE unexpanded use"),
        TE::ModuleCall(call) => {
            let T::ModuleCall {
                module,
                name,
                type_arguments,
                arguments,
                parameter_types,
                acquires,
            } = *call;
            let expected_type = H::Type_::from_vec(eloc, single_types(context, parameter_types));
            let htys = base_types(context, type_arguments);
            let harg = exp(context, result, Some(&expected_type), *arguments);
            let call = H::ModuleCall {
                module,
                name,
                type_arguments: htys,
                arguments: harg,
                acquires,
            };
            HE::ModuleCall(Box::new(call))
        }
        TE::Builtin(bf, targ) => builtin(context, result, eloc, *bf, targ),
        TE::Dereference(te) => {
            let e = exp(context, result, None, *te);
            HE::Dereference(e)
        }
        TE::UnaryExp(op, te) => {
            let e = exp(context, result, None, *te);
            HE::UnaryExp(op, e)
        }

        TE::Pack(_, s, tbs, tfields) => {
            let bs = base_types(context, tbs);

            let decl_fields = context.fields(&s);
            let mut count = 0;
            let mut decl_field = |f: &Field| -> usize {
                match decl_fields {
                    Some(m) => *m.get(f).unwrap(),
                    None => {
                        // none can occur with errors in typing
                        let i = count;
                        count += 1;
                        i
                    }
                }
            };

            let mut texp_fields: Vec<(usize, Field, usize, N::Type, T::Exp)> = tfields
                .into_iter()
                .map(|(f, (exp_idx, (bt, tf)))| (decl_field(&f), f, exp_idx, bt, tf))
                .collect();
            texp_fields.sort_by(|(_, _, eidx1, _, _), (_, _, eidx2, _, _)| eidx1.cmp(eidx2));

            let bind_all_fields = texp_fields
                .iter()
                .any(|(decl_idx, _, exp_idx, _, _)| decl_idx != exp_idx);
            let fields = if !bind_all_fields {
                let mut fs = vec![];
                let tes = texp_fields
                    .into_iter()
                    .map(|(_, f, _, bt, te)| {
                        let bt = base_type(context, bt);
                        fs.push((f, bt.clone()));
                        let t = H::Type_::base(bt);
                        (te, Some(t))
                    })
                    .collect();
                let es = exp_evaluation_order(context, result, tes);
                assert!(
                    fs.len() == es.len(),
                    "ICE exp_evaluation_order changed arity"
                );
                es.into_iter()
                    .zip(fs)
                    .map(|(e, (f, bt))| (f, bt, e))
                    .collect()
            } else {
                let num_fields = decl_fields.as_ref().map(|m| m.len()).unwrap_or(0);
                let mut fields = (0..num_fields).map(|_| None).collect::<Vec<_>>();
                for (decl_idx, f, _exp_idx, bt, tf) in texp_fields {
                    // Might have too many arguments, there will be an error from typing
                    if decl_idx > fields.len() {
                        debug_assert!(context.env.has_diags());
                        break;
                    }
                    let bt = base_type(context, bt);
                    let t = H::Type_::base(bt.clone());
                    let ef = exp_(context, result, Some(&t), tf);
                    assert!(fields.get(decl_idx).unwrap().is_none());
                    let move_tmp = bind_exp(context, result, ef);
                    fields[decl_idx] = Some((f, bt, move_tmp))
                }
                // Might have too few arguments, there will be an error from typing if so
                fields
                    .into_iter()
                    .filter_map(|o| {
                        // if o is None, context should have errors
                        debug_assert!(o.is_some() || context.env.has_diags());
                        o
                    })
                    .collect()
            };
            HE::Pack(s, bs, fields)
        }
        TE::ExpList(titems) => {
            assert!(!titems.is_empty());
            let mut tmp_items = vec![];
            let mut tes = vec![];
            for titem in titems {
                match titem {
                    T::ExpListItem::Single(te, ts) => {
                        let s = single_type(context, *ts);
                        tmp_items.push(TmpItem::Single(Box::new(s)));
                        tes.push((te, None));
                    }
                    T::ExpListItem::Splat(sloc, te, tss) => {
                        let ss = single_types(context, tss);
                        tmp_items.push(TmpItem::Splat(sloc, ss));
                        tes.push((te, None));
                    }
                }
            }
            let es = exp_evaluation_order(context, result, tes);
            assert!(
                es.len() == tmp_items.len(),
                "ICE exp_evaluation_order changed arity"
            );
            let items = es
                .into_iter()
                .zip(tmp_items)
                .map(|(e, tmp_item)| match tmp_item {
                    TmpItem::Single(s) => H::ExpListItem::Single(e, s),
                    TmpItem::Splat(loc, ss) => H::ExpListItem::Splat(loc, e, ss),
                })
                .collect();
            HE::ExpList(items)
        }
        TE::Borrow(mut_, te, f) => {
            let e = exp(context, result, None, *te);
            HE::Borrow(mut_, e, f)
        }
        TE::TempBorrow(mut_, te) => {
            let eb = exp_(context, result, None, *te);
            let tmp = match bind_exp_impl(context, result, eb, true).exp.value {
                HE::Move {
                    from_user: false,
                    var,
                } => var,
                _ => panic!("ICE invalid bind_exp for single value"),
            };
            HE::BorrowLocal(mut_, tmp)
        }
        TE::Cast(te, rhs_ty) => {
            use N::BuiltinTypeName_ as BT;
            let e = exp(context, result, None, *te);
            let bt = match rhs_ty.value.builtin_name() {
                Some(bt @ sp!(_, BT::U8))
                | Some(bt @ sp!(_, BT::U64))
                | Some(bt @ sp!(_, BT::U128)) => bt.clone(),
                _ => panic!("ICE typing failed for cast"),
            };
            HE::Cast(e, bt)
        }
        TE::Annotate(te, rhs_ty) => {
            let expected_ty = type_(context, *rhs_ty);
            return exp_(context, result, Some(&expected_ty), *te);
        }
        TE::Spec(u, tused_locals) => {
            let used_locals = tused_locals
                .into_iter()
                .map(|(var, ty)| {
                    let v = context.remapped_local(var);
                    let st = single_type(context, ty);
                    (v, st)
                })
                .collect();
            HE::Spec(u, used_locals)
        }
        TE::UnresolvedError => {
            assert!(context.env.has_diags());
            HE::UnresolvedError
        }

        TE::IfElse(..) | TE::BinopExp(..) => unreachable!(),
    };
    H::exp(ty, sp(eloc, res))
}

fn exp_evaluation_order(
    context: &mut Context,
    result: &mut Block,
    tes: Vec<(T::Exp, Option<H::Type>)>,
) -> Vec<H::Exp> {
    let mut needs_binding = false;
    let mut e_results = vec![];
    for (te, expected_type) in tes.into_iter().rev() {
        let mut tmp_result = Block::new();
        let e = *exp(context, &mut tmp_result, expected_type.as_ref(), te);
        // If evaluating this expression introduces statements, all previous exps need to be bound
        // to preserve left-to-right evaluation order
        let adds_to_result = !tmp_result.is_empty();

        let e = if needs_binding {
            bind_exp(context, &mut tmp_result, e)
        } else {
            e
        };
        e_results.push((tmp_result, e));

        needs_binding = needs_binding || adds_to_result;
    }

    let mut es = vec![];
    for (mut tmp_result, e) in e_results.into_iter().rev() {
        result.append(&mut tmp_result);
        es.push(e)
    }
    es
}

fn make_temps(context: &mut Context, loc: Loc, ty: H::Type) -> Vec<(Var, H::SingleType)> {
    use H::Type_ as T;
    match ty.value {
        T::Unit => vec![],
        T::Single(s) => vec![(context.new_temp(loc, s.clone()), s)],
        T::Multiple(ss) => ss
            .into_iter()
            .map(|s| (context.new_temp(loc, s.clone()), s))
            .collect(),
    }
}

fn bind_exp(context: &mut Context, result: &mut Block, e: H::Exp) -> H::Exp {
    bind_exp_impl(context, result, e, false)
}

fn bind_exp_(
    result: &mut Block,
    loc: Loc,
    tmps: Vec<(Var, H::SingleType)>,
    e: H::Exp,
) -> H::UnannotatedExp_ {
    bind_exp_impl_(result, loc, tmps, e, false)
}

fn bind_exp_impl(
    context: &mut Context,
    result: &mut Block,
    e: H::Exp,
    bind_unreachable: bool,
) -> H::Exp {
    if matches!(&e.exp.value, H::UnannotatedExp_::Unreachable) && !bind_unreachable {
        return e;
    }
    let loc = e.exp.loc;
    let ty = e.ty.clone();
    let tmps = make_temps(context, loc, ty.clone());
    H::exp(
        ty,
        sp(loc, bind_exp_impl_(result, loc, tmps, e, bind_unreachable)),
    )
}

fn bind_exp_impl_(
    result: &mut Block,
    loc: Loc,
    tmps: Vec<(Var, H::SingleType)>,
    e: H::Exp,
    bind_unreachable: bool,
) -> H::UnannotatedExp_ {
    use H::{Command_ as C, Statement_ as S, UnannotatedExp_ as E};
    if matches!(&e.exp.value, H::UnannotatedExp_::Unreachable) && !bind_unreachable {
        return H::UnannotatedExp_::Unreachable;
    }

    if tmps.is_empty() {
        let cmd = sp(loc, C::IgnoreAndPop { pop_num: 0, exp: e });
        result.push_back(sp(loc, S::Command(cmd)));
        return E::Unit {
            case: H::UnitCase::Implicit,
        };
    }
    let lvalues = tmps
        .iter()
        .map(|(v, st)| sp(v.loc(), H::LValue_::Var(*v, Box::new(st.clone()))))
        .collect();
    let asgn = sp(loc, C::Assign(lvalues, Box::new(e)));
    result.push_back(sp(loc, S::Command(asgn)));

    let mut etemps = tmps
        .into_iter()
        .map(|(var, st)| {
            let evar_ = sp(var.loc(), use_tmp(var));
            let ty = sp(st.loc, H::Type_::Single(st.clone()));
            let evar = H::exp(ty, evar_);
            H::ExpListItem::Single(evar, Box::new(st))
        })
        .collect::<Vec<_>>();
    match etemps.len() {
        0 => unreachable!(),
        1 => match etemps.pop().unwrap() {
            H::ExpListItem::Single(e, _) => e.exp.value,
            H::ExpListItem::Splat(_, _, _) => unreachable!(),
        },
        _ => E::ExpList(etemps),
    }
}

fn use_tmp(var: Var) -> H::UnannotatedExp_ {
    use H::UnannotatedExp_ as E;
    E::Move {
        from_user: false,
        var,
    }
}

fn builtin(
    context: &mut Context,
    result: &mut Block,
    _eloc: Loc,
    sp!(loc, tb_): T::BuiltinFunction,
    targ: Box<T::Exp>,
) -> H::UnannotatedExp_ {
    use H::{BuiltinFunction_ as HB, UnannotatedExp_ as E};
    use T::BuiltinFunction_ as TB;
    match tb_ {
        TB::MoveTo(bt) => {
            let texpected_tys = vec![
                sp(loc, N::Type_::Ref(false, Box::new(N::Type_::signer(loc)))),
                bt.clone(),
            ];
            let texpected_ty_ = N::Type_::Apply(
                Some(AbilitySet::empty()), // Should be unused
                sp(loc, N::TypeName_::Multiple(texpected_tys.len())),
                texpected_tys,
            );
            let expected_ty = type_(context, sp(loc, texpected_ty_));
            let arg = exp(context, result, Some(&expected_ty), *targ);
            let ty = base_type(context, bt);
            E::Builtin(Box::new(sp(loc, HB::MoveTo(ty))), arg)
        }
        TB::MoveFrom(bt) => {
            let ty = base_type(context, bt);
            let arg = exp(context, result, None, *targ);
            E::Builtin(Box::new(sp(loc, HB::MoveFrom(ty))), arg)
        }
        TB::BorrowGlobal(mut_, bt) => {
            let ty = base_type(context, bt);
            let arg = exp(context, result, None, *targ);
            E::Builtin(Box::new(sp(loc, HB::BorrowGlobal(mut_, ty))), arg)
        }
        TB::Exists(bt) => {
            let ty = base_type(context, bt);
            let arg = exp(context, result, None, *targ);
            E::Builtin(Box::new(sp(loc, HB::Exists(ty))), arg)
        }
        TB::Freeze(_bt) => {
            let arg = exp(context, result, None, *targ);
            E::Freeze(arg)
        }
        TB::Assert => unreachable!(),
    }
}

//**************************************************************************************************
// Freezing
//**************************************************************************************************

#[derive(PartialEq, Eq)]
enum Freeze {
    NotNeeded,
    Point,
    Sub(Vec<bool>),
}

fn needs_freeze(context: &Context, sp!(_, actual): &H::Type, sp!(_, expected): &H::Type) -> Freeze {
    use H::Type_ as T;
    match (actual, expected) {
        (T::Unit, T::Unit) => Freeze::NotNeeded,
        (T::Single(actaul_s), T::Single(actual_e)) => {
            let needs = needs_freeze_single(actaul_s, actual_e);
            if needs {
                Freeze::Point
            } else {
                Freeze::NotNeeded
            }
        }
        (T::Multiple(actaul_ss), T::Multiple(actual_es)) => {
            assert!(actaul_ss.len() == actual_es.len());
            let points = actaul_ss
                .iter()
                .zip(actual_es)
                .map(|(a, e)| needs_freeze_single(a, e))
                .collect::<Vec<_>>();
            if points.iter().any(|needs| *needs) {
                Freeze::Sub(points)
            } else {
                Freeze::NotNeeded
            }
        }
        (_actual, _expected) => {
            assert!(context.env.has_diags());
            Freeze::NotNeeded
        }
    }
}

fn needs_freeze_single(sp!(_, actual): &H::SingleType, sp!(_, expected): &H::SingleType) -> bool {
    use H::SingleType_ as T;
    matches!((actual, expected), (T::Ref(true, _), T::Ref(false, _)))
}

fn freeze(context: &mut Context, result: &mut Block, expected_type: &H::Type, e: H::Exp) -> H::Exp {
    use H::{Type_ as T, UnannotatedExp_ as E};

    match needs_freeze(context, &e.ty, expected_type) {
        Freeze::NotNeeded => e,
        Freeze::Point => freeze_point(e),

        Freeze::Sub(points) => {
            let loc = e.exp.loc;
            let actual_tys = match &e.ty.value {
                T::Multiple(v) => v.clone(),
                _ => unreachable!("ICE needs_freeze failed"),
            };
            assert!(actual_tys.len() == points.len());
            let new_temps = actual_tys
                .into_iter()
                .map(|ty| (context.new_temp(loc, ty.clone()), ty))
                .collect::<Vec<_>>();

            let lvalues = new_temps
                .iter()
                .cloned()
                .map(|(v, ty)| sp(loc, H::LValue_::Var(v, Box::new(ty))))
                .collect::<Vec<_>>();
            let assign = sp(loc, H::Command_::Assign(lvalues, Box::new(e)));
            result.push_back(sp(loc, H::Statement_::Command(assign)));

            let exps = new_temps
                .into_iter()
                .zip(points)
                .map(|((var, ty), needs_freeze)| {
                    let e_ = sp(loc, use_tmp(var));
                    let e = H::exp(T::single(ty), e_);
                    if needs_freeze {
                        freeze_point(e)
                    } else {
                        e
                    }
                })
                .collect::<Vec<_>>();
            let ss = exps
                .iter()
                .map(|e| match &e.ty.value {
                    T::Single(s) => s.clone(),
                    _ => panic!("ICE list item has Multple type"),
                })
                .collect::<Vec<_>>();

            let tys = sp(loc, T::Multiple(ss.clone()));
            let items = exps
                .into_iter()
                .zip(ss)
                .map(|(e, s)| H::ExpListItem::Single(e, Box::new(s)))
                .collect();
            H::exp(tys, sp(loc, E::ExpList(items)))
        }
    }
}

fn freeze_point(e: H::Exp) -> H::Exp {
    let frozen_ty = freeze_ty(e.ty.clone());
    let eloc = e.exp.loc;
    let e_ = H::UnannotatedExp_::Freeze(Box::new(e));
    H::exp(frozen_ty, sp(eloc, e_))
}

fn freeze_ty(sp!(tloc, t): H::Type) -> H::Type {
    use H::Type_ as T;
    match t {
        T::Single(s) => sp(tloc, T::Single(freeze_single(s))),
        t => sp(tloc, t),
    }
}

fn freeze_single(sp!(sloc, s): H::SingleType) -> H::SingleType {
    use H::SingleType_ as S;
    match s {
        S::Ref(true, inner) => sp(sloc, S::Ref(false, inner)),
        s => sp(sloc, s),
    }
}

fn bind_for_short_circuit(e: &T::Exp) -> bool {
    use T::UnannotatedExp_ as TE;
    match &e.exp.value {
        TE::Use(_) => panic!("ICE should have been expanded"),
        TE::Value(_)
        | TE::Constant(_, _)
        | TE::Move { .. }
        | TE::Copy { .. }
        | TE::UnresolvedError => false,

        // TODO might want to case ModuleCall for fake natives
        TE::ModuleCall(_) => true,

        TE::Block(seq) => bind_for_short_circuit_sequence(seq),
        TE::Annotate(el, _) => bind_for_short_circuit(el),

        TE::Break
        | TE::Continue
        | TE::IfElse(_, _, _)
        | TE::While(_, _)
        | TE::Loop { .. }
        | TE::Return(_)
        | TE::Abort(_)
        | TE::Builtin(_, _)
        | TE::Dereference(_)
        | TE::UnaryExp(_, _)
        | TE::Borrow(_, _, _)
        | TE::TempBorrow(_, _)
        | TE::BinopExp(_, _, _, _) => true,

        TE::Unit { .. }
        | TE::Spec(_, _)
        | TE::Assign(_, _, _)
        | TE::Mutate(_, _)
        | TE::Pack(_, _, _, _)
        | TE::BorrowLocal(_, _)
        | TE::ExpList(_)
        | TE::Cast(_, _) => panic!("ICE unexpected exp in short circuit check: {:?}", e),
    }
}

fn bind_for_short_circuit_sequence(seq: &T::Sequence) -> bool {
    use T::SequenceItem_ as TItem;
    seq.len() != 1
        || match &seq[0].value {
            TItem::Seq(e) => bind_for_short_circuit(e),
            item @ TItem::Declare(_) | item @ TItem::Bind(_, _, _) => {
                panic!("ICE unexpected item in short circuit check: {:?}", item)
            }
        }
}

//**************************************************************************************************
// Trailing semicolon
//**************************************************************************************************

fn check_trailing_unit(context: &mut Context, block: &mut Block) {
    use H::{Command_ as C, Statement_ as S, UnannotatedExp_ as E};
    macro_rules! hcmd {
        ($loc:pat, $cmd:pat) => {
            sp!(_, S::Command(sp!($loc, $cmd)))
        };
    }
    macro_rules! hignored {
        ($loc:pat, $e:pat) => {
            hcmd!(
                _,
                C::IgnoreAndPop {
                    exp: H::Exp {
                        exp: sp!($loc, $e),
                        ..
                    },
                    ..
                }
            )
        };
    }
    macro_rules! trailing {
        ($uloc: pat) => {
            hcmd!(
                _,
                C::IgnoreAndPop {
                    exp: H::Exp {
                        exp: sp!(
                            $uloc,
                            E::Unit {
                                case: H::UnitCase::Trailing
                            }
                        ),
                        ..
                    },
                    ..
                }
            )
        };
    }
    macro_rules! trailing_returned {
        ($uloc:pat) => {
            hcmd!(
                _,
                C::Return {
                    exp: H::Exp {
                        exp: sp!(
                            $uloc,
                            E::Unit {
                                case: H::UnitCase::Trailing
                            }
                        ),
                        ..
                    },
                    ..
                }
            )
        };
    }
    fn divergent_block(block: &Block) -> bool {
        matches!(
            block.back(),
            Some(hcmd!(_, C::Break))
                | Some(hcmd!(_, C::Continue))
                | Some(hcmd!(_, C::Abort(_)))
                | Some(hcmd!(_, C::Return { .. }))
                | Some(hignored!(_, E::Unreachable))
        )
    }
    macro_rules! invalid_trailing_unit {
        ($context:ident, $loc:expr, $uloc:expr) => {{
            let semi_msg = "Invalid trailing ';'";
            let unreachable_msg = "Any code after this expression will not be reached";
            let info_msg = "A trailing ';' in an expression block implicitly adds a '()' value \
                        after the semicolon. That '()' value will not be reachable";
            $context.env.add_diag(diag!(
                UnusedItem::TrailingSemi,
                ($uloc, semi_msg),
                ($loc, unreachable_msg),
                ($uloc, info_msg),
            ));
            block.pop_back();
        }};
    }

    block
        .iter_mut()
        .for_each(|s| check_trailing_unit_statement(context, s));
    let len = block.len();
    if len < 2 {
        return;
    }
    match (&block[len - 2], &block[len - 1]) {
        (
            sp!(
                loc,
                S::IfElse {
                    if_block,
                    else_block,
                    ..
                }
            ),
            trailing!(uloc),
        )
        | (
            sp!(
                loc,
                S::IfElse {
                    if_block,
                    else_block,
                    ..
                }
            ),
            trailing_returned!(uloc),
        ) if divergent_block(if_block) && divergent_block(else_block) => {
            invalid_trailing_unit!(context, *loc, *uloc)
        }
        (sp!(loc, S::Loop { has_break, .. }), trailing!(uloc))
        | (sp!(loc, S::Loop { has_break, .. }), trailing_returned!(uloc))
            if !has_break =>
        {
            invalid_trailing_unit!(context, *loc, *uloc)
        }
        (hcmd!(loc, C::Break), trailing!(uloc))
        | (hcmd!(loc, C::Break), trailing_returned!(uloc))
        | (hcmd!(loc, C::Continue), trailing!(uloc))
        | (hcmd!(loc, C::Continue), trailing_returned!(uloc))
        | (hcmd!(loc, C::Abort(_)), trailing!(uloc))
        | (hcmd!(loc, C::Abort(_)), trailing_returned!(uloc))
        | (hcmd!(loc, C::Return { .. }), trailing!(uloc))
        | (hcmd!(loc, C::Return { .. }), trailing_returned!(uloc))
        | (hignored!(loc, E::Unreachable), trailing!(uloc))
        | (hignored!(loc, E::Unreachable), trailing_returned!(uloc)) => {
            invalid_trailing_unit!(context, *loc, *uloc)
        }
        _ => (),
    };
}

fn check_trailing_unit_statement(context: &mut Context, sp!(_, s_): &mut H::Statement) {
    use H::Statement_ as S;
    match s_ {
        S::Command(_) => (),
        S::IfElse {
            if_block,
            else_block,
            ..
        } => {
            check_trailing_unit(context, if_block);
            check_trailing_unit(context, else_block)
        }
        S::While {
            cond: (cond_block, _),
            block,
        } => {
            check_trailing_unit(context, cond_block);
            check_trailing_unit(context, block)
        }
        S::Loop { block, .. } => check_trailing_unit(context, block),
    }
}

//**************************************************************************************************
// Unused locals
//**************************************************************************************************

fn check_unused_locals(
    context: &mut Context,
    locals: &mut UniqueMap<Var, H::SingleType>,
    used: BTreeSet<Var>,
) -> BTreeSet<Var> {
    let signature = context
        .signature
        .as_ref()
        .expect("ICE Signature should always be defined when checking a function body");
    let mut unused = BTreeSet::new();
    // report unused locals
    for (v, _) in locals
        .key_cloned_iter()
        .filter(|(v, _)| !used.contains(v) && !v.starts_with_underscore())
    {
        let vstr = match display_var(v.value()) {
            DisplayVar::Tmp => panic!("ICE unused tmp"),
            DisplayVar::Orig(vstr) => vstr,
        };
        let loc = v.loc();
        let msg = if signature.is_parameter(&v) {
            format!(
                "Unused parameter '{0}'. Consider removing or prefixing with an underscore: '_{0}'",
                vstr
            )
        } else {
            // unused local variable; mark for removal
            unused.insert(v);
            format!(
                "Unused local variable '{0}'. Consider removing or prefixing with an underscore: \
                 '_{0}'",
                vstr
            )
        };
        context
            .env
            .add_diag(diag!(UnusedItem::Variable, (loc, msg)));
    }
    for v in &unused {
        locals.remove(v);
    }
    unused
}

fn remove_unused_bindings(unused: &BTreeSet<Var>, block: &mut Block) {
    block
        .iter_mut()
        .for_each(|s| remove_unused_bindings_statement(unused, s))
}

fn remove_unused_bindings_statement(unused: &BTreeSet<Var>, sp!(_, s_): &mut H::Statement) {
    use H::Statement_ as S;
    match s_ {
        S::Command(c) => remove_unused_bindings_command(unused, c),
        S::IfElse {
            if_block,
            else_block,
            ..
        } => {
            remove_unused_bindings(unused, if_block);
            remove_unused_bindings(unused, else_block)
        }
        S::While {
            cond: (cond_block, _),
            block,
        } => {
            remove_unused_bindings(unused, cond_block);
            remove_unused_bindings(unused, block)
        }
        S::Loop { block, .. } => remove_unused_bindings(unused, block),
    }
}

fn remove_unused_bindings_command(unused: &BTreeSet<Var>, sp!(_, c_): &mut H::Command) {
    use H::Command_ as HC;

    if let HC::Assign(ls, _) = c_ {
        remove_unused_bindings_lvalues(unused, ls)
    }
}

fn remove_unused_bindings_lvalues(unused: &BTreeSet<Var>, ls: &mut [H::LValue]) {
    ls.iter_mut()
        .for_each(|l| remove_unused_bindings_lvalue(unused, l))
}

fn remove_unused_bindings_lvalue(unused: &BTreeSet<Var>, sp!(_, l_): &mut H::LValue) {
    use H::LValue_ as HL;
    match l_ {
        HL::Var(v, _) if unused.contains(v) => *l_ = HL::Ignore,
        HL::Var(_, _) | HL::Ignore => (),
        HL::Unpack(_, _, fields) => fields
            .iter_mut()
            .for_each(|(_, l)| remove_unused_bindings_lvalue(unused, l)),
    }
}
