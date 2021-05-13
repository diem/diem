module 0x2::A {
    use Std::Signer;

    struct R1<T: store> has key { f: T }

    fun mutate_r1(addr: address) acquires R1 {
        borrow_global_mut<R1<bool>>(addr).f = true;
    }

    spec mutate_r1 {
        ensures global<R1<bool>>(addr) == update_field(old(global<R1<bool>>(addr)), f, true);
    }

    #[test(s=@0x2)]
    public fun check_mem_label_set(s: &signer) acquires R1 {
        let a = Signer::address_of(s);
        let r = R1 { f: false };
        move_to(s, r);
        mutate_r1(a);
        spec {
            assert global<R1<bool>>(a).f;
        };
    }

    struct R2<T: store> has key { f1: T, f2: T }

    fun mutate_r2(addr: address) acquires R2 {
        let r = borrow_global_mut<R2<bool>>(addr);
        let f = r.f1;
        r.f1 = r.f2;
        r.f2 = f;
    }

    spec mutate_r2 {
        ensures old(spec_get_r2<bool>(addr).f1) == spec_get_r2<bool>(addr).f2;
        ensures old(spec_get_r2<bool>(addr).f2) == spec_get_r2<bool>(addr).f1;
    }

    spec fun spec_get_r2<T: store>(addr: address): R2<T> {
        global<R2<T>>(addr)
    }

    #[test(s=@0x2)]
    public fun check_mem_label_swap(s: &signer) acquires R2 {
        let a = Signer::address_of(s);
        let r = R2 { f1: true, f2: false };
        move_to(s, r);
        mutate_r2(a);
        spec {
            assert !global<R2<bool>>(a).f1;
            assert global<R2<bool>>(a).f2;
        };
    }
}
