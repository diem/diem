module TestEmits {
    use 0x1::Event::{Self, EventHandle};

    struct DummyEvent {
        msg: u64
    }

    // -------------------------
    // simple `emits` statements
    // -------------------------

    public fun simple(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec fun simple {
        emits DummyEvent{msg: 0} to handle;
    }

    public fun simple_wrong_msg_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec fun simple_wrong_msg_incorrect {
        emits DummyEvent{msg: 1} to handle;
    }

    public fun simple_wrong_handle_incorrect(handle: &mut EventHandle<DummyEvent>, _handle2: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec fun simple_wrong_handle_incorrect {
        emits DummyEvent{msg: 0} to _handle2;
    }


    // ------------------------------
    // conditional `emits` statements
    // ------------------------------

    public fun conditional(x: u64, handle: &mut EventHandle<DummyEvent>) {
        if (x > 7) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec fun conditional {
        emits DummyEvent{msg: 0} to handle if x > 7;
    }

    public fun conditional_wrong_condition_incorrect(x: u64, handle: &mut EventHandle<DummyEvent>) {
        if (x > 7) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec fun conditional_wrong_condition_incorrect {
        emits DummyEvent{msg: 0} to handle if x > 0;
    }

    public fun conditional_missing_condition_incorrect(x: u64, handle: &mut EventHandle<DummyEvent>) {
        if (x > 7) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec fun conditional_missing_condition_incorrect {
        emits DummyEvent{msg: 0} to handle;
    }


    // ----------------------------
    // `emits` statements in schema
    // ----------------------------

    public fun emits_in_schema(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec fun emits_in_schema {
        include EmitsInSchemaEmits;
    }
    spec schema EmitsInSchemaEmits {
        handle: EventHandle<DummyEvent>;
        emits DummyEvent{msg: 0} to handle;
    }
}
