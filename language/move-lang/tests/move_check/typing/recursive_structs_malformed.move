address 0x42 {

module M0 {
    struct Foo { f: (Foo, Foo) }
    struct Bar { f: &Bar }
    struct Baz { f: vector<(&Baz, &mut Baz)> }
}

}
