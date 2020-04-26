module TestLCS {

    use 0x0::LCS;

    fun lcs_test1<Thing>(v1: &Thing, v2: &Thing): (vector<u8>, vector<u8>)
    {
        let s1 = LCS::to_bytes(v1);
        let s2 = LCS::to_bytes(v2);
        (s1, s2)
    }
    spec fun lcs_test1 {
        aborts_if false;
        ensures result_1 == result_2 ==> v1 == v2;
        ensures v1 == v2 ==> result_1 == result_2;
        // it knows result is vector<u8>
        ensures len(result_1) > 0 ==> result_1[0] <= max_u8();
    }

    // serialize tests

    fun lcs_test1_incorrect<Thing>(v1: &Thing, v2: &Thing): (vector<u8>, vector<u8>)
    {
        let s1 = LCS::to_bytes(v1);
        let s2 = LCS::to_bytes(v2);
        (s1, s2)
    }
    spec fun lcs_test1_incorrect {
        aborts_if false;
        ensures result_1 == result_2;
        ensures len(result_1) > 0;
        // it knows result is vector<u8>
        // TODO: the below introduces non-determinism so commented out for now
        // ensures result_1[0] < max_u8();
    }

}
