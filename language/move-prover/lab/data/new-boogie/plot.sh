#!/bin/sh
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

FUN_RESULTS="current_boogie.fun_data new_boogie.fun_data"
MOD_RESULTS="current_boogie.mod_data new_boogie.mod_data"

# Plot per function
cargo run -q --release -p prover-lab -- \
    plot --out fun_by_fun.svg --sort ${FUN_RESULTS}

# Plot per module
cargo run -q --release -p prover-lab -- \
    plot --out mod_by_mod.svg --sort ${MOD_RESULTS}
