address 0x1 {
module X {
    public fun u(): u64 {
        0
    }
}

module M {
    use 0x1::X::u;
    struct u {}
}

module N {
    struct Bar {}
    use 0x1::X::u as Bar;
}

}
