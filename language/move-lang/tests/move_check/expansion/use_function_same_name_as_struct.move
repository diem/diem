address 0x2 {
module X {
    public fun u(): u64 {
        0
    }
}

module M {
    use 0x2::X::u;
    struct u {}
}

module N {
    struct Bar {}
    use 0x2::X::u as Bar;
}

}
