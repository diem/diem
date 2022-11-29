#!/bin/sh
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

FUN_RESULTS="boogie_array.fun_data boogie_array_intern.fun_data smt_array.fun_data smt_array_ext.fun_data smt_seq.fun_data"
MOD_RESULTS="boogie_array.mod_data boogie_array_intern.mod_data smt_array.mod_data smt_array_ext.mod_data smt_seq.mod_data"

# Plot per function
cargo run -q --release -p prover-lab -- \
    plot --out fun_by_fun.svg --sort ${FUN_RESULTS}

# Plot per module
cargo run -q --release -p prover-lab -- \
    plot --out mod_by_mod.svg --sort ${MOD_RESULTS}
