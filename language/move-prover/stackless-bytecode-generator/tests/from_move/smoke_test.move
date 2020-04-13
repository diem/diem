// This module contains just some arbitrary code to smoke test the basic functionality of translation from Move
// to stackless bytecode. Coverage for byte code translation is achieved by many more tests in the prover.

// dep: ../tests/sources/stdlib/modules/transaction.move

module SmokeTest {
    use 0x0::Transaction;

    // -----------------
    // Basic Ops
    // -----------------

	fun bool_ops(a: u64, b: u64): (bool, bool) {
        let c: bool;
        let d: bool;
        c = a > b && a >= b;
        d = a < b || a <= b;
        if (!(c != d)) abort 42;
        (c, d)
    }

	fun arithmetic_ops(a: u64): (u64, u64) {
        let c: u64;
        c = (6 + 4 - 1) * 2 / 3 % 4;
        if (c != 2) abort 42;
        (c, a)
    }

    // -----------------
    // Resources
    // -----------------

    resource struct R {
        x: u64
    }

    fun create_resource() {
        move_to_sender<R>(R{x:1});
    }

    fun exists_resource(): bool {
        exists<R>(Transaction::sender())
    }

    fun move_from_addr(a: address) acquires R {
        let r = move_from<R>(a);
        let R{x: _} = r;
    }

    fun move_from_addr_to_sender(a: address) acquires R {
        let r = move_from<R>(a);
        let R{x: x} = r;
        move_to_sender<R>(R{x: x});
    }

    fun borrow_global_mut_test(a: address) acquires R {
        let r = borrow_global_mut<R>(a);
        _ = r;
        let r2 = borrow_global_mut<R>(a);
        _ = r2;
    }

    resource struct G<X> {
        x: X
    }

    fun create_resoure_generic() {
        move_to_sender<G<u64>>(G{x:1});
    }


    resource struct A {
        addr: address,
        val: u64,
    }

    resource struct B {
        val: u64,
        a: A,
    }

    resource struct C {
        val: u64,
        b: B,
    }

    fun identity(a: A, b: B, c: C): (A,B,C) {
        (a, b, c)
    }

    fun pack_A(a: address, va: u64): A {
        A{ addr:a, val:va }
    }

    fun pack_B(a: address, va: u64, vb: u64): B {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        var_b
    }

    fun pack_C(a: address, va: u64, vb: u64, vc: u64): C {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let var_c = C{ val: vc, b: var_b };
        var_c
    }

    fun unpack_A(a: address, va: u64): (address, u64) {
        let var_a = A{ addr:a, val:va };
        let A{addr: aa, val:v1} = var_a;
        (aa, v1)
    }

    fun unpack_B(a: address, va: u64, vb: u64): (address, u64, u64) {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let B{val: v2, a: A{ addr:aa, val: v1}} = var_b;
        (aa, v1, v2)
    }

    fun unpack_C(a: address, va: u64, vb: u64, vc: u64): (address, u64, u64, u64) {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let var_c = C{ val: vc, b: var_b };
        let C{val: v3, b: B{val: v2, a: A{ addr:aa, val: v1}}} = var_c;
        (aa, v1, v2, v3)
    }

    fun ref_A(a: address, b: bool): A {
        let var_a = if (b) A{ addr: a, val: 1 }
                    else A{ addr: a, val: 42 };
        let var_a_ref = &var_a;
        let b_val_ref = &var_a_ref.val;
        let b_var = *b_val_ref;
        if (b_var != 42) abort 42;
        var_a
    }
}
