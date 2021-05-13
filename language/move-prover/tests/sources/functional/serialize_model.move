// separate_baseline: cvc4
// TODO(cvc4): cvc4 produces a false positive
module 0x42::TestBCS {
    use Std::BCS;

    spec module {
        pragma verify = true;
    }

    fun bcs_test1<Thing>(v1: &Thing, v2: &Thing): (vector<u8>, vector<u8>)
    {
        let s1 = BCS::to_bytes(v1);
        let s2 = BCS::to_bytes(v2);
        (s1, s2)
    }
    spec bcs_test1 {
        aborts_if false;
        ensures result_1 == result_2 ==> v1 == v2;
        ensures v1 == v2 ==> result_1 == result_2;
        // it knows result is vector<u8>
        ensures len(result_1) > 0 ==> result_1[0] <= max_u8();
    }

    // serialize tests

    fun bcs_test1_incorrect<Thing>(v1: &Thing, v2: &Thing): (vector<u8>, vector<u8>)
    {
        let s1 = BCS::to_bytes(v1);
        let s2 = BCS::to_bytes(v2);
        (s1, s2)
    }
    spec bcs_test1_incorrect {
        aborts_if false;
        ensures result_1 == result_2;
        ensures len(result_1) > 0;
        // it knows result is vector<u8>
        // TODO: the below introduces non-determinism so commented out for now
        // ensures result_1[0] < max_u8();
    }

}
