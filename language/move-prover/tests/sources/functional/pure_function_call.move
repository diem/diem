module TestPureFun {
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::Vector;

    resource struct T {
        x: u64,
    }

    public fun init(dr_account: &signer): bool {
        assert(Signer::address_of(dr_account) == 0xA550C18, 0);
        move_to(dr_account, T { x: 2 });
        false
    }

    spec fun init {
        aborts_if Signer::spec_address_of(dr_account) != CoreAddresses::DIEM_ROOT_ADDRESS();
        aborts_if exists<T>(Signer::spec_address_of(dr_account));
        ensures dr_x() == pure_f_2();
    }

    public fun get_x(addr: address): u64 acquires T {
        assert(exists<T>(addr), 10);
        assert(true, 0); // assertions are ignored when translating Move funs to spec funs.
        *&borrow_global<T>(addr).x
    }


    public fun get_x_plus_one(addr: address): u64 acquires T {
        get_x(addr) + 1
    }

    public fun increment_x(addr: address) acquires T {
        let t = borrow_global_mut<T>(addr);
        t.x = t.x + 1;
    }

    spec fun increment_x {
        ensures get_x(addr) == old(get_x(addr)) + 1;
        ensures get_x(addr) == old(get_x_plus_one(addr));
    }

    public fun pure_f_2(): u64 {
        pure_f_1() + 1
    }

    public fun pure_f_1(): u64 {
        pure_f_0() + 1
    }

    public fun pure_f_0(): u64 {
        0
    }

    public fun get_elem(v: &vector<T>): u64 {
        Vector::borrow(v, 0).x
    }

    spec fun get_elem {
        aborts_if Vector::is_empty(v);
    }

    spec module {
        define dr_x(): u64 {
            get_x(CoreAddresses::DIEM_ROOT_ADDRESS())
        }
    }

}
