// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use handlebars::{
    to_json, BlockContext, Context, Handlebars, Helper, HelperDef, HelperResult, Output,
    RenderContext, RenderError, Renderable,
};

#[derive(Clone, Copy)]
pub struct StratificationHelper {
    depth: usize,
}

impl StratificationHelper {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

impl HelperDef for StratificationHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if !h.params().is_empty() {
            return Err(RenderError::new("helper cannot have parameters"));
        }
        let templ = h
            .template()
            .ok_or_else(|| RenderError::new("expected block template"))?;
        let inverse = h
            .inverse()
            .ok_or_else(|| RenderError::new("expected else template"))?;

        for i in 0..self.depth {
            let this_suffix = if i == 0 {
                "stratified".to_owned()
            } else {
                format!("level{}", i)
            };
            let next_suffix = format!("level{}", i + 1);
            rc.push_block(BlockContext::new());
            let block = rc.block_mut().expect("block defined");
            block.set_local_var("@this_suffix".to_owned(), to_json(this_suffix));
            block.set_local_var("@next_suffix".to_owned(), to_json(next_suffix));
            block.set_local_var("@this_level".to_owned(), to_json(i));
            if i == self.depth - 1 {
                inverse.render(r, ctx, rc, out)?;
            } else {
                templ.render(r, ctx, rc, out)?;
            }
        }
        Ok(())
    }
}
