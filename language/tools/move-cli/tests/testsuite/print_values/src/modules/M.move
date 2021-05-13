address 0x2 {
module M {
    use Std::Debug;
    use Std::Vector;

    struct Foo has drop {}
    struct Bar has drop { x: u128, y: Foo, z: bool }
    struct Box<T> has drop { x: T }

    public fun test()  {
        let x = 42;
        Debug::print(&x);

        let v = Vector::empty();
        Vector::push_back(&mut v, 100);
        Vector::push_back(&mut v, 200);
        Vector::push_back(&mut v, 300);
        Debug::print(&v);

        let foo = Foo {};
        Debug::print(&foo);

        let bar = Bar { x: 404, y: Foo {}, z: true };
        Debug::print(&bar);

        let box = Box { x: Foo {} };
        Debug::print(&box);
    }
}
}
