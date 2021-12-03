/// The Event module defines an `EventHandleGenerator` that is used to create
/// `EventHandle`s with unique GUIDs. It contains a counter for the number
/// of `EventHandle`s it generates. An `EventHandle` is used to count the number of
/// events emitted to a handle and emit events to the event store.
module Std::Event {
    use Std::BCS;
    use Std::GUID::{Self, GUID};

    /// Wrapper for a GUID for layout compatibility with legacy EventHandle id's
    // This is a hack for layout compatibility. The old EventHandle.guid was a 24 byte vector<u8>
    // created from `append(u64 counter bytes, account address bytes)`. This dummy struct mimics
    // the BCS layout of the old value.
    // Note: the reason why this works is somewhat subtle. The BCS encoding for a vector V of
    // length N is `uleb128_encoded_bytes(N) | contents(V)`
    // (see https://github.com/diem/bcs#fixed-and-variable-length-sequences).
    // uleb128_encoded_bytes(24) fits into 1 byte, so using len_bytes: u8 is what we want here
    struct GUIDWrapper has store, drop { len_bytes: u8, guid: GUID }

    /// A handle for an event such that:
    /// 1. Other modules can emit events to this handle.
    /// 2. Storage can use this handle to prove the total number of events that happened in the past.
    struct EventHandle<phantom T: drop + store> has store {
        /// Total number of events emitted to this event stream.
        counter: u64,
        /// A globally unique ID for this event stream.
        guid: GUIDWrapper,
    }

    /// Deprecated. Only kept around so Diem clients know how to deserialize existing EventHandleGenerator's
    struct EventHandleGenerator has key {
        // A monotonically increasing counter
        counter: u64,
        addr: address,
    }

    /// Use EventHandleGenerator to generate a unique event handle for `sig`
    public fun new_event_handle<T: drop + store>(account: &signer): EventHandle<T> {
        // must be 24 for compatibility with legacy Event ID's--see comment on GUIDWrapper
        let len_bytes = 24u8;
         EventHandle<T> {
            counter: 0,
            guid: GUIDWrapper { len_bytes, guid: GUID::create(account) }
        }
    }

    /// Emit an event with payload `msg` by using `handle_ref`'s key and counter.
    public fun emit_event<T: drop + store>(handle_ref: &mut EventHandle<T>, msg: T) {
        write_to_event_store<T>(BCS::to_bytes(&handle_ref.guid.guid), handle_ref.counter, msg);
        handle_ref.counter = handle_ref.counter + 1;
    }

    /// Return the GUIID associated with this EventHandle
    public fun guid<T: drop + store>(handle_ref: &EventHandle<T>): &GUID {
        &handle_ref.guid.guid
    }

    /// Log `msg` as the `count`th event associated with the event stream identified by `guid`
    native fun write_to_event_store<T: drop + store>(guid: vector<u8>, count: u64, msg: T);

    /// Destroy a unique handle.
    public fun destroy_handle<T: drop + store>(handle: EventHandle<T>) {
        EventHandle<T> { counter: _, guid: _ } = handle;
    }

    // ****************** TEST-ONLY FUNCTIONS **************

    #[test_only]
    public fun create_guid_wrapper_for_test<T: drop + store>(s: &signer): GUIDWrapper {
        let EventHandle<T> { counter: _, guid } = new_event_handle<T>(s);
        guid
    }

    // ****************** SPECIFICATIONS *******************
    spec module {} // switch documentation context to module

    spec module {
        /// Functions of the event module are mocked out using the intrinsic
        /// pragma. They are implemented in the prover's prelude.
        pragma intrinsic = true;

        /// Determines equality between the guids of two event handles. Since fields of intrinsic
        /// structs cannot be accessed, this function is provided.
        fun spec_guid_eq<T>(h1: EventHandle<T>, h2: EventHandle<T>): bool {
            // The implementation currently can just use native equality since the mocked prover
            // representation does not have the `counter` field.
            h1 == h2
        }
    }
}
