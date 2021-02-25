address 0x2 {
module M {
    struct R has key { f: u64 }

    public fun publish(account: &signer) {
        move_to(account, R { f: 10 })
    }
}
}
