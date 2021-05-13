module {{default}}::M {
    use Std::Event;

    struct MyEvent has copy, drop, store { b: bool }

    public fun emit_event(account: &signer) {
        let handle = Event::new_event_handle<MyEvent>(account);
        Event::emit_event(&mut handle, MyEvent{ b: true });
        Event::destroy_handle(handle);
    }
}


//! new-transaction
script {
use {{default}}::M;

fun main(account: signer) {
    let account = &account;
    M::emit_event(account);
}
}
// check: "Keep(EXECUTED)"
