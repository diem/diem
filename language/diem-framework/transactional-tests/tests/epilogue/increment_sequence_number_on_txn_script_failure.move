//# init --addresses Alice=0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#      --private-keys Alice=7f19f4fe121277c4b70585adcfaab625ee4575dee9d1a5e04329b5acf819d7ec


//# run --signers 0xB1E55ED
//#     --private-key TreasuryCompliance
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(s: signer) {
        DiemAccount::create_parent_vasp_account<XUS>(&s, @Alice, x"ab94c5ec24d6642416d2323dfa06fca8", x"", false);
    }
}


// When a transaction aborts, the sequence number should still be bumped up.
//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
script {
    fun main() {
        assert(false, 77);
    }
}


//# run --signers 0xA550C18
//#     --private-key DiemRoot
script {
    use DiemFramework::DiemAccount;

    fun main() {
        assert(DiemAccount::sequence_number(@Alice) == 1, 42);
    }
}
