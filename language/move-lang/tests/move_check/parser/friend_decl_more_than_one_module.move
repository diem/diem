address 0x42 {
module A {}
module B {}

module M {
    use 0x42::A;
    friend A 0x42::B;
}
}
