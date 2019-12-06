address 0x1:

module X {
    public foo(): u64 { 0 }
}

module M {
    use 0x1::X;

    foo(): u64 { 0 }

    t0() {
        Self::fooo();
        foooo();
        X::foooooo();
    }
}
