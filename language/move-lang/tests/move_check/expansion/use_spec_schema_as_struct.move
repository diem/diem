address 0x1 {
module X {
    spec schema Foo<T> {
        ensures true;
    }
}

module M {
    use 0x1::X::Foo;
    fun t(): Foo<u64> {
        abort 0
    }
}

}
