//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer-address: 0x0
//! block-time: 1


//! block-prologue
//! proposer-address: 0x0
//! block-time: 0

//! new-transaction
script {
    use 0x1::DiemTimestamp;
    fun main(account: &signer) {
        DiemTimestamp::set_time_has_started(account);
    }
}

//! new-transaction
//! sender: diemroot
script {
    use 0x1::DiemTimestamp;
    fun main(account: &signer) {
        DiemTimestamp::set_time_has_started(account);
    }
}
