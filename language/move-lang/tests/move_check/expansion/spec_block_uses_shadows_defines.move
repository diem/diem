address 0x2 {
module M {
    resource struct R1 {}
    resource struct R2<T> {}

    fun t1(): (R2<u64>, R1) {
        abort 0
    }

    spec module {
        use 0x2::M::{R1 as R, R2 as S};
        // TODO syntax change to move spec heleprs outside of blocks
        define S(): bool { false }
        define R(): bool { false }

        ensures exists<S<u64>>(0x1) == exists<R>(0x1);
    }

    fun t2(): (R2<u64>, R1) {
        abort 0
    }

}
}
