address 0x1 {

/// The Event module defines an `EventHandleGenerator` that is used to create
/// `EventHandle`s with unique GUIDs. It contains a counter for the number
/// of `EventHandle`s it generates. An `EventHandle` is used to count the number of
/// events emitted to a handle and emit events to the event store.
module Event {
    use 0x1::LCS;
    use 0x1::Signer;
    use 0x1::Vector;

    /// A resource representing the counter used to generate uniqueness under each account. There won't be destructor for
    /// this resource to guarantee the uniqueness of the generated handle.
    resource struct EventHandleGenerator {
        // A monotonically increasing counter
        counter: u64,
        addr: address,
    }

    /// A handle for an event such that:
    /// 1. Other modules can emit events to this handle.
    /// 2. Storage can use this handle to prove the total number of events that happened in the past.
    resource struct EventHandle<T: copyable> {
        /// A globally unique ID for this event stream.
        guid: vector<u8>,
    }

    /// Publishs a new event handle generator.
    public fun publish_generator(account: &signer) {
        move_to(account, EventHandleGenerator{ counter: 0, addr: Signer::address_of(account) })
    }

    /// Derive a fresh unique id by using sender's EventHandleGenerator. The generated vector<u8> is indeed unique because it
    /// was derived from the hash(sender's EventHandleGenerator || sender_address). This module guarantees that the
    /// EventHandleGenerator is only going to be monotonically increased and there's no way to revert it or destroy it. Thus
    /// such counter is going to give distinct value for each of the new event stream under each sender. And since we
    /// hash it with the sender's address, the result is guaranteed to be globally unique.
    fun fresh_guid(counter: &mut EventHandleGenerator): vector<u8> {
        let sender_bytes = LCS::to_bytes(&counter.addr);
        let count_bytes = LCS::to_bytes(&counter.counter);
        counter.counter = counter.counter + 1;

        // EventHandleGenerator goes first just in case we want to extend address in the future.
        Vector::append(&mut count_bytes, sender_bytes);

        count_bytes
    }

    /// Use EventHandleGenerator to generate a unique event handle for `sig`
    public fun new_event_handle<T: copyable>(account: &signer): EventHandle<T>
    acquires EventHandleGenerator {
        EventHandle<T> {
            guid: fresh_guid(borrow_global_mut<EventHandleGenerator>(Signer::address_of(account)))
        }
    }

    /// Emit an event with payload `msg` by using handle's key and counter. Will change the payload from vector<u8> to a
    /// generic type parameter once we have generics.
    public fun emit_event<T: copyable>(handle_ref: &mut EventHandle<T>, msg: T) {
        let guid = *&handle_ref.guid;
        write_to_event_store<T>(guid, msg);
    }

    /// Native procedure that writes to the actual event stream in Event store
    /// This will replace the "native" portion of EmitEvent bytecode
    native fun write_to_event_store<T: copyable>(guid: vector<u8>, msg: T);

    /// Destroy a unique handle.
    public fun destroy_handle<T: copyable>(handle: EventHandle<T>) {
        EventHandle<T> { guid: _ } = handle;
    }

    // ****************** SPECIFICATIONS *******************
    spec module {} // switch documentation context to module

    spec module {
        /// > NOTE: specification and verification of event related functionality is currently not happening.
        /// > Since events cannot be observed from Move programs, this does not affect the verification of
        /// > other functionality; however, this should be completed at a later point to ensure the framework
        /// > generates events as expected.
        ///
        /// Functions of the event module are mocked out using the intrinsic
        /// pragma. They are implemented in the prover's prelude as no-ops.
        pragma intrinsic = true;
    }
}

}
