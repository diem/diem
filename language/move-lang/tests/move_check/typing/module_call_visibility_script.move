address 0x2 {

module X {
    public fun f_public() {}
    public(script) fun f_script() {}
}

module Y {
    friend 0x2::M;
    public(friend) fun f_friend() {}
}

module M {
    use 0x2::X;
    use 0x2::Y;

    public fun f_public() {}
    public(friend) fun f_friend() {}
    public(script) fun f_script() {}
    fun f_private() {}

    // a public(script) fun can call public(script) funs in another module
    public(script) fun f_script_call_script() { X::f_script() }

    // a public(script) fun can call public(friend) funs in another module (subject to the
    // constraints of the friend list)
    public(script) fun f_script_call_friend() { Y::f_friend() }

    // a public(script) fun can call public funs in another module
    public(script) fun f_script_call_public() { X::f_public() }

    // a public(script) fun can call private, public, public(friend), and public(script) funs
    // defined in its own module
    public(script) fun f_script_call_self_private() { Self::f_private() }
    public(script) fun f_script_call_self_public() { Self::f_public() }
    public(script) fun f_script_call_self_friend() { Self::f_friend() }
    public(script) fun f_script_call_self_script() { Self::f_script() }
}

}
