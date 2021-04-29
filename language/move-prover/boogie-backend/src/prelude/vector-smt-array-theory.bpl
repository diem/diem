// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Boogie model for vectors, based on smt arrays.

// This version of vectors requires boogie to be called with `-useArrayTheory`.
// It is not extensional.

// Currently we just include the basic vector array theory.

{% include "vector-array-theory" %}
