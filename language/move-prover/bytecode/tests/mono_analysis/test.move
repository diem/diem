address 0x123 {
module Test {

    struct A<T1, T2> {
        x1: T1,
        x2: T2,
    }

    struct B<T1> {
        x1: A<T1, u64>,
    }

    public fun f1<T>(x1: T): A<T, u64> {
        A{x1, x2: 10}
    }

    public fun f2(x: u8): B<u8> {
        B{x1: f1(x)}
    }

    public fun f3<T>(x1: T): A<T, u64> {
        A{x1, x2: 1}
    }
    spec f3 {
        pragma opaque = true;
    }

    public fun f4<T>(x1: T): B<T> {
        B{x1: f3(x1)}
    }

    public fun f5(): B<u128> {
        f4(1)
    }

}
}
