address 0x2 {
module X {}
module M {
    use 0x2::X;

    fun foo(s: X::S): X::S {
        let s = s;
        s
    }
}
}
