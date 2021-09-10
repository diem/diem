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


//# run --signers 0xDD
//#     --private-key DesignatedDealer
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(s: signer) {
        let cap = DiemAccount::extract_withdraw_capability(&s);
        DiemAccount::pay_from<XUS>(&cap, @Alice, 10000, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}


//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
//#     --gas-budget 5000
//#     --gas-price 1
script {
    fun main() {}
}


//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    use Std::Signer;

    fun main(account: signer) {
        let account = &account;
        let sender = Signer::address_of(account);
        // Ensures that the account was deducted for the gas fee.
        assert(DiemAccount::balance<XUS>(sender) < 10000, 42);
        // Ensures that we are not just charging max_gas for the transaction.
        assert(DiemAccount::balance<XUS>(sender) >= 5000, 42);
    }
}
