address 0x2 {
module ResourceExists {
    struct R has key { }

    public fun f(account: &signer) {
        move_to<R>(account, R {});
        move_to<R>(account, R {}); // will fail here
    }
}
}
