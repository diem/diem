// no-boogie-test
module 0x42::M {
    struct S {
        f: u64
    }
    spec module {
        // Tuples as result type.
        fun f1(): (u64, u64) { (1u64, 2u64) }

        // Functions as result type.
        fun f2(): | |num { | | 1 }

        // Invoke
        fun f3(f: |u64|u64): u64 {
            f(1u64)
        }

        // Lambda outside of all/any
        fun f4(): u64 {
            let f = |x| x + 1;
            1
        }

        // Pack
        fun f5(): S {
            S{f: 1}
        }

        // Multiple variable bindings
        // Those aren't supported even in the checker, so commented out to see the other issues.
        // fun f9(): (num, num) {
        //    let (x, y) = (1, 2);
        //    (x, y)
        //}

        // Unpack
        // This isn't supported even in the checker, so commented out to see the other issues.
        // fun f10(s: S): num {
        //    let S{f:x} = s;
        //    x
        //}
    }

    /// Need to use the specctions, otherwise the monomorphizer will eliminate them.
    fun use_them(): bool { true }
    spec use_them {
        ensures f1() == f1();
        ensures f2() == f2();
        ensures f3(|x|x) == f3(|x|x);
        ensures f4() == f4();
        ensures f5() == f5();
    }
}
