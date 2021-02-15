address 0x42 {
module A {}

module M {
    use 0x42::A as AliasedA;
    friend 0x42::A;
    friend AliasedA;
}
}
