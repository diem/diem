//# init --addresses Alice=0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#                  Bob=0x94b0c6bf91ac1ad628b48a9a6727ddcc
//#      --private-keys Alice=7f19f4fe121277c4b70585adcfaab625ee4575dee9d1a5e04329b5acf819d7ec
//#                     Bob=c454cda2c0108697dba4200dbb3e75050ddddf9399eb1a07b22c3246c747e30a


//# run --signers 0xB1E55ED
//#     --private-key TreasuryCompliance
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;

    fun main(s: signer) {
        DiemAccount::create_parent_vasp_account<XUS>(&s, @Alice, x"ab94c5ec24d6642416d2323dfa06fca8", x"", false);
        DiemAccount::create_parent_vasp_account<XUS>(&s, @Bob, x"2882f4c1f39a300d7c03e71e585531e1", x"", false);
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
        DiemAccount::pay_from<XUS>(&cap, @Bob, 10000, x"", x"");
        DiemAccount::restore_withdraw_capability(cap);
    }
}


//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
//#     --gas-price 1
script {
    fun main() {}
}


//# run --signers 0x94b0c6bf91ac1ad628b48a9a6727ddcc
//#     --private-key Bob
//#     --gas-price 1
script {
    fun main() {
        let x = 1;
        while (x < 2000) x = x + 1;
    }
}


//# run --signers 0xA550C18
//#     --private-key DiemRoot
script {
    use DiemFramework::DiemAccount;
    use DiemFramework::XUS::XUS;
    fun main() {
        // Alice did less work than bob so she should pay less gas.
        assert(DiemAccount::balance<XUS>(@Bob) < DiemAccount::balance<XUS>(@Alice), 42);
    }
}
