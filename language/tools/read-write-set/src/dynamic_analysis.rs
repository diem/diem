// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ResourceKey, TypeTag},
    resolver::MoveResolver,
};
use move_model::{
    model::{FunctionEnv, GlobalEnv},
    ty::Type,
};
use prover_bytecode::{
    access_path::{AbsAddr, AccessPath, Offset, Root},
    access_path_trie::AccessPathTrie,
    read_write_set_analysis::ReadWriteSetState,
};
use read_write_set_types::Access;
use resource_viewer::{AnnotatedMoveValue, MoveValueAnnotator};
use std::ops::Deref;

/// A read/write set state with no unbound formals or type variables
#[derive(Debug)]
pub struct ConcretizedFormals(AccessPathTrie<Access>);

/// A read/write set state with no secondary indexes and no unbound formals or type variables
#[derive(Debug)]
pub struct ConcretizedSecondaryIndexes(ConcretizedFormals);

impl ConcretizedFormals {
    /// Return the `ResourceKey`'s that may be accessed by `self`. If `is_write` is true, return the
    /// keys that may be written; otherwise, return the keys that may be read.
    /// For example: if `self` is 0x7/0x1::AModule::AResource/f/g -> ReadWrite, this will return
    /// 0x7/0x1::AModule.
    pub fn get_keys_(&self, is_write: bool, env: &GlobalEnv) -> Vec<ResourceKey> {
        self.0
            .iter()
            .flat_map(|(r, node)| {
                let keep = node.get_child_data().map_or(false, |d| {
                    if is_write {
                        d.is_write()
                    } else {
                        d.is_read()
                    }
                });
                if keep {
                    match r {
                        Root::Global(g) => {
                            let type_ = g.struct_type().normalize(env).expect(
                                "Getting type tag should never fail for ConcretizedFormals",
                            );
                            g.address()
                                .get_concrete_addresses()
                                .iter()
                                .map(|a| ResourceKey::new(*a, type_.clone()))
                                .collect()
                        }
                        _ => panic!("Unexpected root type {:?} for ConcretizedFormals", r),
                    }
                } else {
                    vec![]
                }
            })
            .collect()
    }

    /// Return the `ResourceKey`'s written by `self`.
    pub fn get_keys_written(&self, env: &GlobalEnv) -> Vec<ResourceKey> {
        self.get_keys_(true, env)
    }

    /// Return the `ResourceKey`'s read by `self`.
    pub fn get_keys_read(&self, env: &GlobalEnv) -> Vec<ResourceKey> {
        self.get_keys_(false, env)
    }

    /// Concretize all secondary indexes in `self` using `blockchain_view` and return the result. For
    /// example: if `self` is 0x7/0x1::AModule::AResource/addr_field/0x2::M2::R/f -> Write and the
    /// value of 0x7/0x1::AModule::AResource/addr_field is 0xA in `blockchain_view`, this will
    /// return { 0x7/0x1::AModule::AResource/addr_field -> Read, 0xA/0x2::M2::R/f -> Write }
    fn concretize_secondary_indexes(
        self,
        blockchain_view: &impl MoveResolver,
        env: &GlobalEnv,
    ) -> ConcretizedSecondaryIndexes {
        // TODO: check if there are no secondary indexes and return accesses if so
        let annotator = MoveValueAnnotator::new(blockchain_view);
        let mut acc = AccessPathTrie::default();
        // TODO: do not unwrap here. iter_paths doesn't support Result, so need to create an Iter instead
        // of iter_paths or return a bool inside resolve_secondary instead of using ?
        self.0.iter_paths(|access_path, access| {
            Self::concretize_secondary_indexes_(&annotator, access_path, access, env, &mut acc)
                .unwrap();
        });
        ConcretizedSecondaryIndexes(ConcretizedFormals(acc))
    }

