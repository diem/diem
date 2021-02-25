address 0x1 {
module Funtions {
    struct R has key { v: vector<u8> }

    public fun id(a: address): address { a }

    public fun id_ref(a: &address): &address { a }

    public fun id_generic<T>(t: T): T { t }

    public fun id_ref_generic<T>(t: &T): &T { t }

    public fun choice(a: vector<address>, b: vector<address>, c: bool): vector<address> {
        if (c) { a } else { b }
    }

    public fun write_vec(r: &mut R, v: vector<u8>) {
        r.v = v
    }

    public fun call_write_vec(a: address, v: vector<u8>) acquires R {
        write_vec(borrow_global_mut<R>(a), v)
    }

}
}
