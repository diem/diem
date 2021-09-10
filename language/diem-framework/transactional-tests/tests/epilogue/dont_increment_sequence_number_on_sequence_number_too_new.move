//# init --addresses Alice=0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#      --private-keys Alice=7f19f4fe121277c4b70585adcfaab625ee4575dee9d1a5e04329b5acf819d7ec


//# run --signers 0xA550C18
//#     --private-key DiemRoot
script {
    use DiemFramework::DiemAccount::create_validator_account;

    fun main(s: signer) {
        create_validator_account(&s, @Alice, x"ab94c5ec24d6642416d2323dfa06fca8", x"40");
    }
}


//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
//#     --sequence-number 5
// This should fail because the sequence number is too new.
script {
    fun main() {}
}


//# run --signers 0x7e19cb2c0b2a3bbcb48950f59d8d676a
//#     --private-key Alice
//#     --sequence-number 0
// Running with 0 should succeed because sequence number wasn't bumped.
script {
    fun main() {}
}
