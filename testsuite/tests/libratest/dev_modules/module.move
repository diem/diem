// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

address {{sender}}:

module MyModule {
    use 0x0::Libra;
    use 0x0::LBR;

    // The identity function for coins: takes a Libra::T<LBR::T> as input and hands it back
    public fun id(c: Libra::T<LBR::T>): Libra::T<LBR::T> {
        c
    }
}
