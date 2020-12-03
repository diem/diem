// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides definitions of tag types to be used by MIRAI analyzing diem-crypto.
//! This module gets compiled only if the diem-crypto is compiled via MIRAI in a debug build.

use mirai_annotations::*;

/// A MIRAI tag type that tracks if a public key is checked to protect against invalid point
/// attacks, small subgroup attacks, and typos. This tag type is only used at compilation time.
/// This type should only be accessible inside diem-crypto.
pub type ValidatedPublicKeyTag = ValidatedPublicKeyTagKind<VALIDATED_PUBLIC_KEY_TAG_MASK>;

/// A generic tag type intended to only be used by ValidatedPublicKeyTag
pub struct ValidatedPublicKeyTagKind<const MASK: TagPropagationSet> {}

/// The propagation set of ValidatedPublicKeyTag. An empty propagation set is used to make sure that
/// ValidatedPublicKeyTag can only be explicitly attached to public keys.
const VALIDATED_PUBLIC_KEY_TAG_MASK: TagPropagationSet = tag_propagation_set!();
