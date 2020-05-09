//! account: bob, 100000000, 0, unhosted
//! account: alice, 0, 0, unhosted

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 10001);
    }
}
// TODO: fix account limits
// chec: ABORTED
// chec: 9

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
// TODO: fix account limits
// chec: ABORTED
// chec: 11

//! new-transaction
//! sender: bob
//! gas-price: 1000
script {
    fun main() { }
}
// TODO: fix account limits

// chec: ABORTED
// chec: 11
