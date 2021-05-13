// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Note: If this test file fails to run, it is possible that the
// compiled version of the Move stdlib needs to be updated. This code
// is compiled with the latest compiler and stdlib, but it runs with
// the compiled stdlib.

address {{sender}} {

module MyModule {
    use DiemFramework::Diem::Diem;

    // The identity function for coins: takes a Diem<T> as input and hands it back
    public fun id<T>(c: Diem<T>): Diem<T> {
        c
    }
}

}
