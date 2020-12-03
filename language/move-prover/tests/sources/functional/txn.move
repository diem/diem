address 0x1 {

module TestTransaction {
    use 0x1::Signer;
    use 0x1::DiemAccount;
    use 0x1::DiemTimestamp;

    spec module {
        pragma verify = true;
    }

    resource struct T {
        value: u128,
    }

    fun check_sender1(sender: &signer) {
        assert(Signer::address_of(sender) == 0xdeadbeef, 1);
    }
    spec fun check_sender1 {
        aborts_if Signer::spec_address_of(sender) != 0xdeadbeef;
    }

    fun check_sender2(sender: &signer) acquires T {
	    borrow_global<T>(Signer::address_of(sender));
    }
    spec fun check_sender2 {
        aborts_if !exists<T>(Signer::spec_address_of(sender));
    }

    fun exists_account(account: &signer) {
        DiemTimestamp::assert_operating();
        assert(DiemAccount::exists_at(Signer::address_of(account)), 1);
    }
    spec fun exists_account {
        include DiemTimestamp::AbortsIfNotOperating;
        // TODO: we can remove the following line once we have the feature to inject
        // the postconditions of the "prologue" functions as invariants
        aborts_if !exists<DiemAccount::DiemAccount>(Signer::spec_address_of(account));
    }
}
}
