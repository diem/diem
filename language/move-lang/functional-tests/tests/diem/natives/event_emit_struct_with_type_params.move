module {{default}}::M {
    use Std::Event;

    struct MyEvent<phantom T1, phantom T2> has copy, drop, store { b: bool }

    public fun emit_event<T1: copy + drop + store, T2: copy + drop + store>(account: &signer) {
        let handle = Event::new_event_handle<MyEvent<T2, T1>>(account);
        Event::emit_event(&mut handle, MyEvent{ b: true });
        Event::destroy_handle(handle);
    }
}


//! new-transaction
script {
use {{default}}::M;

fun main(account: signer) {
    let account = &account;
    M::emit_event<bool, u64>(account);
}
}
// check: "Keep(EXECUTED)"
