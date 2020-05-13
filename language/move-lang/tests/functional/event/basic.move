module Holder {
    use 0x0::Event;
    resource struct Hold {
        cap: Event::EventHandleGeneratorCreationCapability,
    }

    public fun hold(cap: Event::EventHandleGeneratorCreationCapability) {
        move_to_sender(Hold{ cap })
    }
}

//! new-transaction
script {
    use {{default}}::Holder;
    use 0x0::Event;
    fun main() {
        Holder::hold(
            Event::grant_event_handle_creation_operation()
        );
    }
}
// check: ABORTED
// check: 0

//! new-transaction
script {
    use 0x0::Event;
    fun main() {
        Event::grant_event_generator()
    }
}
// check: ABORTED
// check: 0
