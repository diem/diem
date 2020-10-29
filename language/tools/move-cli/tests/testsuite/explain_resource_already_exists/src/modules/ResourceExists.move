address 0x2 {
module ResourceExists {
    resource struct R { }

    public fun f(account: &signer) {
        move_to<R>(account, R {});
        move_to<R>(account, R {}); // will fail here
    }
}
}
