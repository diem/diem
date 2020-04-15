address 0x0:

module Event {
    use 0x0::LCS;
    use 0x0::Vector;
    use 0x0::Transaction;

    // An operations capability restricting who can create an
    // EventHandleGenerator.
    resource struct EventHandleGeneratorCreationCapability {}

    // A resource representing the counter used to generate uniqueness under each account. There won't be destructor for
    // this resource to guarantee the uniqueness of the generated handle.
    resource struct EventHandleGenerator {
        // A monotonically increasing counter
        counter: u64,
        addr: address,
    }

    // A handle for an event such that:
    // 1. Other modules can emit events to this handle.
    // 2. Storage can use this handle to prove the total number of events that happened in the past.
    resource struct EventHandle<T: copyable> {
        // Total number of events emitted to this event stream.
        counter: u64,
        // A globally unique ID for this event stream.
        guid: vector<u8>,
    }

    public fun grant_event_handle_creation_operation(): EventHandleGeneratorCreationCapability {
        Transaction::assert(Transaction::sender() == 0xA550C18, 0);
        EventHandleGeneratorCreationCapability{}
    }

    public fun new_event_generator(
        addr: address,
        _cap: &EventHandleGeneratorCreationCapability
    ): EventHandleGenerator {
        EventHandleGenerator{ counter: 0, addr }
    }

    // Derive a fresh unique id by using sender's EventHandleGenerator. The generated vector<u8> is indeed unique because it
    // was derived from the hash(sender's EventHandleGenerator || sender_address). This module guarantees that the
    // EventHandleGenerator is only going to be monotonically increased and there's no way to revert it or destroy it. Thus
    // such counter is going to give distinct value for each of the new event stream under each sender. And since we
    // hash it with the sender's address, the result is guaranteed to be globally unique.
    fun fresh_guid(counter: &mut EventHandleGenerator): vector<u8> {
        let sender_bytes = LCS::to_bytes(&counter.addr);
        let count_bytes = LCS::to_bytes(&counter.counter);
        counter.counter = counter.counter + 1;

        // EventHandleGenerator goes first just in case we want to extend address in the future.
        Vector::append(&mut count_bytes, sender_bytes);

        count_bytes
    }

    // Use EventHandleGenerator to generate a unique event handle that one can emit an event to.
    public fun new_event_handle_from_generator<T: copyable>(event_generator: &mut EventHandleGenerator): EventHandle<T> {
        EventHandle<T> {counter: 0, guid: fresh_guid(event_generator)}
    }


    // Use EventHandleGenerator to generate a unique event handle that one can emit an event to.
    public fun new_event_handle<T: copyable>(): EventHandle<T>
    acquires EventHandleGenerator {
        let event_generator = borrow_global_mut<EventHandleGenerator>(Transaction::sender());
        new_event_handle_from_generator(event_generator)
    }

    // Emit an event with payload `msg` by using handle's key and counter. Will change the payload from vector<u8> to a
    // generic type parameter once we have generics.
    public fun emit_event<T: copyable>(handle_ref: &mut EventHandle<T>, msg: T) {
        let guid = *&handle_ref.guid;

        write_to_event_store<T>(guid, handle_ref.counter, msg);
        handle_ref.counter = handle_ref.counter + 1;
    }

    // Native procedure that writes to the actual event stream in Event store
    // This will replace the "native" portion of EmitEvent bytecode
    native fun write_to_event_store<T: copyable>(guid: vector<u8>, count: u64, msg: T);

    // Destroy a unique handle.
    public fun destroy_handle<T: copyable>(handle: EventHandle<T>) {
        EventHandle<T> { counter: _, guid: _ } = handle;
    }
}
