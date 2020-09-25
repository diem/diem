address 0x2 {
module Test {
    resource struct R { i: u64 }

    public fun publish(account: &signer, i: u64) {
        move_to(account, R { i })
    }
}
}
