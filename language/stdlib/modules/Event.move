address 0x1 {

/**
The Event module defines an `EventHandleGenerator` that is used to create
`EventHandle`s with unique GUIDs. It contains a counter for the number
of `EventHandle`s it generates. An `EventHandle` is used to count the number of
events emitted to a handle and emit events to the event store.
*/
module Event {
    use 0x1::LCS;
    use 0x1::Signer;
    use 0x1::Vector;

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

    public fun publish_generator(account: &signer) {
        move_to(account, EventHandleGenerator{ counter: 0, addr: Signer::address_of(account) })
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

    // Use EventHandleGenerator to generate a unique event handle for `sig`
    public fun new_event_handle<T: copyable>(account: &signer): EventHandle<T>
    acquires EventHandleGenerator {
        EventHandle<T> {
            counter: 0,
            guid: fresh_guid(borrow_global_mut<EventHandleGenerator>(Signer::address_of(account)))
        }
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

    // ****************** SPECIFICATIONS *******************

    spec module {
        /// Functions of the event module are mocked out using the intrinsic
        /// pragma. They are implemented in the prover's prelude as no-ops.
        ///
        /// Functionality in this module uses GUIDs created from serialization of
        /// addresses and integers. These constructs are difficult to treat by the
        /// verifier and the verification problem propagates up to callers of
        /// those functions. Since events cannot be observed by Move programs,
        /// mocking out functions of this module does not have effect on other
        /// verification result.
        ///
        /// A specification of the functions is neverthelesse  included in the
        /// comments of this module and it has been verified.
        ///
        /// > TODO(wrwg): We may want to have support by the Move prover to
        /// > mock out functions for callers but still have them verified
        /// > standlone.
        pragma intrinsic = true;
    }

    /*
     Specification of non-mocked out version.

    /// # Module specifications
    spec module {
        /// Turn off abortion verification of overflow of large integers (u64 and u128)
        /// for code in this module. These abortions are unlikely to happen and uninteresting,
        /// and with this pragma true, callers of this module do not need to reason about
        /// overflow of internal counters.
        pragma addition_overflow_unchecked = true;

        /// Helper function that returns whether or not an EventHandleGenerator is
        /// initilaized at the given address `addr`.
        define ehg_exists(addr: address): bool {
            exists<EventHandleGenerator>(addr)
        }
        /// Helper function that returns the EventHandleGenerator at `addr`.
        define get_ehg(addr: address): EventHandleGenerator {
            global<EventHandleGenerator>(addr)
        }
        /// Helper function that returns the serialized counter of the `EventHandleGenerator` ehg.
        define serialized_ehg_counter(ehg: EventHandleGenerator): vector<u8> {
            LCS::serialize(ehg.counter)
        }
        /// Helper function that returns the serialized address of the `EventHandleGenerator` ehg.
        define serialized_ehg_addr(ehg: EventHandleGenerator): vector<u8> {
            LCS::serialize(ehg.addr)
        }
    }

    /// ## Management of EventHandleGenerators

    spec module {
        /// `ehg_destroyed` is true whenever an `EventHandleGenerator` is destroyed.
        /// It should never to true to preserve uniqueness of EventHandleGenerators.
        global ehg_destroyed: bool;
    }
    spec struct EventHandleGenerator {
        /// Updates the ehg_destroyed variable to true if an
        /// `EventHandleGenerator` is ever unpacked.
        invariant pack ehg_destroyed = ehg_destroyed;
        invariant unpack ehg_destroyed = true;
    }
    spec schema EventHandleGeneratorNeverDestroyed {
        /// **Informally:** No `EventHandleGenerator` should ever be destroyed.
        /// Together with `EventHandleGeneratorAtSameAddress`, the `EventHandleGenerator`s
        /// can never be alternated.
        invariant module !ehg_destroyed;
    }
    spec schema EventHandleGeneratorAtSameAddress {
        /// **Informally:** An EventHandleGenerator should be located at `addr` and is never moved.
        invariant module forall addr: address where ehg_exists(addr): get_ehg(addr).addr == addr;
    }
    spec module {
        /// Apply `EventHandleGeneratorNeverDestroyed` to all functions to ensure that the
        /// `EventHandleGenerator`s are never destroyed.
        /// Together with `EHGCounterIncreasesOnEventHandleCreate`, this proves
        /// the uniqueness of the `EventHandleGenerator` resource throughout transient
        /// states at each address.
        /// Without this, the specification would allow an implementation to remove
        /// an `EventHandleGenerator`, say `ehg`, and then have it generate a number
        /// of events until the `ehg.counter` >= `old(ehg.counter)`. Violating the property
        /// that an EventHandleGenerator should only be used to generate unique GUIDs.
        ///
        /// > TODO(#4549): Potential bug. Without `* except fresh_guid;`, this
        /// takes a long time to return from the Boogie solver. Sometimes it
        /// returns a precondition violation error quickly (seemingly
        /// non-determistic). We expect all functions to satisfy this schema.
        apply EventHandleGeneratorNeverDestroyed to * except fresh_guid;

        /// Apply `EventHandleGeneratorAtSameAddress` to all functions to enforce
        /// that all `EventHandleGenerator`s reside at the address they hold.
        ///
        /// > TODO(#4549): Potential bug. Refer to the previous TODO.
        /// The solver takes a long time unless fresh_guid is excepted.
        apply EventHandleGeneratorAtSameAddress to * except fresh_guid;
    }
    spec fun publish_generator {
        /// Creates a new `EventHandleGenerator` with an initial counter 0 and the
        /// signer `account`'s address.
        aborts_if exists<EventHandleGenerator>(Signer::spec_address_of(account));
        ensures global<EventHandleGenerator>(Signer::spec_address_of(account))
                    == EventHandleGenerator { counter: 0, addr: Signer::spec_address_of(account) };
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Uniqueness and Counter Incrementation of EventHandleGenerators

    spec schema EHGCounterUnchanged {
        /// **Informally:** If an `EventHandleGenerator` exists, then it never changes
        /// except when a function generates a new `EventHandle` GUID.
        ensures forall addr: address where old(ehg_exists(addr)):
                    ehg_exists(addr) && get_ehg(addr).counter == old(get_ehg(addr).counter);
    }

    spec schema EHGCounterIncreasesOnEventHandleCreate {
        /// **Informally:** If the `EventHandleGenerator` has been initialized,
        /// then the counter never decreases and stays initialized.
        /// This proves the uniqueness of the `EventHandleGenerator`s at entry and
        /// exit of functions in this module.
        ///
        /// > TODO(#4549): Unable to prove thaat the counter increments using schemas.
        /// However, we can prove the that the `EventHandleGenerator` increments
        /// in the post conditions of the functions.
        ///
        /// Base case:
        ensures forall addr: address where !old(ehg_exists(addr)) && ehg_exists(addr):
                    get_ehg(addr).counter == 0;
        /// Induction step:
        ///
        /// > TODO(kkmc): Change `counter >= old(counter)` to `counter == old(counter) + 1`
        /// Currently, this can be proved at the function level but not the global invariant level.
        ensures forall addr: address where old(ehg_exists(addr)):
                    ehg_exists(addr) && get_ehg(addr).counter >= old(get_ehg(addr).counter);
    }
    spec module {
        /// Apply `EHGCounterUnchanged` to all functions except for those
        /// that create a new GUID and increment the `EventHandleGenerator` counter.
        apply EHGCounterUnchanged to * except fresh_guid, new_event_handle;
        /// Apply `EHGCounterIncreasesOnEventHandleCreate` to all functions except fresh_guid
        /// and its callees.
        apply EHGCounterIncreasesOnEventHandleCreate to *;
    }
    spec fun fresh_guid {
        /// The byte array returned is the concatenation of the serialized
        /// EventHandleGenerator counter and address.
        ensures counter.counter == old(counter).counter + 1;
        ensures Vector::eq_append(
                    result,
                    old(serialized_ehg_counter(counter)),
                    old(serialized_ehg_addr(counter))
                );
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Uniqueness of EventHandle GUIDs

    spec schema UniqueEventHandleGUIDs {
        /// **Informally:** All `EventHandle`s have unqiue GUIDs.
        ///
        /// There are several ways we may want to express this invariant:
        ///
        /// INV 1: The first invariant is that all `EventHandle`s have unique GUIDs.
        /// An intuitive way to write this is to keep a list of previously existing `EventHandle`s
        /// and for every `pack` of an `EventHandle`, the GUID must be different from all the
        /// previously existing ones. However, this requires us to keep track of all of the
        /// newly generated `EventHandle` resources (possibly through a ghost variable) and
        /// compare their GUIDs to each other, which is currently not possible.
        ///
        /// INV 2: Enforce `fresh_guid` to only return unique GUIDs; every call only increases the counter
        /// of the EventHandleGenerator and the output of the `fresh_guid`.
        ///
        /// > TODO(kkmc): The move-prover does not currently encode the property that LCS::serialize
        /// returns the same number of bytes for the same primitive types. Which means that
        /// `fresh_guid` can return the same GUIDs.
        ///
        /// E.g. If LCS::serialize(addr1) == <0,1,2>, LCS::serialize(addr2) == <0,1>,
        ///         LCS::serialize(ctr1) == <3>, LCS::serialize(ctr2) == <2,3>,
        /// then <0,1,2> appended with <3> is equal to <0,1> appended with <2,3>.
        invariant module true;
    }
    spec module {
        /// Apply `UniqueEventHandleGUIDs` to enforce all `EventHandle` resources to have unique GUIDs.
        apply UniqueEventHandleGUIDs to *;
    }
    spec fun new_event_handle {
        aborts_if !ehg_exists(Signer::spec_address_of(account));
        aborts_if get_ehg(Signer::spec_address_of(account)).counter + 1 > max_u64();
        ensures ehg_exists(Signer::spec_address_of(account));
        ensures get_ehg(Signer::spec_address_of(account)).counter ==
                    old(get_ehg(Signer::spec_address_of(account)).counter) + 1;
        ensures result.counter == 0;
        ensures Vector::eq_append(
                    result.guid,
                    old(serialized_ehg_counter(get_ehg(Signer::spec_address_of(account)))),
                    old(serialized_ehg_addr(get_ehg(Signer::spec_address_of(account))))
                );
    }

    // Switch documentation context back to module level.
    spec module {}

    spec fun emit_event {
        /// The counter in `EventHandle<T>` increases and the event is emitted to the event store.
        ///
        /// > TODO(kkmc): Do we need to specify that the event was sent to the event store?
        ensures handle_ref.counter == old(handle_ref.counter) + 1;
        ensures handle_ref.guid == old(handle_ref.guid);
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Destruction of EventHandles

    spec module {
        /// Variable that counts the total number of event handles ever to exist.
        global total_num_of_event_handles<T>: num;
    }
    spec struct EventHandle {
        /// Count the total number of `EventHandle`s.
        /// This is used in the post condition of `destroy_handle` to ensure that an
        /// `EventHandle` is destroyed.
        invariant pack total_num_of_event_handles<T> = total_num_of_event_handles<T> + 1;
        invariant unpack total_num_of_event_handles<T> = total_num_of_event_handles<T> - 1;
    }
    spec fun destroy_handle {
        /// `destroy_handle` should have unpacked an `EventHandle` and thereby decreasing the
        /// total number of `EventHandle`s.
        aborts_if false;
        ensures total_num_of_event_handles<T> == old(total_num_of_event_handles<T>) - 1;
    }
    */
}

}
