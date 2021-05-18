module 0x2::A {
    use 0x1::Signer;

    #[test(s=@0x2)]
    public fun check_signer_spec_address_of(s: &signer) {
        let a = Signer::address_of(s);
        spec {
            assert a == Signer::spec_address_of(s);
            assert a == Signer::address_of(s);
        };
    }
}
