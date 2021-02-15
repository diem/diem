address 0x42 {
module A {}
module B {}
module C {}

module M {
    // friend with fully qualified module id
    friend 0x42::A;

    // friend with imported name
    use 0x42::B;
    friend B;

    // friend with asliased name
    use 0x42::C as AliasedC;
    friend AliasedC;
}
}
