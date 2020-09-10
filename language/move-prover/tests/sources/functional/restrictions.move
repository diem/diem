// no-boogie-test
module M {

    struct S {
        f: u64
    }

    spec module {
        // Tuples as result type.
        define f1(): (num, num) { (1, 2) }

        // Functions as result type.
        define f2(): | |num { | | 1 }

        // Invoke
        define f3(f: |num|num): num {
            f(1)
        }

        // Lambda outside of all/any
        define f4(): num {
            let f = |x| x + 1;
            1
        }

        // Pack
        define f5(): S {
            S{f: 1}
        }

        // Multiple variable bindings
        // Those aren't supported even in the checker, so commented out to see the other issues.
        // define f9(): (num, num) {
        //    let (x, y) = (1, 2);
        //    (x, y)
        //}

        // Unpack
        // This isn't supported even in the checker, so commented out to see the other issues.
        // define f10(s: S): num {
        //    let S{f:x} = s;
        //    x
        //}
    }
}
