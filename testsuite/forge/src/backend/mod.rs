// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod local;
pub use local::{LocalNode, *};

mod k8s;
pub use k8s::{K8sNode, *};
