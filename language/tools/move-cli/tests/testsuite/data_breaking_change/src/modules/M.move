address 0x2 {
module M {
    resource struct R { f: u64 }

    public fun publish(account: &signer) {
        move_to(account, R { f: 10 })
    }
}
}
