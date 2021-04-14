module 0x42::TestMonomorphization {
    use 0x1::Signer;
    use 0x1::BCS;
    use 0x1::Event;
    use 0x1::Vector;

    struct R<T: store> has key {
        x: T,
        y: T
    }

    public fun pack_R(): R<u64> {
        R{x: 1, y: 2}
    }
    spec fun pack_R {
        ensures result == spec_pack_R();
    }
    spec define spec_pack_R(): R<u64> { R{x: 1, y: 2}}

    public fun create_R(account: &signer) {
        move_to<R<u64>>(account, R{x:1, y:2} );
    }
    spec fun create_R {
        aborts_if exists<R<u64>>(Signer::spec_address_of(account));
        ensures exists<R<u64>>(Signer::spec_address_of(account));
    }

    public fun mutate_R(addr: address) acquires R {
        borrow_global_mut<R<bool>>(addr).y = false;
    }
    spec fun mutate_R {
        ensures global<R<bool>>(addr) == update_field(old(global<R<bool>>(addr)), y, false);
    }

    public fun create_R_generic<T: store>(account: &signer, x: T, y: T) {
        move_to<R<T>>(account, R{x, y});
    }
    spec fun create_R_generic {
        aborts_if exists<R<T>>(Signer::spec_address_of(account));
        ensures exists<R<T>>(Signer::spec_address_of(account));
    }

    public fun use_vec(_x: vector<u64>) {
    }

    public fun use_bcs<T>(x: &T): (vector<u8>, vector<u8>) {
        (BCS::to_bytes(x), BCS::to_bytes(&0x2))
    }
    spec fun use_bcs {
        ensures result_1 == BCS::serialize(x);
        ensures result_2 == BCS::serialize(0x2);
    }

    struct E has copy, drop, store {
        msg: u64
    }

    public fun use_event(handle: &mut Event::EventHandle<E>) {
        Event::emit_event(handle, E{msg: 0});
    }
    spec fun use_event {
        emits E{msg: 0} to handle;
    }

    // The following set of functions exercise different style of vector instantiations each with an error which
    // should print the vector. Running outside of the test environment (without value redaction) allow to manually
    // inspect printing.
    public fun vec_int(x: vector<u64>): vector<u64> { Vector::push_back(&mut x, 1); x }
    spec fun vec_int { ensures result[0] != 1; }
    public fun vec_addr(x: vector<address>): vector<address> { Vector::push_back(&mut x, 0x1); x }
    spec fun vec_addr { ensures result[0] != 0x1; }
    public fun vec_bool(x: vector<bool>): vector<bool> { Vector::push_back(&mut x, true); x }
    spec fun vec_bool { ensures result[0] != true; }
    public fun vec_struct_int(x: vector<R<u64>>): vector<R<u64>> { Vector::push_back(&mut x, R{x: 1, y: 1}); x }
    spec fun vec_struct_int { ensures result[0].x != 1; }
    public fun vec_struct_addr(x: vector<R<address>>): vector<R<address>> { Vector::push_back(&mut x, R{x: 0x1, y: 0x2}); x }
    spec fun vec_struct_addr { ensures result[0].x != 0x1; }

    public fun vec_vec(x: vector<vector<u64>>): vector<vector<u64>> {
        Vector::push_back(&mut x, Vector::empty<u64>()); x
    }
    spec fun vec_vec { ensures len(result[0]) != 0; }
}
