address 0x1 {
module X {
    public fun u(): u64 {
        0
    }
}

module M {
    use 0x1::X::u;
    fun u() {
    }
}

module N {
    fun bar() {
    }
    use 0x1::X::u as bar;
}

}
