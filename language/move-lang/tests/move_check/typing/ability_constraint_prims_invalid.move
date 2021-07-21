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
        // prims
        c<signer>();
        c<vector<signer>>();
        c<vector<NoC>>();
        k<u64>();
        k<signer>();
        k<vector<NoC>>();
        k<vector<NoK>>();
        cds<signer>();
        cds<vector<NoC>>();
        cds<vector<Cup<u8>>>();
        let Sc {} = Sc<signer> {};
        let Sc {} = Sc<vector<signer>> {};
        let Sc {} = Sc<vector<NoC>> {};
        let Sk {} = Sk<u64> {};
        let Sk {} = Sk<signer> {};
        let Sk {} = Sk<vector<NoC>> {};
        let Sk {} = Sk<vector<NoK>> {};
        let Scds {} = Scds<signer> {};
        let Scds {} = Scds<vector<NoC>> {};
        let Scds {} = Scds<vector<Cup<u8>>> {};
    }


}
}
