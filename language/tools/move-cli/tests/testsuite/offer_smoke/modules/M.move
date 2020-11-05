// Just a dummy module with a test resource
address 0x1 {
module M {
    resource struct T { b: bool }

    public fun create(): T {
        T { b: true }
    }

    public fun publish(account: &signer, t: T) {
        move_to(account,  t);
    }
}
}
