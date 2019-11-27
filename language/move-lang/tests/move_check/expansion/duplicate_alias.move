address 0x1:

module X {}
module Y {}

module M {
    use 0x1::X as Z;
    use 0x1::Y as Z;
}
