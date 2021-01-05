address 0x42 {
module M {
    struct S {
        f1: Option<u64>,
        f2: Option::Option<u64>,
    }

    fun ex(
        _a1: Option<u64>,
        _a2: Option::Option<u64>,
        s: &signer,
    ) {
        Option::none<u64>();
        Option::some(0);
        Signer::borrow_address(s);
        Signer::address_of(s);
        Vector::empty<u64>();
        Vector::push_back(&mut Vector::empty(), 0);
    }
}
}
