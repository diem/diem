module M {
    use 0x1::Event;

    struct MyEvent { b: bool }

    public fun emit_event(account: &signer) {
        let handle = Event::new_event_handle<MyEvent>(account);
        Event::emit_event(&mut handle, MyEvent{ b: true });
        Event::destroy_handle(handle);
    }
}


//! new-transaction
script {
use {{default}}::M;

fun main(account: &signer) {
    M::emit_event(account);
}
}
// check: "Keep(EXECUTED)"
