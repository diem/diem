// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{unique_map::UniqueMap, *};
use std::{cmp::Ordering, fmt::Debug, iter::IntoIterator};

/// Unique set wrapper around `UniqueMap` where the value of the map is not needed
#[derive(Clone)]
pub struct UniqueSet<T: TName>(UniqueMap<T, ()>);

impl<T: TName> UniqueSet<T> {
    pub fn new() -> Self {
        Self(UniqueMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, x: T) -> Result<(), (T, T::Loc)> {
        self.0.add(x, ())
    }

    pub fn contains(&self, x: &T) -> bool {
        self.0.contains_key(x)
    }

    pub fn contains_(&self, x_: &T::Key) -> bool {
        self.0.contains_key_(&x_)
    }

    // interesection of two sets. Keeps the loc of the first set
    pub fn intersect(&self, other: &Self) -> Self {
        let mut intersection = Self::new();
        for x in self.cloned_iter() {
            if !other.contains(&x) {
                continue;
            }
            assert!(intersection.add(x).is_ok());
        }
        intersection
    }

    // union of two sets. Prefers the loc of the first set
    pub fn union(&self, other: &Self) -> Self {
        let mut joined = Self::new();
        for x in self.cloned_iter() {
            assert!(joined.add(x).is_ok());
        }
        for (loc, x_) in other {
            if joined.contains_(x_) {
                continue;
            }
            assert!(joined.add(T::add_loc(loc, x_.clone())).is_ok())
        }
        joined
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.iter().all(|(_, x_)| other.contains_(x_))
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    pub fn cloned_iter(&self) -> impl Iterator<Item = T> {
        self.into_iter()
            .map(|(loc, k_)| T::add_loc(loc, k_.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn from_elements(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self, (T::Key, T::Loc, T::Loc)> {
        Ok(Self(UniqueMap::maybe_from_iter(
            iter.into_iter().map(|x| (x, ())),
        )?))
    }

    pub fn from_elements_(
        loc: T::Loc,
        iter: impl IntoIterator<Item = T::Key>,
    ) -> Result<Self, (T::Key, T::Loc, T::Loc)> {
        Ok(Self(UniqueMap::maybe_from_iter(
            iter.into_iter().map(|x_| (T::add_loc(loc, x_), ())),
        )?))
    }
}

impl<T: TName> PartialEq for UniqueSet<T> {
    fn eq(&self, other: &UniqueSet<T>) -> bool {
        self.0 == other.0
    }
}
impl<T: TName> Eq for UniqueSet<T> {}

impl<T: TName> PartialOrd for UniqueSet<T> {
    fn partial_cmp(&self, other: &UniqueSet<T>) -> Option<Ordering> {
        (self.0).0.keys().partial_cmp((other.0).0.keys())
    }
}

impl<T: TName> Ord for UniqueSet<T> {
    fn cmp(&self, other: &UniqueSet<T>) -> Ordering {
        (self.0).0.keys().cmp((other.0).0.keys())
    }
}

impl<T: TName + Debug> Debug for UniqueSet<T>
where
    T::Key: Debug,
    T::Loc: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: TName> Hash for UniqueSet<T>
where
    T::Key: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for k in (self.0).0.keys() {
            k.hash(state);
        }
    }
}

//**************************************************************************************************
// IntoIter
//**************************************************************************************************

pub struct IntoIter<T: TName>(
    std::iter::Map<unique_map::IntoIter<T, ()>, fn((T, ())) -> T>,
    usize,
);

impl<T: TName> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 > 0 {
            self.1 -= 1;
        }
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.1, Some(self.1))
    }
}

impl<T: TName> IntoIterator for UniqueSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        IntoIter(
            self.0.into_iter().map(|ab_v| {
                let (ab, ()) = ab_v;
                ab
            }),
            len,
        )
    }
}

//**************************************************************************************************
// Iter
//**************************************************************************************************

pub struct Iter<'a, T: TName>(
    std::iter::Map<
        unique_map::Iter<'a, T, ()>,
        fn((T::Loc, &'a T::Key, &'a ())) -> (T::Loc, &'a T::Key),
    >,
    usize,
);

impl<'a, T: TName> Iterator for Iter<'a, T> {
    type Item = (T::Loc, &'a T::Key);

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 > 0 {
            self.1 -= 1;
        }
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.1, Some(self.1))
    }
}

impl<'a, T: TName> IntoIterator for &'a UniqueSet<T> {
    type Item = (T::Loc, &'a T::Key);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        Iter(self.0.iter().map(|(loc, x_, ())| (loc, x_)), len)
    }
}
