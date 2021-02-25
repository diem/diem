address 0x2 {
module Events {
    use 0x1::Event;
    use 0x1::Signer;

    struct AnEvent has copy, drop, store { i: u64 }
    struct Handle has key{ h: Event::EventHandle<AnEvent> }

    public fun emit(account: &signer, i: u64) acquires Handle {
        let addr = Signer::address_of(account);
        if (!exists<Handle>(addr)) {
            Event::publish_generator(account);
            move_to(account, Handle { h: Event::new_event_handle(account) })
        };

        let handle = borrow_global_mut<Handle>(addr);

        Event::emit_event(&mut handle.h, AnEvent { i })
    }
}
}
