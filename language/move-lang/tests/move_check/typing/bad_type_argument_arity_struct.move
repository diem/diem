address 0x42 {
module M {

    struct S<T> has copy, drop { f: T }

    struct B {
        s1: S,
        s2: S<>,
        s3: S<bool, u64>,
    }

    fun foo(
        s1: S,
        s2: S<>,
        s3: S<u64, bool>,
        s4: S<S<u64, bool>>
    ): (S, S<>, S<u64, address>, S<S<u64, u8>>) {
        s1.f;
        s2.f;
        s3.f;
        *&s4.f;
        s4.f.f;

        (s1, s2, s3, s4)
    }

    fun s<T>(f: T): S {
        S { f }
    }

    fun bar(): u64 {
        let s = s(0);
        s.f
    }

}
}
