address 0x2 {
module Test {
    struct R has key { i: u64 }

    public fun publish(account: &signer, i: u64) {
        move_to(account, R { i })
    }
}
}
