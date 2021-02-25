address 0x42 {
module M {

    struct S<T> has copy, drop { f: T }

    const S1: S = S { f: 0 };
    const S2: S<> = S { f: 0 };
    const S3: S<u64, bool> = S { f: 0 };
    const S4: S<S<u64, bool>> = S { f: S { f: 0 } };

    fun t() {
        S1.f;
        S2.f;
        S3.f;
        *&S4.f;
        S4.f.f;
    }

}
}
