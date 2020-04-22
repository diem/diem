// dep: tests/sources/stdlib/modules/transaction.move

module TestResources {
    use 0x0::Transaction;

    // ---------------
    // Simple resource
    // ---------------

    resource struct R {
        x: u64
    }

    fun create_resource() {
        move_to_sender<R>(R{x:1});
    }
    spec fun create_resource {
        aborts_if exists<R>(sender());
        ensures exists<R>(sender());
    }

    fun create_resource_incorrect() {
        if(exists<R>(Transaction::sender())) {
            abort 1
        };
    }
    spec fun create_resource_incorrect {
     aborts_if exists<R>(sender());
     ensures exists<R>(sender());
    }

    fun move_from_addr(a: address) acquires R {
        let r = move_from<R>(a);
        let R{x: _} = r;
    }
    spec fun move_from_addr {
        aborts_if !exists<R>(a);
    }

    fun move_from_addr_to_sender(a: address) acquires R {
        let r = move_from<R>(a);
        let R{x: x} = r;
        move_to_sender<R>(R{x: x});
    }
    spec fun move_from_addr_to_sender {
        aborts_if !exists<R>(a);
        aborts_if (sender() != a) && exists<R>(sender());
        ensures exists<R>(sender());
        ensures (sender() != a) ==> !exists<R>(a);
        ensures old(global<R>(a).x) == global<R>(sender()).x;
        //ensures old(global<R>(a)) == global<R>(sender()); // FIXME: this should be verified
    }

    fun move_from_addr_and_return(a: address): R acquires R {
        let r = move_from<R>(a);
        let R{x: x} = r;
        R{x: x}
    }
    spec fun move_from_addr_and_return {
        aborts_if !exists<R>(a);
        ensures old(exists<R>(a));
        ensures result.x == old(global<R>(a).x);
        //ensures result == old(global<R>(a)); // FIXME: this should be verified
    }

    fun move_from_sender_and_return(): R acquires R {
        let r = move_from<R>(Transaction::sender());
        let R{x: x} = r;
        R{x: x}
    }
    spec fun move_from_sender_and_return {
        aborts_if !exists<R>(sender());
        ensures result.x == old(global<R>(sender()).x);
        //ensures result == old(global<R>(sender())); // FIXME: this should be verified
    }

    fun move_from_sender_to_sender() acquires R {
        let r = move_from<R>(Transaction::sender());
        let R{x: x} = r;
        move_to_sender<R>(R{x: x});
    }
    spec fun move_from_sender_to_sender {
        aborts_if !exists<R>(sender());
        ensures exists<R>(sender());
        ensures old(global<R>(sender()).x) == global<R>(sender()).x;
        //ensures old(global<R>(sender())) == global<R>(sender()); // FIXME: this should be verified
    }

    fun borrow_global_mut_correct(a: address) acquires R {
        let r = borrow_global_mut<R>(a);
        _ = r;
        let r2 = borrow_global_mut<R>(a);
        _ = r2;
    }
    spec fun borrow_global_mut_correct {
        aborts_if !exists<R>(a);
    }


    // ---------------
    // Nested resource
    // ---------------

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
    spec fun identity {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == b;
        ensures result_3 == c;
    }

    fun pack_A(a: address, va: u64): A {
        A{ addr:a, val:va }
    }
    spec fun pack_A {
        aborts_if false;
        ensures result.addr == a;
        ensures result.val == va;
    }

    fun pack_B(a: address, va: u64, vb: u64): B {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        var_b
    }
    spec fun pack_B {
        aborts_if false;
        ensures result.val == vb;
        ensures result.a.val == va;
        ensures result.a.addr == a;
    }

    fun pack_C(a: address, va: u64, vb: u64, vc: u64): C {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let var_c = C{ val: vc, b: var_b };
        var_c
    }
    spec fun pack_C {
        aborts_if false;
        ensures result.val == vc;
        ensures result.b.val == vb;
        ensures result.b.a.val == va;
        ensures result.b.a.addr == a;
    }

    fun unpack_A(a: address, va: u64): (address, u64) {
        let var_a = A{ addr:a, val:va };
        let A{addr: aa, val:v1} = var_a;
        (aa, v1)
    }
    spec fun unpack_A {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == va;
    }

    fun unpack_B(a: address, va: u64, vb: u64): (address, u64, u64) {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let B{val: v2, a: A{ addr:aa, val: v1}} = var_b;
        (aa, v1, v2)
    }
    spec fun unpack_B {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == va;
        ensures result_3 == vb;
    }

    fun unpack_C(a: address, va: u64, vb: u64, vc: u64): (address, u64, u64, u64) {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let var_c = C{ val: vc, b: var_b };
        let C{val: v3, b: B{val: v2, a: A{ addr:aa, val: v1}}} = var_c;
        (aa, v1, v2, v3)
    }
    spec fun unpack_C {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == va;
        ensures result_3 == vb;
        ensures result_4 == vc;
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
    spec fun ref_A {
        aborts_if b;
        ensures result.addr == a;
    }


    // ---------------
    // Packs in spec
    // ---------------

    fun spec_pack_R(): R {
        R{x: 7}
    }
    spec fun spec_pack_R {
        aborts_if false;
        ensures result.x == 7;
        ensures result == R{x: 7};
    }

    fun spec_pack_A(): A {
        A{ addr: Transaction::sender(), val: 7 }
    }
    spec fun spec_pack_A {
        aborts_if false;
        ensures result.addr == sender();
        ensures result.val == 7;
        ensures result == A{ addr: sender(), val: 7 };
        ensures result == A{ val: 7, addr: sender() };
    }

    fun spec_pack_B(): B {
        B{ val: 77, a: A{ addr: Transaction::sender(), val: 7 }}
    }
    spec fun spec_pack_B {
        aborts_if false;
        ensures result.val == 77;
        ensures result.a.val == 7;
        ensures result.a.addr == sender();
        ensures result == B{ val: 77, a: A{ addr: sender(), val: 7 }};
        ensures result == B{ val: 77, a: A{ val: 7, addr: sender()}};
        ensures result == B{ a: A{ addr: sender(), val: 7 }, val: 77 };
        ensures result == B{ a: A{ val: 7, addr: sender()}, val: 77 };
    }
}
