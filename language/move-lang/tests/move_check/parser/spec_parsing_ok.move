// These are some basic parsing tests for specifications which are expected to succeed.
// Full testing of spec parsing correctness is done outside of this crate.
//
// Note that even though we only test parsing, we still need to ensure that the move code (not the specification)
// is type checking correctly, because with no parsing error, the test harness
// will run subsequent phases of the move-lang compiler.
//
// For parse failures, see the `spec_*_fail.move` test cases.

module M {
    spec module {
        use 0x0::Vector;
        global expected_coin_sum: u64;
        global other: bool;

        native define all<T>(x: SomeCollection<T>, predicate: |T|bool): bool;

        define spec_fun_using_spec_constructs(x: u64, y: u64) : u64 {
            // This function would crash in phases after expansion if we would pass it on as a regular function. Testing
            // here that we don't pass it on.
            let _ = |z| z + 1;
            let _ = x .. y;
            x
        }

    }

    resource struct T{x: u64}
    resource struct R{x: u64}

    struct SomeCoin {
        x : u64,
        y: u64,
    }

    spec struct SomeCoin {
        // Data invariants
        invariant x > 0;
        invariant x == y;
        invariant update old(y) < x;  // this weird coin only allows to increase it's value

        // Tracking the sum of all coins alive.
        invariant update expected_coin_sum = expected_coin_sum - old(x) + x;
        invariant pack expected_coin_sum = expected_coin_sum + x;
        invariant unpack expected_coin_sum = expected_coin_sum - x;
    }

    spec fun with_aborts_if {
      aborts_if x == 0;
    }
    fun with_aborts_if(x: u64): u64 {
        x
    }

    spec fun with_ensures {
        ensures RET == x + 1;
    }
    fun with_ensures(x: u64): u64 {
        x + 1
    }

    spec fun with_multiple_conditions_and_acquires {
        aborts_if y == 0;
        aborts_if 0 == y;
        ensures RET == x/y;
        ensures x/y == RET;
    }

    fun with_multiple_conditions_and_acquires(addr: address, x: u64, y: u64): u64
    acquires T, R {
        let _ = borrow_global_mut<T>(addr);
        let _ = borrow_global_mut<R>(addr);
        x / y
    }

    spec fun using_block {
        ensures RET = {let y = x; y + 1};
    }
    fun using_block(x: u64): u64 {
        x + 1
    }

    spec fun using_lambda
    {
        ensures all(x, |y, z| x + y + z);
    }
    fun using_lambda(x: u64): u64 {
        x
    }

    spec fun using_index_and_range {
        ensures RET = x[1] && x[0..3];
    }
    fun using_index_and_range(x: u64): u64 {
        x
    }

    spec fun using_implies {
        ensures x > 0 ==> RET == x - 1;
        ensures x == 0 ==> RET == x;
    }
    fun using_implies(x: u64): u64 {
        x
    }

    spec module {
        global generic<T>: u64;
        invariant update generic<u64> = 23;
        invariant update Self::generic<u64> = 24;
    }

    fun some_generic<T>() {
    }
    spec fun some_generic {
        ensures generic<T> == 1;
        ensures Self::generic<T> == 1;
    }

    spec schema ModuleInvariant<X, Y> {
        requires global<X>(0x0).f == global<X>(0x1).f;
        ensures global<X>(0x0).f == global<X>(0x1).f;
    }

    spec fun some_generic {
        include ModuleInvariant<T, T>;
    }

    spec module {
        apply ModuleInvariant<X, Y> to *foo*<Y, X>;
        apply ModuleInvariant<X, Y> to *foo*<Y, X>, bar except public *, internal baz<X>;
        pragma do_not_verify, timeout = 60;
    }
}
