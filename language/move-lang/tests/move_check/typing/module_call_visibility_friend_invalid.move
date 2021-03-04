address 0x2 {

module X {
    fun f_private() {}
    public(friend) fun f_friend() {}
}

module Y {
    friend 0x2::M;
    fun f_private() {}
}

module M {
    use 0x2::X;
    use 0x2::Y;

    // a public(friend) fun cannot call friend funs in other modules if not being in the friend list
    public(friend) fun f_friend_call_friend() { X::f_friend() }

    // a public(friend) fun cannot call private funs in other modules, regardless of whether being
    // in the friend list of not.
    public(friend) fun f_friend_call_private_1() { X::f_private() }
    public(friend) fun f_friend_call_private_2() { Y::f_private() }
}

}
