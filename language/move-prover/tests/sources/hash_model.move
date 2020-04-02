// dep: tests/sources/stdlib/modules/hash.move

module TestHash {

    use 0x0::Hash;

    // sha2 tests

    fun hash_test1(v1: vector<u8>, v2: vector<u8>): (vector<u8>, vector<u8>)
    {
        let h1 = Hash::sha2_256(v1);
        let h2 = Hash::sha2_256(v2);
        (h1, h2)
    }
    spec fun hash_test1 {
        aborts_if false;
        ensures result_1 == result_2 ==> v1 == v2;
        ensures v1 == v2 ==> result_1 == result_2;
        ensures len(result_1) == 32;
        // it knows result is vector<u8>
        ensures len(result_1) > 0 ==> result_1[0] <= max_u8();
    }

    fun hash_test2(v1: vector<u8>, v2: vector<u8>): bool
    {
        let h1 = Hash::sha2_256(v1);
        let h2 = Hash::sha2_256(v2);
        h1 == h2
    }
    spec fun hash_test2 {
        aborts_if false;
        ensures result == (v1 == v2);
    }

    // sha3 tests
    fun hash_test3(v1: vector<u8>, v2: vector<u8>): (vector<u8>, vector<u8>)
    {
        let h1 = Hash::sha3_256(v1);
        let h2 = Hash::sha3_256(v2);
        (h1, h2)
    }
    spec fun hash_test3 {
        aborts_if false;
        ensures result_1 == result_2 ==> v1 == v2;
        ensures v1 == v2 ==> result_1 == result_2;
        ensures len(result_1) == 32;
        // it knows result is vector<u8>
        ensures len(result_1) > 0 ==> result_1[0] <= max_u8();
    }

    fun hash_test4(v1: vector<u8>, v2: vector<u8>): bool
    {
        let h1 = Hash::sha3_256(v1);
        let h2 = Hash::sha3_256(v2);
        h1 == h2
    }
    spec fun hash_test4 {
        aborts_if false;
        ensures result == (v1 == v2);
    }
}
