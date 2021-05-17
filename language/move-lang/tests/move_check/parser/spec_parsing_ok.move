// These are some basic parsing tests for specifications which are expected to succeed.
// Full testing of spec parsing correctness is done outside of this crate.
//
// Note that even though we only test parsing, we still need to ensure that the move code (not the specification)
// is type checking correctly, because with no parsing error, the test harness
// will run subsequent phases of the move-lang compiler.
//
// For parse failures, see the `spec_*_fail.move` test cases.

module 0x8675309::M {
    spec module {
        global expected_coin_sum: u64;
        global other: bool;

        native fun all<T>(x: SomeCollection<T>, predicate: |T|bool): bool;

        fun spec_fun_using_spec_constructs(x: u64, y: u64) : u64 {
            // This function would crash in phases after expansion if we would pass it on as a regular function. Testing
            // here that we don't pass it on.
            let _ = |z| z + 1;
            let _ = x .. y;
            x
        }

    }

    struct T has key {x: u64}
    struct R has key {x: u64}

    struct SomeCoin {
        x : u64,
        y: u64,
    }

    spec SomeCoin {
        // Data invariants
        invariant x > 0;
        invariant x == y;
    }

    spec with_aborts_if {
      aborts_if x == 0;
    }
    fun with_aborts_if(x: u64): u64 {
        x
    }

    spec with_ensures {
        ensures RET == x + 1;
    }
    fun with_ensures(x: u64): u64 {
        x + 1
    }

    spec with_multiple_conditions_and_acquires {
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

    spec using_block {
        ensures RET = {let y = x; y + 1};
    }
    fun using_block(x: u64): u64 {
        x + 1
    }

    spec using_lambda
    {
        ensures all(x, |y, z| x + y + z);
    }
    fun using_lambda(x: u64): u64 {
        x
    }

    spec using_index_and_range {
        ensures RET = x[1] && x[0..3];
    }
    fun using_index_and_range(x: u64): u64 {
        x
    }

    spec using_implies {
        ensures x > 0 ==> RET == x - 1;
        ensures x == 0 ==> RET == x;
    }
    fun using_implies(x: u64): u64 {
        x
    }

    spec with_emits {
        emits _msg to _guid;
        emits _msg to _guid if true;
        emits _msg to _guid if x > 7;
    }
    fun with_emits<T: drop>(_guid: vector<u8>, _msg: T, x: u64): u64 {
        x
    }

    spec module {
        global x: u64;
        local y: u64;
        z: u64;
        global generic<T>: u64;
        invariant update generic<u64> = 23;
        invariant update Self::generic<u64> = 24;
    }

    fun some_generic<T>() {
    }
    spec some_generic {
        ensures generic<T> == 1;
        ensures Self::generic<T> == 1;
    }

    spec schema ModuleInvariant<X, Y> {
        requires global<X>(0x0).f == global<X>(0x1).f;
        ensures global<X>(0x0).f == global<X>(0x1).f;
    }

    spec some_generic {
        include ModuleInvariant<T, T>{foo:bar, x:y};
    }

    spec module {
        apply ModuleInvariant<X, Y> to *foo*<Y, X>;
        apply ModuleInvariant<X, Y> to *foo*<Y, X>, bar except public *, internal baz<X>;
        pragma do_not_verify, timeout = 60;
    }

    spec module {
        invariant forall x: num, y: num, z: num : x == y && y == z ==> x == z;
        invariant forall x: num : exists y: num : y >= x;
        invariant exists x in 1..10, y in 8..12 : x == y;
    }
}
