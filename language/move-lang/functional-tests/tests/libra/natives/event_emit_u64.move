script {
use 0x1::Event;

fun main(account: &signer) {
    let handle = Event::new_event_handle<u64>(account);
    Event::emit_event(&mut handle, 42);
    Event::destroy_handle(handle);
}
}
// check: "Keep(EXECUTED)"
