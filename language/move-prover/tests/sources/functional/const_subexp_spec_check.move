module TestAbortsIf {
    spec module {
        pragma const_sub_exp = true;
    }

    resource struct T {
        x: u64,
    }

    public fun foo(x: u64): u64 {
        x
    }

    spec fun foo {
        // requires x < 5;
        // requires true;
        // requires true ==> x < 5;
        // requires (x > 5 && x < 5) ==> x < 5;
    }

    public fun bar(x: u64): u64 {
        x + x
    }

    // spec fun bar {
    //     requires x < 5;
    //     ensures result < 10;
    //     ensures x < 3 ==> result < 6;
    //     ensures x < 3 ==> result < 0;
    // }

    public fun zar(x: u64): u64 {
        x + x
    }

    spec fun zar {
        requires x < 5;
        requires x > 3;
        requires x != 4;    // assume false
        requires x == 4;    // asserts true
        // ensures result == x + x;
    }
}