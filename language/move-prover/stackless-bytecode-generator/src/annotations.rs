// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};

/// A container for an extensible, dynamically typed set of annotations.
#[derive(Debug, Default)]
pub struct Annotations {
    map: BTreeMap<TypeId, Box<dyn Any>>,
}

impl Annotations {
    /// Tests whether annotation of type T is present.
    pub fn has<T: Any>(&self) -> bool {
        let id = TypeId::of::<T>();
        self.map.contains_key(&id)
    }

    /// Gets annotation of type T.
    pub fn get<T: Any>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.map.get(&id).and_then(|d| d.downcast_ref::<T>())
    }

    /// Sets annotation of type T.
    pub fn set<T: Any>(&mut self, x: T) {
        let id = TypeId::of::<T>();
        self.map.insert(id, Box::new(x));
    }
}
