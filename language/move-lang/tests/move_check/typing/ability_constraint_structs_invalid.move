address 0x42 {
module M {
    struct NoC has drop, store, key {}
    struct NoK has copy, drop, store {}
    struct Cup<T> { f: T }
    struct Box<T> has copy, drop, store, key { f: T }
    struct Pair<T1, T2> has copy, drop, store, key { f1: T1, f2: T2 }

    fun c<T: copy>() {}
    fun k<T: key>() {}
    fun cds<T: copy + drop + store>() {}

    struct Sc<phantom T: copy> {}
    struct Sk<phantom T: key> {}
    struct Scds<phantom T: copy + drop + store> {}

    // tests that a variety of constraint instantiations are all invalid
    fun t() {
        c<NoC>();
        c<Cup<u64>>();
        c<Box<NoC>>();
        k<NoK>();
        k<Cup<u64>>();
        k<Box<Cup<u64>>>();
        cds<NoC>();
        cds<Cup<u64>>();
        cds<Cup<NoC>>();
        cds<Pair<u64, NoC>>();
        let Sc {} = Sc<NoC> {};
        let Sc {} = Sc<Cup<u64>> {};
        let Sc {} = Sc<Box<NoC>> {};
        let Sk {} = Sk<NoK> {};
        let Sk {} = Sk<Cup<u64>> {};
        let Sk {} = Sk<Box<Cup<u64>>> {};
        let Scds {} = Scds<NoC> {};
        let Scds {} = Scds<Cup<u64>> {};
        let Scds {} = Scds<Cup<NoC>> {};
        let Scds {} = Scds<Pair<u64, NoC>> {};
    }


}
}
