// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::{Exp, Value_},
    macros::r#macro::{ExpansionBuilder, MacroProcessor, MemberName},
    parser::ast::BinOp_,
    shared::CompilationEnv,
};

pub fn register_macros(env: &mut CompilationEnv) {
    env.register_macro(
        "Std",
        "Vector",
        "foreach",
        Box::new(ForeachProcessor { mutable: false }),
    );
    env.register_macro(
        "Std",
        "Vector",
        "foreach_mut",
        Box::new(ForeachProcessor { mutable: true }),
    )
}

struct ForeachProcessor {
    mutable: bool,
}

impl MacroProcessor for ForeachProcessor {
    fn expand(&self, b: &mut ExpansionBuilder, args: Vec<Exp>) -> Exp {
        // Vector::foreach[_mut]!(vec, |var| body)
        // --->
        // {
        //   let _i = 0;
        //   while (_i < Vector::length(vec)) {
        //     let var = Vector::borrow[_mut](vec, _i);
        //     <<body>>;
        //     _i = _i + 1;
        //   }
        // }
        let (vec, var, body) = match b.extract_args2(args) {
            Some((a1, a2)) => match b.extract_lambda1(a2) {
                Some((var, body)) => (a1, var, body),
                None => return b.error(),
            },
            None => return b.error(),
        };

        let length_fun = MemberName::create("Std", "Vector", "length");
        let borrow_fun = MemberName::create(
            "Std",
            "Vector",
            if self.mutable { "borrow_mut" } else { "borrow" },
        );
        let index = b.new_local();
        b.block(
            vec![(index, b.value(Value_::U64(0)))],
            vec![b.while_(
                b.binary(
                    BinOp_::Lt,
                    b.local(index),
                    b.call(length_fun, vec![vec.clone()]),
                ),
                b.block(
                    vec![(var, b.call(borrow_fun, vec![vec, b.local(index)]))],
                    vec![body, b.increment(index)],
                ),
            )],
        )
    }
}
