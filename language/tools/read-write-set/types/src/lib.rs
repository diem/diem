// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod access;

pub use access::Access;

use move_binary_format::normalized::Type;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ResourceKey, TypeTag},
};
use std::{
    collections::btree_map::BTreeMap,
    fmt::{self, Formatter},
};

/// Offset of an access path: either a field, vector index, or global key
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Offset {
    /// Index into contents of a struct by field offset
    Field(usize),
    /// Unknown index into a vector
    VectorIndex,
    /// A type index into global storage. Only follows a field or vector index of type address
    Global(Type),
}

#[derive(Debug, Clone)]
pub struct TrieNode {
    /// Optional data associated with the parent in the trie
    data: Option<Access>,
    /// Child pointers labeled by offsets
    children: BTreeMap<Offset, TrieNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Root {
    Const(AccountAddress),
    Formal(usize),
}

#[derive(Debug, Clone)]
pub struct AccessPath {
    pub root: Root,
    pub offsets: Vec<Offset>,
}

#[derive(Debug, Clone)]
pub struct ReadWriteSet(BTreeMap<Root, TrieNode>);

impl AccessPath {
    pub fn offset(&self) -> &[Offset] {
        self.offsets.as_slice()
    }
    pub fn root(&self) -> &Root {
        &self.root
    }
    pub fn add_offset(&mut self, offset: Offset) {
        self.offsets.push(offset)
    }
    pub fn new_global_constant(addr: AccountAddress, ty: Type) -> Self {
        Self {
            root: Root::Const(addr),
            offsets: vec![Offset::Global(ty)],
        }
    }
    pub fn has_secondary_index(&self) -> bool {
        self.offsets.iter().skip(1).any(|offset| match offset {
            Offset::Global(_) => true,
            Offset::Field(_) | Offset::VectorIndex => false,
        })
    }

    pub fn to_resource_key(&self) -> Option<ResourceKey> {
        if self.offsets.is_empty() || self.has_secondary_index() {
            return None;
        }
        if let (Root::Const(addr), Some(Offset::Global(ty))) = (&self.root, self.offsets.first()) {
            Some(ResourceKey::new(*addr, ty.clone().into_struct_tag()?))
        } else {
            None
        }
    }
}

impl Offset {
    fn sub_type_actuals(&self, type_actuals: &[Type]) -> Self {
        match self {
            Offset::Global(s) => Offset::Global(s.subst(type_actuals)),
            Offset::Field(_) | Offset::VectorIndex => self.clone(),
        }
    }
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            data: None,
            children: BTreeMap::new(),
        }
    }

    fn iter_paths_opt<F>(&self, access_path: &AccessPath, f: &mut F)
    where
        F: FnMut(&AccessPath, &Access),
    {
        if let Some(access) = &self.data {
            f(access_path, access);
        }
        for (k, v) in self.children.iter() {
            let mut new_ap = access_path.clone();
            new_ap.offsets.push(k.clone());
            v.iter_paths_opt(&new_ap, f)
        }
    }

    fn sub_type_actuals(&self, type_actuals: &[Type]) -> Self {
        Self {
            data: self.data,
            children: self
                .children
                .iter()
                .map(|(offset, node)| {
                    (
                        offset.sub_type_actuals(type_actuals),
                        node.sub_type_actuals(type_actuals),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }
}

impl ReadWriteSet {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add_access_path(&mut self, access_path: AccessPath, access: Access) {
        let mut node = self.0.entry(access_path.root).or_insert_with(TrieNode::new);
        for offset in access_path.offsets {
            node = node.children.entry(offset).or_insert_with(TrieNode::new);
        }
        node.data = Some(access);
    }

    fn iter_paths_impl<F>(&self, mut f: F) -> Option<()>
    where
        F: FnMut(&AccessPath, &Access) -> Option<()>,
    {
        let mut result = Some(());
        for (key, node) in self.0.iter() {
            let access_path = AccessPath {
                root: key.clone(),
                offsets: vec![],
            };
            node.iter_paths_opt(&access_path, &mut |access_path, access| {
                if result.is_none() {
                    return;
                }
                result = f(access_path, access);
            });
        }
        result
    }

    pub fn iter_paths<F>(&self, f: F) -> Option<()>
    where
        F: FnMut(&AccessPath, &Access) -> Option<()>,
    {
        self.iter_paths_impl(f)
    }

    pub fn sub_actuals(
        &self,
        actuals: &[Option<AccountAddress>],
        type_actuals: &[TypeTag],
    ) -> Self {
        let types: Vec<_> = type_actuals
            .iter()
            .map(|ty| Type::from(ty.clone()))
            .collect();
        Self(
            self.0
                .iter()
                .map(|(root, node)| {
                    let root = match root {
                        Root::Const(addr) => Root::Const(*addr),
                        Root::Formal(i) => {
                            if let Some(addr) = actuals.get(*i).and_then(|v| *v) {
                                Root::Const(addr)
                            } else {
                                Root::Formal(*i)
                            }
                        }
                    };
                    (root, node.sub_type_actuals(&types))
                })
                .collect::<BTreeMap<_, _>>(),
        )
    }
}

impl fmt::Display for Root {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Root::Const(addr) => write!(f, "0x{}", addr.short_str_lossless()),
            Root::Formal(i) => write!(f, "Formal({})", i),
        }
    }
}
impl fmt::Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Offset::Global(ty) => write!(f, "{}", ty),
            Offset::VectorIndex => write!(f, "[_]"),
            Offset::Field(i) => write!(f, "{:?}", i),
        }
    }
}
impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)?;
        for offset in &self.offsets {
            f.write_str("/")?;
            write!(f, "{}", offset)?;
        }
        Ok(())
    }
}

impl fmt::Display for ReadWriteSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.iter_paths(|path, v| writeln!(f, "{}: {:?}", path, v).ok());
        Ok(())
    }
}
