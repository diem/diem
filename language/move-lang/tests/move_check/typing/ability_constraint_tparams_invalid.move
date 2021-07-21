address 0x42 {
module M {
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
    fun t<
        TnoC: drop + store + key,
        TnoK: copy + drop + store,
        T,
    >() {
        c<TnoC>();
        c<Cup<TnoK>>();
        c<Box<TnoC>>();
        k<TnoK>();
        k<Cup<TnoC>>();
        k<Box<Cup<TnoC>>>();
        cds<TnoC>();
        cds<Cup<TnoC>>();
        cds<Cup<TnoK>>();
        cds<Pair<u64, TnoC>>();
        let Sc {} = Sc<TnoC> {};
        let Sc {} = Sc<Cup<TnoK>> {};
        let Sc {} = Sc<Box<TnoC>> {};
        let Sk {} = Sk<TnoK> {};
        let Sk {} = Sk<Cup<TnoC>> {};
        let Sk {} = Sk<Box<Cup<TnoC>>> {};
        let Scds {} = Scds<TnoC> {};
        let Scds {} = Scds<Cup<TnoC>> {};
        let Scds {} = Scds<Cup<TnoK>> {};
        let Scds {} = Scds<Pair<u64, TnoC>> {};
    }
}
}
