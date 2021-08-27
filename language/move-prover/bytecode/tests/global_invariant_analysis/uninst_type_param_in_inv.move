module 0x42::Demo {
    struct S1<T: store> has key, store {
        t: T,
        v: u8,
    }

    struct S2<T: store> has key, store {
        t: T,
        v: u8,
    }

    fun f1(addr: address) acquires S1 {
        if (exists<S1<bool>>(addr)) {
            *&mut borrow_global_mut<S1<bool>>(addr).v = 0;
        };

        if (exists<S1<u64>>(addr)) {
            *&mut borrow_global_mut<S1<u64>>(addr).v = 0;
        };
    }

    spec module {
        invariant<T1, T2>
            (exists<S1<T1>>(@0x2) && exists<S2<T2>>(@0x2))
                ==> global<S1<T1>>(@0x2).v == 0;
    }
}
