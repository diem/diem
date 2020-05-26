address 0x1 {
module M {
    resource struct S {}
    resource struct R<T> {}

    fun t1(): (R<u64>, S) {
        abort 0
    }

    spec module {
        use 0x1::M::{S as R, R as S};
        ensures exists<S<u64>>(0x1) == exists<R>(0x1);
    }

    fun t2(): (R<u64>, S) {
        abort 0
    }

}
}
