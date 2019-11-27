address 0x1:

module X {}

module M {
    use 0x1::X;
    foo(x: X) {}
}
