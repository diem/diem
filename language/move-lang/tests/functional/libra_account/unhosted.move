//! account: bob, 100000000, 0, unhosted
//! account: alice, 0, 0, unhosted

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::mint_to_address<LBR::T>({{bob}}, 10001);
    }
}
// check: ABORTED
// check: 9

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::deposit({{bob}},
            LibraAccount::withdraw_from_sender<LBR::T>(10001)
        );
    }
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: bob
//! gas-price: 1000
script {
        fun main() { }
    }
// check: ABORTED
// check: 11