    /// Construct a `ConcretizedFormals` from `accesses` by binding the formals and type variables in
    /// `accesses` to `actuals` and `type_actuals`.
    fn from_args_(
        accesses: AccessPathTrie<Access>,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        fun_env: &FunctionEnv,
    ) -> ConcretizedFormals {
        let mut sub_map = AccessPathTrie::default();
        let mut actual_indices = vec![];
        for (i, actual) in actuals.iter().enumerate() {
            sub_map.bind_local(i, actual.clone(), fun_env);
            actual_indices.push(i);
        }
        ConcretizedFormals(ReadWriteSetState::sub_actuals(
            accesses,
            &actual_indices,
            type_actuals,
            fun_env,
            &sub_map,
        ))
    }

    /// Construct a `ConcretizedFormals` from `accesses` by binding the formals and type variables in
    /// `accesses` to `signers`/`actuals` and `type_actuals`.
    /// For example: if `accesses` is Formal(0)/0x1::M::S<TypeVar(0)>/f -> Read, `signers` is 0xA
    /// and type_actuals is 0x2::M2::S2, this will return  0xA/0x1::M::S<0x2::M2::S@>/f -> Read
    pub fn from_args(
        accesses: AccessPathTrie<Access>,
        signers: &[AccountAddress],
        actuals: &[Vec<u8>],
        type_actuals: &[TypeTag],
        fun_env: &FunctionEnv,
    ) -> Result<ConcretizedFormals> {
        let mut new_actuals = signers.iter().map(AbsAddr::from).collect::<Vec<AbsAddr>>();
        let formal_types = fun_env.get_parameter_types();
        assert_eq!(
            formal_types.len(),
            actuals.len() + signers.len(),
            "Formal/actual arity mismatch"
        );
        assert_eq!(
            fun_env.get_type_parameter_count(),
            type_actuals.len(),
            "Type formal/type actual arity mismatch"
        );
        // The analysis abstracts addresses. Deserialize the address actuals, use an empty AbsAddr for
        // the rest
        let num_signers = signers.len();
        // formal_types includes all formal types (including signers), but actuals is only
        // the non-signer actuals.
        for (actual_index, ty) in formal_types[num_signers..].iter().enumerate() {
            let actual = if ty.is_address() {
                AbsAddr::from(&AccountAddress::from_bytes(&actuals[actual_index])?)
            } else {
                AbsAddr::default()
            };
            new_actuals.push(actual);
        }

        let env = fun_env.module_env.env;
        let new_type_actuals = type_actuals
            .iter()
            .map(|t| Type::from_type_tag(t, env))
            .collect::<Vec<Type>>();
        Ok(Self::from_args_(
            accesses,
            &new_actuals,
            &new_type_actuals,
            fun_env,
        ))
    }

    /// Concretize the secondary in `offsets` -> `access` using `annotator` and add the results to
    /// `acc`.
    fn concretize_offsets<T: MoveResolver>(
        annotator: &MoveValueAnnotator<T>,
        access_path: AccessPath,
        mut next_value: AnnotatedMoveValue,
        next_offset_index: usize,
        access: &Access,
        env: &GlobalEnv,
        acc: &mut AccessPathTrie<Access>,
    ) -> Result<()> {
        let offsets = access_path.offsets();
        for next_offset_index in next_offset_index..offsets.len() {
            let next_offset = &offsets[next_offset_index];
            match next_offset {
                Offset::Field(index) => {
                    if let AnnotatedMoveValue::Struct(s) = next_value {
                        let fields = s.value;
                        next_value = fields[*index].1.clone();
                    } else {
                        panic!("Malformed access path {:?}; expected struct value as prefix to field offset {:?}, but got {:?}",
                               access_path, next_offset, next_value)
                    }
                }
                Offset::Global(g_offset) => {
                    // secondary index. previous offset should have been an address value
                    if let AnnotatedMoveValue::Address(a) = next_value {
                        let mut new_ap = AccessPath::new_global_constant(
                            move_model::addr_to_big_uint(&a),
                            g_offset.clone(),
                        );
                        for o in offsets[next_offset_index + 1..].iter() {
                            new_ap.add_offset(o.clone())
                        }
                        return Self::concretize_secondary_indexes_(
                            annotator, &new_ap, access, env, acc,
                        );
                    } else {
                        panic!(
                            "Malformed access path {:?}: expected address value before Global offset, but found {:?}",
                            access_path,
                            next_value
                        );
                    }
                }
                Offset::VectorIndex => {
                    if let AnnotatedMoveValue::Vector(_v_type, v_contents) = &next_value {
                        // concretize offsets for each element in the vector
                        for val in v_contents {
                            Self::concretize_offsets(
                                annotator,
                                access_path.clone(),
                                val.clone(),
                                next_offset_index + 1,
                                access,
                                env,
                                acc,
                            )?;
                        }
                        return Ok(());
                    } else {
                        panic!(
                            "Malformed access path {:?}: expected vector value before VectorIndex offset, but found {:?}",
                            access_path,
                            next_value
                        );
                    }
                }
            }
        }
        assert!(
            access_path.all_addresses_types_bound(),
            "Failed to concretize secondary indexes in ap {:?}",
            access_path
        );
        // Got to the end of the concrete access path. Add it to the accumulator
        acc.update_access_path_weak(access_path, Some(*access));

        Ok(())
    }

