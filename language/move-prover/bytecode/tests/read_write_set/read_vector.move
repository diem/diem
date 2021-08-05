// dep: ../../move-stdlib/modules/Vector.move

module 0x2::ReadVector {
    use Std::Vector;

    struct S has drop { a: vector<address> }
    struct Glob has key { b: bool }

    fun extract_addr_from_vec(s: S): address {
        let S { a } = s;
        *Vector::borrow(&a, 0)
    } // ret |-> { Formal(0)/a/0 }

    fun read_addr_from_vec(s: S): bool acquires Glob {
        let a = extract_addr_from_vec(s);
        *&borrow_global<Glob>(a).b
    }
}
