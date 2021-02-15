address 0x42 {
module A {}

module M {
    friend 0x42::A
}
}
