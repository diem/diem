// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{effects::Action, instance::Instance};
use anyhow::Result;
use futures::future::{BoxFuture, FutureExt};
use slog_scope::info;
use std::fmt;

pub struct DeleteLibraData {
    instance: Instance,
}

impl DeleteLibraData {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }
}

impl Action for DeleteLibraData {
    fn apply(&self) -> BoxFuture<Result<()>> {
        async move {
            info!("DeleteLibraData {}", self.instance);
            self.instance
                .run_cmd(vec!["sudo rm -rf /data/libra/common"])
                .await
        }
        .boxed()
    }
}

impl fmt::Display for DeleteLibraData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DeleteLibraData {}", self.instance)
    }
}
