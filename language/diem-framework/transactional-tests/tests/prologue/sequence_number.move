//# init --addresses Alice=0x76c3623ac80c38e2f1f4e90f07c8e21a
//#      --private-keys Alice=1db3dd81f19eb4a22028036e05043a3b42415e2297439195c804744fd16ecd6a


//# run --signers 0xA550C18
//#     --private-key DiemRoot
script {
    use DiemFramework::DiemAccount::create_validator_account;

    fun main(s: signer) {
        create_validator_account(&s, @Alice, x"35830210d93ec6a22db9394a7367f72d", x"40");
    }
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 1
//
// This should fail since the sequence number is too new.
script {
    fun main() {}
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 0
//
// This should succeed since the sequence number matches the default (0).
script {
    fun main() {}
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 0
//
// The sequence number was incremented to 1 by the previsous transaction, so trying to
// execute with sequence number 0 again will result in another failure since it's now too old.
script {
    fun main() {}
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 1
//
// Current sequence number => should succeed.
// Sequence number is now 2.
script {
    fun main() {}
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 2
//
// Current sequence number => should succeed.
// Sequence number is now 3.
script {
    fun main() {}
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 0
//
// 0 < 3 => should fail.
script {
    fun main() {}
}


//# run --signers 0x76c3623ac80c38e2f1f4e90f07c8e21a
//#     --private-key Alice
//#     --sequence-number 5
//
// 5 > 3 => should fail.
script {
    fun main() {}
}
