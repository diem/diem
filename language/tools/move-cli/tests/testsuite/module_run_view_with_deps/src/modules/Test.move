address 0x2 {
module Test {
    use 0x1::Debug;

    resource struct R { i: u64 }

    public fun publish(account: &signer, i: u64) {
        Debug::print(&i);
        move_to(account, R { i })
    }
}
}
