// separate_baseline: no_opaque
// separate_baseline: cvc4
// TODO(cvc4): cvc4 currently produces false positives for most of the verification problems here.
module 0x42::TestEmits {
    use Std::Event::{Self, EventHandle};

    struct DummyEvent has copy, drop, store { msg: u64 }

    // -------------------------
    // simple `emits` statements
    // -------------------------

    public fun simple(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec simple {
        emits DummyEvent{msg: 0} to handle;
    }

    public fun simple_wrong_msg_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec simple_wrong_msg_incorrect {
        emits DummyEvent{msg: 1} to handle;
    }

    public fun simple_wrong_handle_incorrect(handle: &mut EventHandle<DummyEvent>, _handle2: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec simple_wrong_handle_incorrect {
        emits DummyEvent{msg: 0} to _handle2;
    }


    // ---------------
    // multiple events
    // ---------------

    public fun multiple(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec multiple {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 1} to handle;
    }

    public fun multiple_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec multiple_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 1} to handle;
        emits DummyEvent{msg: 2} to handle;
    }

    public fun multiple_same(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec multiple_same {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 0} to handle;
    }

    public fun multiple_same_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec multiple_same_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 0} to handle;
    }

    public fun multiple_different_handle(handle: &mut EventHandle<DummyEvent>, handle2: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
        Event::emit_event(handle2, DummyEvent{msg: 0});
        Event::emit_event(handle2, DummyEvent{msg: 0});
        Event::emit_event(handle2, DummyEvent{msg: 1});
    }
    spec multiple_different_handle {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 1} to handle;
        emits DummyEvent{msg: 0} to handle2;
        emits DummyEvent{msg: 0} to handle2;
        emits DummyEvent{msg: 1} to handle2;
    }


    // ------------------------------
    // conditional `emits` statements
    // ------------------------------

    public fun conditional(x: u64, handle: &mut EventHandle<DummyEvent>) {
        if (x > 7) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec conditional {
        emits DummyEvent{msg: 0} to handle if x > 7;
    }

    public fun conditional_wrong_condition_incorrect(x: u64, handle: &mut EventHandle<DummyEvent>) {
        if (x > 7) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec conditional_wrong_condition_incorrect {
        emits DummyEvent{msg: 0} to handle if x > 0;
    }

    public fun conditional_missing_condition_incorrect(x: u64, handle: &mut EventHandle<DummyEvent>) {
        if (x > 7) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec conditional_missing_condition_incorrect {
        emits DummyEvent{msg: 0} to handle;
    }

    public fun conditional_bool(b: bool, handle: &mut EventHandle<DummyEvent>) {
        if (b) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec conditional_bool {
        emits DummyEvent{msg: 0} to handle if b;
    }

    public fun conditional_multiple(b0: bool, b1: bool, b2: bool, handle: &mut EventHandle<DummyEvent>) {
        if (b0) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        };
        if (b1) {
            Event::emit_event(handle, DummyEvent{msg: 1});
        };
        if (b2) {
            Event::emit_event(handle, DummyEvent{msg: 2});
        }
    }
    spec conditional_multiple {
        emits DummyEvent{msg: 0} to handle if b0;
        emits DummyEvent{msg: 1} to handle if b1;
        emits DummyEvent{msg: 2} to handle if b2;
    }

    public fun conditional_multiple_incorrect(b: bool, handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
        if (b) {
            Event::emit_event(handle, DummyEvent{msg: 2});
        }
    }
    spec conditional_multiple_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 1} to handle;
        emits DummyEvent{msg: 2} to handle;
    }

    public fun conditional_multiple_same(b0: bool, b1: bool, b2: bool, handle: &mut EventHandle<DummyEvent>) {
        if (b0) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        };
        if (b1) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        };
        if (b2) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec conditional_multiple_same {
        emits DummyEvent{msg: 0} to handle if b0;
        emits DummyEvent{msg: 0} to handle if b1;
        emits DummyEvent{msg: 0} to handle if b2;
    }

    public fun conditional_multiple_same_incorrect(b: bool, handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 0});
        if (b) {
            Event::emit_event(handle, DummyEvent{msg: 0});
        }
    }
    spec conditional_multiple_same_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 0} to handle;
    }


    // ----------------------------
    // `emits` statements in schema
    // ----------------------------

    public fun emits_in_schema(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
    }
    spec emits_in_schema {
        include EmitsInSchemaEmits;
    }
    spec schema EmitsInSchemaEmits {
        handle: EventHandle<DummyEvent>;
        emits DummyEvent{msg: 0} to handle;
    }

    public fun emits_in_schema_condition(handle: &mut EventHandle<DummyEvent>, x: u64) {
        if (x > 7) {
            emits_in_schema(handle)
        };
    }
    spec emits_in_schema_condition {
        include x > 7 ==> EmitsInSchemaEmits;
    }


    // ----------------------------
    // pragma emits_is_partial
    // ----------------------------

    public fun partial(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec partial {
        pragma emits_is_partial;
        emits DummyEvent{msg: 0} to handle;
    }

    public fun partial_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec partial_incorrect {
        emits DummyEvent{msg: 0} to handle;
    }


    // ----------------------------
    // pragma emits_is_strict
    // ----------------------------

    public fun strict(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec strict {
    }

    public fun strict_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec strict_incorrect {
        pragma emits_is_strict;
    }

    // ------------------------
    // calling opaque functions
    // ------------------------

    public fun callee(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 7});
        Event::emit_event(handle, DummyEvent{msg: 77});
    }
    spec callee {
        pragma opaque;
        aborts_if false;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
    }

    public fun opaque(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        callee(handle);
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec opaque {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
        emits DummyEvent{msg: 1} to handle;
    }

    public fun opaque_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        callee(handle);
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec opaque_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
        emits DummyEvent{msg: 1} to handle;
        emits DummyEvent{msg: 2} to handle;
    }

    public fun opaque_in_call_chain(handle: &mut EventHandle<DummyEvent>) {
        opaque(handle);
    }
    spec opaque_in_call_chain {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
        emits DummyEvent{msg: 1} to handle;
    }

    public fun opaque_completeness_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        callee(handle);
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec opaque_completeness_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 1} to handle;
    }


    // -------------------------------------------------
    // calling opaque functions with partial emits specs
    // -------------------------------------------------

    public fun callee_partial(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 7});
        Event::emit_event(handle, DummyEvent{msg: 77});
    }
    spec callee_partial {
        pragma opaque;
        aborts_if false;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
        pragma emits_is_partial;
    }

    public fun opaque_partial(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        callee_partial(handle);
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec opaque_partial {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
        emits DummyEvent{msg: 1} to handle;
        pragma emits_is_partial;
    }

    public fun opaque_partial_incorrect(handle: &mut EventHandle<DummyEvent>) {
        Event::emit_event(handle, DummyEvent{msg: 0});
        callee_partial(handle);
        Event::emit_event(handle, DummyEvent{msg: 1});
    }
    spec opaque_partial_incorrect {
        emits DummyEvent{msg: 0} to handle;
        emits DummyEvent{msg: 7} to handle;
        emits DummyEvent{msg: 77} to handle;
        emits DummyEvent{msg: 1} to handle;
        // The completeness check of the `emits` spec of this function should fail.
    }
}
