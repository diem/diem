// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides CollectorSerializer. See it's documentation for more help

use std::fmt::Arguments;

use slog::{Key, Result, Serializer};

use crate::kv_categorizer::KVCategorizer;

/// This serializer collects all KV pairs into a Vec, converting the values to `String`.
/// It filters out the one that are of `KVCategory::Ignore`
pub struct CollectorSerializer<'a, C: KVCategorizer>(Vec<(Key, String)>, &'a C);

impl<'a, C: KVCategorizer> CollectorSerializer<'a, C> {
    /// Create a collector serializer that will use the given categorizer to collect desired values
    pub fn new(categorizer: &'a C) -> Self {
        CollectorSerializer(Vec::new(), categorizer)
    }

    /// Once done collecting KV pairs call this to retrieve collected values
    pub fn into_inner(self) -> Vec<(Key, String)> {
        self.0
    }
}

/// Define a macro to implement serializer emit functions.
macro_rules! impl_emit_body(
    ($s:expr, $k:expr, $v:expr) => {
        if $s.1.ignore($k) {
            return Ok(())
        }
        $s.0.push(($k, format!("{}", $v)));
    };
);

/// Define a macro to implement serializer emit functions for standard types.
macro_rules! impl_emit(
    ($name:ident, $t:ty) => {
        /// Emit $t
        fn $name(&mut self, key: Key, val: $t) -> Result {
            impl_emit_body!(self, key, val);
            Ok(())
        }
    };
);

impl<'a, C: KVCategorizer> Serializer for CollectorSerializer<'a, C> {
    /// Emit ()
    fn emit_unit(&mut self, key: Key) -> Result {
        impl_emit_body!(self, key, "()");
        Ok(())
    }

    /// Emit None
    fn emit_none(&mut self, key: Key) -> Result {
        impl_emit_body!(self, key, "None");
        Ok(())
    }

