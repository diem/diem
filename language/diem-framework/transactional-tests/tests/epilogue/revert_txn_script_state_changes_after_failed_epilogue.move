//# init --addresses Alice=0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#                  Bob=0x94b0c6bf91ac1ad628b48a9a6727ddcc
//#                  Carol=0x19b25e27bbea4636167ce772d4d604c7
//#      --private-keys Alice=7f19f4fe121277c4b70585adcfaab625ee4575dee9d1a5e04329b5acf819d7ec
//#                     Bob=c454cda2c0108697dba4200dbb3e75050ddddf9399eb1a07b22c3246c747e30a
//#                     Carol=65082c6cfba04dfb5af2eed0ce38160c0455ec9e60cac50cf24030c85c9e2156


//# run --signers 0xB1E55ED
//#     --private-key TreasuryCompliance
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(s: signer) {
        DiemAccount::create_parent_vasp_account<XUS>(&s, @Alice, x"ab94c5ec24d6642416d2323dfa06fca8", x"", false);
        DiemAccount::create_parent_vasp_account<XUS>(&s, @Bob, x"2882f4c1f39a300d7c03e71e585531e1", x"", false);
        DiemAccount::create_parent_vasp_account<XUS>(&s, @Carol, x"2882f4c1f39a300d7c03e71e585531e1", x"", false);
    }
}


//# run --signers 0xDD
//#     --private-key DesignatedDealer
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(s: signer) {
        let cap = DiemAccount::extract_withdraw_capability(&s);
        DiemAccount::pay_from<XUS>(&cap, @Alice, 1000000, x"", x"");
        DiemAccount::pay_from<XUS>(&cap, @Bob, 1000000, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}


// Transfer all of the Alice's funds to Carol. this script will execute successfully, but
// the epilogue will fail because Alice spent her gas deposit. The VM should revert the state
// changes and re-execute the epilogue. Alice will still be charged for the gas she used.
//
//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
//#     --gas-price 1
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(account: signer) {
        let cap = DiemAccount::extract_withdraw_capability(&account);
        DiemAccount::pay_from<XUS>(&cap, @Carol, 1000000, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}


// Bob sends the same transaction script, with the amount set to 1000 instead of his full balance.
// This transaction should go through and bob will pay for the gas he used.
//
//# run --signers 0x94b0c6bf91ac1ad628b48a9a6727ddcc
//#     --private-key Bob
//#     --gas-price 1
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(account: signer) {
        let cap = DiemAccount::extract_withdraw_capability(&account);
        DiemAccount::pay_from<XUS>(&cap, @Carol, 1000, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}


// Check that the following invariants holds:
// 1) Carol's balance has went up by 1000, receiving funds from Bob but not Alice.
// 2) Alice's balance is exactly 1000 greater than Bob's, indicating they consumed the same amount of gas.
//
//# run --signers 0xA550C18
//#     --private-key DiemRoot
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main() {
        assert(DiemAccount::balance<XUS>(@Carol) == 1000, 42);
        assert(DiemAccount::balance<XUS>(@Alice) == DiemAccount::balance<XUS>(@Bob) + 1000, 43)
    }
}
