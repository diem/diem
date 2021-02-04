address 0x2 {

module X {
    public fun f_public() {}
    public(script) fun f_script() {}
}

module M {
    use 0x2::X;

    public fun f_public() {}
    public fun f_private() {}

    // a public(script) fun can call public(script) funs in another module
    public(script) fun f_script_call_script() { X::f_script() }

    // a public(script) fun can call public funs in another module
    public(script) fun f_script_call_public() { X::f_public() }

    // a public(script) fun can call private, public, and public(script) funs in its own module
    public(script) fun f_script_call_self_private() { Self::f_private() }
    public(script) fun f_script_call_self_public() { Self::f_public() }
    public(script) fun f_script_call_self_script() { Self::f_script_call_script() }
}

}
