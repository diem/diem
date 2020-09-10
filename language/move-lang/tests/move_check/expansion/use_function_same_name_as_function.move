address 0x2 {
module X {
    public fun u(): u64 {
        0
    }
}

module M {
    use 0x2::X::u;
    fun u() {
    }
}

module N {
    fun bar() {
    }
    use 0x2::X::u as bar;
}

}