    /// Concretize the secondary indexes in `access_path` and add the result to `acc`. For example
    fn concretize_secondary_indexes_<T: MoveResolver>(
        annotator: &MoveValueAnnotator<T>,
        access_path: &AccessPath,
        access: &Access,
        env: &GlobalEnv,
        acc: &mut AccessPathTrie<Access>,
    ) -> Result<()> {
        if let Root::Global(g) = access_path.root() {
            assert!(
                g.is_statically_known(),
                "Attempting to extract concrete addresses from unknown global {:?}",
                g
            );
            let addrs = g.address().get_concrete_addresses();
            assert!(!addrs.is_empty()); // TODO: enforce this in AbsAddr constructor instead
            let tag = g.struct_type().get_type_tag(env).unwrap(); // safe because we checked g.is_statically_known
            for addr in &addrs {
                if let Some(resource_bytes) = annotator.get_resource_bytes(addr, &tag) {
                    let mut ap = AccessPath::new_global_constant(
                        move_model::addr_to_big_uint(addr),
                        g.struct_type().clone(),
                    );
                    for o in access_path.offsets() {
                        ap.add_offset(o.clone())
                    }
                    let resource = annotator.view_resource(&tag, &resource_bytes)?;
                    Self::concretize_offsets(
                        annotator,
                        ap,
                        AnnotatedMoveValue::Struct(resource),
                        0,
                        access,
                        env,
                        acc,
                    )?;
                } // else, resource not present. this can happen/is expected because the R/W set is overapproximate
            }
        } else {
            panic!(
                "Malformed access path {:?}: Bad root type {:?}. This root cannot be concretized by reading global state",
                access_path,
                access_path.root()
            )
        }
        Ok(())
    }
}

/// Bind all formals and type variables in `accesses` using `signers`, `actuals`, and
/// `type_actuals`. In addition, concretize all secondary indexes in `accesses` against the state in
/// `blockchain_view`.
pub fn concretize(
    accesses: AccessPathTrie<Access>,
    signers: &[AccountAddress],
    actuals: &[Vec<u8>],
    type_actuals: &[TypeTag],
    fun_env: &FunctionEnv,
    blockchain_view: &impl MoveResolver,
) -> Result<ConcretizedSecondaryIndexes> {
    let concretized_formals =
        ConcretizedFormals::from_args(accesses, signers, actuals, type_actuals, fun_env)?;
    Ok(concretized_formals.concretize_secondary_indexes(blockchain_view, fun_env.module_env.env))
}

impl Deref for ConcretizedFormals {
    type Target = AccessPathTrie<Access>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ConcretizedSecondaryIndexes {
    type Target = ConcretizedFormals;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
