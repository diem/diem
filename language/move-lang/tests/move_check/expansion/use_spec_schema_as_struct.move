address 0x2 {
module X {
    spec schema Foo<T> {
        ensures true;
    }
}

module M {
    use 0x2::X::Foo;
    fun t(): Foo<u64> {
        abort 0
    }
}

}