    impl_emit!(emit_usize, usize);
    impl_emit!(emit_isize, isize);
    impl_emit!(emit_bool, bool);
    impl_emit!(emit_char, char);
    impl_emit!(emit_u8, u8);
    impl_emit!(emit_i8, i8);
    impl_emit!(emit_u16, u16);
    impl_emit!(emit_i16, i16);
    impl_emit!(emit_u32, u32);
    impl_emit!(emit_i32, i32);
    impl_emit!(emit_f32, f32);
    impl_emit!(emit_u64, u64);
    impl_emit!(emit_i64, i64);
    impl_emit!(emit_f64, f64);
    impl_emit!(emit_str, &str);
    impl_emit!(emit_arguments, &Arguments<'_>);
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::assert_equal;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use slog::{b, record, Level, Record, Result as SlogResult, KV};

    use crate::kv_categorizer::{InlineCategorizer, KVCategory};

    #[derive(Clone)]
    struct TestKv {
        key: Key,
        vusize: usize,
        visize: isize,
        vbool: bool,
        vchar: char,
        vu8: u8,
        vi8: i8,
        vu16: u16,
        vi16: i16,
        vu32: u32,
        vi32: i32,
        vf32: f32,
        vu64: u64,
        vi64: i64,
        vf64: f64,
        vstr: String,
    }

    impl TestKv {
        fn new<R: Rng>(key: Key, rng: &mut R) -> Self {
            TestKv {
                key,
                vusize: rng.gen(),
                visize: rng.gen(),
                vbool: rng.gen(),
                vchar: rng.gen(),
                vu8: rng.gen(),
                vi8: rng.gen(),
                vu16: rng.gen(),
                vi16: rng.gen(),
                vu32: rng.gen(),
                vi32: rng.gen(),
                vf32: rng.gen(),
                vu64: rng.gen(),
                vi64: rng.gen(),
                vf64: rng.gen(),
                vstr: format!("value{}", rng.gen::<i64>()),
            }
        }

        fn to_vec(&self) -> Vec<(Key, String)> {
            vec![
                (self.key, "None".to_owned()),
                (self.key, "()".to_owned()),
                (self.key, format!("{}", self.vusize)),
                (self.key, format!("{}", self.visize)),
                (self.key, format!("{}", self.vbool)),
                (self.key, format!("{}", self.vchar)),
                (self.key, format!("{}", self.vu8)),
                (self.key, format!("{}", self.vi8)),
                (self.key, format!("{}", self.vu16)),
                (self.key, format!("{}", self.vi16)),
                (self.key, format!("{}", self.vu32)),
                (self.key, format!("{}", self.vi32)),
                (self.key, format!("{}", self.vf32)),
                (self.key, format!("{}", self.vu64)),
                (self.key, format!("{}", self.vi64)),
                (self.key, format!("{}", self.vf64)),
                (self.key, self.vstr.clone()),
            ]
        }
    }

    impl KV for TestKv {
        fn serialize(&self, _record: &Record<'_>, serializer: &mut dyn Serializer) -> SlogResult {
            serializer
                .emit_none(self.key)
                .expect("failure emitting none");
            serializer
                .emit_unit(self.key)
                .expect("failure emitting unit");
            serializer
                .emit_usize(self.key, self.vusize)
                .expect("failure emitting usize");
            serializer
                .emit_isize(self.key, self.visize)
                .expect("failure emitting isize");
            serializer
                .emit_bool(self.key, self.vbool)
                .expect("failure emitting bool");
            serializer
                .emit_char(self.key, self.vchar)
                .expect("failure emitting char");
            serializer
                .emit_u8(self.key, self.vu8)
                .expect("failure emitting u8");
            serializer
                .emit_i8(self.key, self.vi8)
                .expect("failure emitting i8");
            serializer
                .emit_u16(self.key, self.vu16)
                .expect("failure emitting u16");
            serializer
                .emit_i16(self.key, self.vi16)
                .expect("failure emitting i16");
            serializer
                .emit_u32(self.key, self.vu32)
                .expect("failure emitting u32");
            serializer
                .emit_i32(self.key, self.vi32)
                .expect("failure emitting i32");
            serializer
                .emit_f32(self.key, self.vf32)
                .expect("failure emitting f32");
            serializer
                .emit_u64(self.key, self.vu64)
                .expect("failure emitting u64");
            serializer
                .emit_i64(self.key, self.vi64)
                .expect("failure emitting i64");
            serializer
                .emit_f64(self.key, self.vf64)
                .expect("failure emitting f64");
            serializer
                .emit_str(self.key, &self.vstr)
                .expect("failure emitting str");
            Ok(())
        }
    }

    fn do_test<C, V, E>(categorizer: &C, kv_values: V, kv_expected: E)
    where
        C: KVCategorizer,
        V: IntoIterator<Item = TestKv>,
        E: IntoIterator<Item = TestKv>,
    {
        let mut serializer = CollectorSerializer::new(categorizer);

        for value in kv_values {
            value
                .serialize(
                    &record!(Level::Info, "test", &format_args!(""), b!()),
                    &mut serializer,
                )
                .expect("serialize failed!");
        }

        assert_equal(
            serializer.into_inner(),
            kv_expected.into_iter().flat_map(|x| x.to_vec()),
        );
    }

    #[test]
    fn test_inline_all() {
        let mut rng: StdRng = SeedableRng::from_seed([1; 32]);
        let input = vec![
            TestKv::new("test1", &mut rng),
            TestKv::new("test2", &mut rng),
        ];
        do_test(&InlineCategorizer, vec![], vec![]);
        do_test(&InlineCategorizer, input.clone(), input);
    }

    struct TestCategorizer;
    impl KVCategorizer for TestCategorizer {
        fn categorize(&self, _key: Key) -> KVCategory {
            unimplemented!(); // It's not used by serializer
        }

        fn name(&self, _key: Key) -> &'static str {
            unimplemented!(); // It's not used by serializer
        }

        fn ignore(&self, key: Key) -> bool {
            key.starts_with("ignore")
        }
    }

    #[test]
    fn test_ignoring() {
        let mut rng: StdRng = SeedableRng::from_seed([2; 32]);
        let normal = vec![
            TestKv::new("test1", &mut rng),
            TestKv::new("test2", &mut rng),
        ];
        let ignore = vec![
            TestKv::new("ignore1", &mut rng),
            TestKv::new("ignore2", &mut rng),
        ];
        let n = || normal.iter().cloned();
        let i = || ignore.iter().cloned();
        let e = || vec![];

        do_test(&TestCategorizer, e(), e());
        do_test(&TestCategorizer, n(), n());
        do_test(&TestCategorizer, i(), e());
        do_test(&TestCategorizer, n().chain(i()), n());
        do_test(&TestCategorizer, i().chain(n()), n());
    }
}
