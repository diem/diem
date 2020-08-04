module TestEnsuresFalseSmokeTest {
    spec module {
        pragma verify = false;
        pragma writeref_test = true;
    }

    public fun foo(x: u64): u64 {
        x
    }
    spec fun foo {
        ensures result == x;
    }

    resource struct T {
        x: u64,
        y: u64,
    }

    public fun mod_T_x(addr: address)
    acquires T
    {
        let t = borrow_global_mut<T>(addr);
        let x = &mut t.x;
        *x = 0;
        let y = &mut t.y;
        *y = 10;
    }

    spec fun mod_T_x {
        ensures global<T>(addr).x == 0;
        // ensures global<T>(addr).y == 10;
        // ensures true;
    }
}
