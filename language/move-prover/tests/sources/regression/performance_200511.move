// A test case which reproduces a performance/non-termination problem. See the spec of fun create for details.

module Test {
    use 0x0::Transaction;
    use 0x0::LCS;
    use 0x0::Vector;

    spec module {
        define eq_append<Element>(v: vector<Element>, v1: vector<Element>, v2: vector<Element>): bool {
            len(v) == len(v1) + len(v2) &&
            v[0..len(v1)] == v1 &&
            v[len(v1)..len(v)] == v2
        }
    }

    struct EventHandle<T: copyable> {
        // Total number of events emitted to this event stream.
        counter: u64,
        // A globally unique ID for this event stream.
        guid: vector<u8>,
    }

    struct Event1 {}
    struct Event2 {}

    resource struct T {
        received_events: EventHandle<Event1>,
        sent_events: EventHandle<Event2>,
    }

    struct EventHandleGenerator {
        // A monotonically increasing counter
        counter: u64,
    }

    fun fresh_guid(counter: &mut EventHandleGenerator, sender: address): vector<u8> {
        let sender_bytes = LCS::to_bytes(&sender);
        let count_bytes = LCS::to_bytes(&counter.counter);
        counter.counter = counter.counter + 1;

        Vector::append(&mut count_bytes, sender_bytes);

        count_bytes
    }
    spec fun fresh_guid {
        aborts_if counter.counter + 1 > max_u64();
        ensures eq_append(result, LCS::serialize(old(counter.counter)), LCS::serialize(sender));
    }

    fun new_event_handle_impl<T: copyable>(counter: &mut EventHandleGenerator, sender: address): EventHandle<T> {
        EventHandle<T> {counter: 0, guid: fresh_guid(counter, sender)}
    }
    spec fun new_event_handle_impl {
        aborts_if counter.counter + 1 > max_u64();
        ensures eq_append(result.guid, LCS::serialize(old(counter.counter)), LCS::serialize(sender));
        ensures result.counter == 0;
    }

    public fun create(fresh_address: address, auth_key_prefix: vector<u8>) : vector<u8> {
        let generator = EventHandleGenerator{counter: 0};
        let authentication_key = auth_key_prefix;
        Vector::append(&mut authentication_key, LCS::to_bytes(&fresh_address));
        Transaction::assert(Vector::length(&authentication_key) == 32, 12);


        move_to_sender<T>(T{
            received_events: new_event_handle_impl<Event1>(&mut generator, fresh_address),
            sent_events: new_event_handle_impl<Event2>(&mut generator, fresh_address)
        });

        authentication_key
    }
    spec fun create {
        // To reproduce, put this to true.
        pragma verify=true;

        // The next two aborts_if and ensures are correct. However, if they are removed, verification terminates
        // with the expected result.
        aborts_if len(LCS::serialize(fresh_address)) + len(auth_key_prefix) != 32;
        aborts_if exists<T>(sender());
        ensures eq_append(result, auth_key_prefix, LCS::serialize(fresh_address));

        // These two ensures are wrong and should produce an error. Instead, the solver hangs without bounding
        // serialization result size. To reproduce, set --serialize-bound=0 to remove any bound.
        ensures eq_append(global<T>(sender()).received_events.guid, LCS::serialize(2), LCS::serialize(fresh_address));
        ensures eq_append(global<T>(sender()).sent_events.guid, LCS::serialize(3), LCS::serialize(fresh_address));

        // Correct version of the above ensures:
        //ensures eq_append(global<T>(sender()).received_events.guid, LCS::serialize(0), LCS::serialize(fresh_address));
        //ensures eq_append(global<T>(sender()).sent_events.guid, LCS::serialize(1), LCS::serialize(fresh_address));
    }
}
