address 0x2 {

module X {}
module Y {}

module M {
    use 0x2::X as Z;
    use 0x2::Y as Z;
}

}
