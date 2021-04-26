address 0x2 {
module M {
    struct S1 has drop { b: bool }
    struct S2 has drop { u: u64 }
    struct S3 has drop { a: address }
    fun check(): bool {
        false
    }
    fun num(u: u64): u64 {
        u + 1
    }
    spec schema Foo<T> {
        ensures true;
    }

    fun t<T>(): S3 {
        use 0x2::M::{check as num, num as t, S1 as S2, S2 as Foo, S3 as T};
        num() && true;
        t(0) + 1;
        S2 { b: false };
        Foo { u: 0 };
        T { a: @0x0 }
    }

    fun t2<T>(x: T) {
        {
            use 0x2::M::{check as num, num as t, S1 as S2, S2 as Foo, S3 as T};
            num() && true;
            t(0) + 1;
            S2 { b: false };
            Foo { u: 0 };
            T { a: @0x0 }
        };
        check() && true;
        num(0) + 1;
        t2<T>(x);
        S1 { b: false };
        S2 { u: 0 };
        S3 { a: @0x0 };
    }
}
}
