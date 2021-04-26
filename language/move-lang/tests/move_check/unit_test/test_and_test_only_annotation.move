// functions cannot be annotated as both #[test] and #[test_only]
address 0x1 {
module M {
    #[test(_a=@0x1, _b=@0x2)]
    #[test_only]
    public fun boo(_a: signer, _b: signer) { }
}
}
