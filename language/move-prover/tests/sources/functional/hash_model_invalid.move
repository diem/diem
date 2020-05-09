module TestHash {

    use 0x0::Hash;

    spec module {
        pragma verify = true;
    }


    // sha2 test
    fun hash_test1(v1: vector<u8>, v2: vector<u8>): (vector<u8>, vector<u8>)
    {
        let h1 = Hash::sha2_256(v1);
        let h2 = Hash::sha2_256(v2);
        (h1, h2)
    }
    spec fun hash_test1 {
        aborts_if false;
        // TODO: two failing ensures seem to create non-determinism; one time the first, the next the
        // second is reported. Investigate whether this is caused by boogie_wrapper or by inherent to boogie.
        // ensures result_1 == result_2;
        ensures len(result_1) > 0 ==> result_1[0] < max_u8(); // should be <=
    }

    // sha3 test
    fun hash_test2(v1: vector<u8>, v2: vector<u8>): (vector<u8>, vector<u8>)
    {
        let h1 = Hash::sha3_256(v1);
        let h2 = Hash::sha3_256(v2);
        (h1, h2)
    }
    spec fun hash_test2 {
        aborts_if false;
        // ensures result_1 == result_2;
        ensures len(result_1) > 0 ==> result_1[0] < max_u8();
    }
}
