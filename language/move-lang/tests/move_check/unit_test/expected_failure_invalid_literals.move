// check that only non-annotated integer literals and u64s can be assigned to
// abort codes
address 0x1 {
module M {
    #[test]
    #[expected_failure(abort_code=0)]
    fun ok1() { }

    #[test]
    #[expected_failure(abort_code=0u64)]
    fun ok2() { }

    #[test]
    #[expected_failure(abort_code=0u8)]
    fun fail_annot1() { }

    #[test]
    #[expected_failure(abort_code=0u128)]
    fun fail_annot3() { }
}
}
