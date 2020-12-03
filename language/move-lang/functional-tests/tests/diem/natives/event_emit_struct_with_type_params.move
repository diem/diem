module M {
    use 0x1::Event;

    struct MyEvent<T1, T2> { b: bool }

    public fun emit_event<T1: copyable, T2: copyable>(account: &signer) {
        let handle = Event::new_event_handle<MyEvent<T2, T1>>(account);
        Event::emit_event(&mut handle, MyEvent{ b: true });
        Event::destroy_handle(handle);
    }
}


//! new-transaction
script {
use {{default}}::M;

fun main(account: &signer) {
    M::emit_event<bool, u64>(account);
}
}
// check: "Keep(EXECUTED)"
