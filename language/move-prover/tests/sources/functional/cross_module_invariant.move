module TestCrossModuleInv {
    use 0x1::DiemAccount;
    use 0x1::XUS;
    use 0x1::Signer;

    public fun add_XUS_balance(account: &signer) {
        assert(Signer::address_of(account) == 0x1, 1);
        DiemAccount::add_currency<XUS::XUS>(account);
    }

    spec module {
        // TODO: this test has been deactivated because of non-determinism in the generated prover diagnosis
        //   The test should be rewritten to not depend on libra framework. Functional tests should be standalone.
        pragma verify = false;
        /// When we verify module `TestCrossModuleInv`, although the code in this
        /// module doesn't violate the invariant, prover would still generate
        /// an error saying the invariant is violated at `DiemAccount::add_currency`.
        /// `DiemAccount::add_currency` is verified even when `DiemAccount` is not
        /// a target because `DiemAccount::add_currency` directly modifies Balance
        /// resource, which is mentioned in the invariant here.
        invariant [global, deactivated] forall addr: address: exists<DiemAccount::Balance<XUS::XUS>>(addr) ==> addr == 0x1;
    }
}
