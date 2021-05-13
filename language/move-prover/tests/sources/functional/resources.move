module 0x42::TestResources {

    use Std::Signer;
    spec module {
        pragma verify = true;
    }

    // ---------------
    // Simple resource
    // ---------------

    struct R has key {
        x: u64
    }

    public fun create_resource_at_signer(account: &signer) {
        move_to<R>(account, R{x:1});
    }
    spec create_resource_at_signer {
        aborts_if exists<R>(Signer::spec_address_of(account));
        ensures exists<R>(Signer::spec_address_of(account));
    }

    public fun create_resource(account: &signer) {
        move_to<R>(account, R{x:1});
    }
    spec create_resource {
        aborts_if exists<R>(Signer::spec_address_of(account));
        ensures exists<R>(Signer::spec_address_of(account));
    }

    public fun create_resource_incorrect(account: &signer) {
        if(exists<R>(Signer::address_of(account))) {
            abort 1
        };
    }
    spec create_resource_incorrect {
     aborts_if exists<R>(Signer::spec_address_of(account));
     ensures exists<R>(Signer::spec_address_of(account));
    }

    public fun move_from_addr(a: address) acquires R {
        let r = move_from<R>(a);
        let R{x: _} = r;
    }
    spec move_from_addr {
        aborts_if !exists<R>(a);
    }

    public fun move_from_addr_to_sender(account: &signer, a: address) acquires R {
        let r = move_from<R>(a);
        let R{x: x} = r;
        move_to<R>(account, R{x: x});
    }
    spec move_from_addr_to_sender {
        aborts_if !exists<R>(a);
        aborts_if (Signer::spec_address_of(account) != a) && exists<R>(Signer::spec_address_of(account));
        ensures exists<R>(Signer::spec_address_of(account));
        ensures (Signer::spec_address_of(account) != a) ==> !exists<R>(a);
        ensures old(global<R>(a).x) == global<R>(Signer::spec_address_of(account)).x;
        ensures old(global<R>(a)) == global<R>(Signer::spec_address_of(account));
    }

    public fun move_from_addr_and_return(a: address): R acquires R {
        let r = move_from<R>(a);
        let R{x: x} = r;
        R{x: x}
    }
    spec move_from_addr_and_return {
        aborts_if !exists<R>(a);
        ensures old(exists<R>(a));
        ensures result.x == old(global<R>(a).x);
        ensures result == old(global<R>(a));
    }

    public fun move_from_sender_and_return(account: &signer): R acquires R {
        let r = move_from<R>(Signer::address_of(account));
        let R{x: x} = r;
        R{x: x}
    }
    spec move_from_sender_and_return {
        aborts_if !exists<R>(Signer::spec_address_of(account));
        ensures result.x == old(global<R>(Signer::spec_address_of(account)).x);
        ensures result == old(global<R>(Signer::spec_address_of(account)));
    }

    public fun move_from_sender_to_sender(account: &signer) acquires R {
        let r = move_from<R>(Signer::address_of(account));
        let R{x: x} = r;
        move_to<R>(account, R{x: x});
    }
    spec move_from_sender_to_sender {
        aborts_if !exists<R>(Signer::spec_address_of(account));
        ensures exists<R>(Signer::spec_address_of(account));
        ensures old(global<R>(Signer::spec_address_of(account)).x) == global<R>(Signer::spec_address_of(account)).x;
        ensures old(global<R>(Signer::spec_address_of(account))) == global<R>(Signer::spec_address_of(account));
    }

    public fun borrow_global_mut_correct(a: address) acquires R {
        let r = borrow_global_mut<R>(a);
        _ = r;
        let r2 = borrow_global_mut<R>(a);
        _ = r2;
    }
    spec borrow_global_mut_correct {
        aborts_if !exists<R>(a);
    }


    // ---------------
    // Nested resource
    // ---------------

    struct A {
        addr: address,
        val: u64,
    }

    struct B {
        val: u64,
        a: A,
    }

    struct C {
        val: u64,
        b: B,
    }

    public fun identity(a: A, b: B, c: C): (A,B,C) {
        (a, b, c)
    }
    spec identity {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == b;
        ensures result_3 == c;
    }

    public fun pack_A(a: address, va: u64): A {
        A{ addr:a, val:va }
    }
    spec pack_A {
        aborts_if false;
        ensures result.addr == a;
        ensures result.val == va;
    }

    public fun pack_B(a: address, va: u64, vb: u64): B {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        var_b
    }
    spec pack_B {
        aborts_if false;
        ensures result.val == vb;
        ensures result.a.val == va;
        ensures result.a.addr == a;
    }

    public fun pack_C(a: address, va: u64, vb: u64, vc: u64): C {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let var_c = C{ val: vc, b: var_b };
        var_c
    }
    spec pack_C {
        aborts_if false;
        ensures result.val == vc;
        ensures result.b.val == vb;
        ensures result.b.a.val == va;
        ensures result.b.a.addr == a;
    }

    public fun unpack_A(a: address, va: u64): (address, u64) {
        let var_a = A{ addr:a, val:va };
        let A{addr: aa, val:v1} = var_a;
        (aa, v1)
    }
    spec unpack_A {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == va;
    }

    public fun unpack_B(a: address, va: u64, vb: u64): (address, u64, u64) {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let B{val: v2, a: A{ addr:aa, val: v1}} = var_b;
        (aa, v1, v2)
    }
    spec unpack_B {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == va;
        ensures result_3 == vb;
    }

    public fun unpack_C(a: address, va: u64, vb: u64, vc: u64): (address, u64, u64, u64) {
        let var_a = A{ addr: a, val: va };
        let var_b = B{ val: vb, a: var_a };
        let var_c = C{ val: vc, b: var_b };
        let C{val: v3, b: B{val: v2, a: A{ addr:aa, val: v1}}} = var_c;
        (aa, v1, v2, v3)
    }
    spec unpack_C {
        aborts_if false;
        ensures result_1 == a;
        ensures result_2 == va;
        ensures result_3 == vb;
        ensures result_4 == vc;
    }

    public fun ref_A(a: address, b: bool): A {
        let var_a = if (b) A{ addr: a, val: 1 }
                    else A{ addr: a, val: 42 };
        let var_a_ref = &var_a;
        let b_val_ref = &var_a_ref.val;
        let b_var = *b_val_ref;
        if (b_var != 42) abort 42;
        var_a
    }
    spec ref_A {
        aborts_if b;
        ensures result.addr == a;
    }


    // ---------------
    // Packs in spec
    // ---------------

    public fun spec_pack_R(): R {
        R{x: 7}
    }
    spec spec_pack_R {
        aborts_if false;
        ensures result.x == 7;
        ensures result == R{x: 7};
    }

    public fun spec_pack_A(account: &signer): A {
        A{ addr: Signer::address_of(account), val: 7 }
    }
    spec spec_pack_A {
        aborts_if false;
        ensures result.addr == Signer::spec_address_of(account);
        ensures result.val == 7;
        ensures result == A{ addr: Signer::spec_address_of(account), val: 7 };
        ensures result == A{ val: 7, addr: Signer::spec_address_of(account) };
    }

    public fun spec_pack_B(account: &signer): B {
        B{ val: 77, a: A{ addr: Signer::address_of(account), val: 7 }}
    }
    spec spec_pack_B {
        aborts_if false;
        ensures result.val == 77;
        ensures result.a.val == 7;
        ensures result.a.addr == Signer::spec_address_of(account);
        ensures result == B{ val: 77, a: A{ addr: Signer::spec_address_of(account), val: 7 }};
        ensures result == B{ val: 77, a: A{ val: 7, addr: Signer::spec_address_of(account)}};
        ensures result == B{ a: A{ addr: Signer::spec_address_of(account), val: 7 }, val: 77 };
        ensures result == B{ a: A{ val: 7, addr: Signer::spec_address_of(account)}, val: 77 };
    }

    // ------------
    // Empty struct
    // ------------

    struct Empty {}

    public fun create_empty(): Empty {
        Empty{}
    }
    spec create_empty {
        ensures result == Empty{};
    }

}
