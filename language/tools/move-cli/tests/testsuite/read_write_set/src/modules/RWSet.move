address 0x1 {
module RWSet {
    struct S { f: u64 }

    public fun read_f(s: &S): u64 {
        s.f
    }

    public fun write_f(s: &mut S) {
        s.f = 7;
    }
}
}
