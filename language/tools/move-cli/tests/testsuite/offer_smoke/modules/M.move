// Just a dummy module with a test resource
address 0x1 {
module M {
    struct T has key, store { b: bool }

    public fun create(): T {
        T { b: true }
    }

    public fun publish(account: &signer, t: T) {
        move_to(account,  t);
    }
}
}
