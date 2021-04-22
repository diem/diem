address 0x1 {
module UpdateReturn {

    struct S has key { f: u64 }

    public fun write_f(s: &mut S, x: u64): u64 {
        s.f = 7;
        x
    }

    // this function will update the return value via a join
    public fun abort_or_write(s: &mut S, b: bool, x: u64): u64 {
        if (b) abort(77);
        write_f(s, x)
    }
}
}
