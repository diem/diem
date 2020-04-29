// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Note: If this test file fails to run, it is possible that the
// staged version of the Move stdlib needs to be updated. This code
// is compiled with the latest compiler and stdlib, but it runs with
// the staged stdlib.

address {{sender}} {

module MyModule {
    use 0x0::Libra;
    use 0x0::LBR;

    // The identity function for coins: takes a Libra::T<LBR::T> as input and hands it back
    public fun id(c: Libra::T<LBR::T>): Libra::T<LBR::T> {
        c
    }
}

}
