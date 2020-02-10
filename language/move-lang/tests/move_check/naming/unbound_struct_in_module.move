address 0x1:
module X {}
module M {
    use 0x1::X;

    fun foo(s: X::S): X::S {
        let s = s;
        s
    }
}
