module 0x2::A {
    use Std::Event;

    struct MyEvent<phantom T> has copy, drop, store { b: bool }

    public fun do_emit<T: copy + drop + store>(account: &signer) {
        let handle = Event::new_event_handle<MyEvent<T>>(account);
        Event::emit_event(&mut handle, MyEvent{ b: true });
        Event::destroy_handle(handle);
    }

    #[test(a=@0x2)]
    public fun emit(a: &signer) {
        Event::publish_generator(a);
        do_emit<u64>(a);
    }
}
