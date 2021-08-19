module 0x1::M1 {
    use 0x1::M2::C;

    struct A<T> { f: u64, v: vector<u8>, b: B<T> }

    struct B<T> { a: address, c: C<T>, t: T }

    struct S<T> { t: T }

    struct G { x: u64, s: S<bool> }
}
