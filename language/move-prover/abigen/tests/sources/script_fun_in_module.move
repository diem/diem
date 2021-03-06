address 0x1 {
module ScriptFunInModule {
    struct NoCall has drop {}

    /// This is a doc comment on this script fun
    public(script) fun this_is_a_script_fun(this_is_an_arg: u64, _another_arg: address) {
        abort this_is_an_arg
    }

    /// This is another doc comment on a different script fun
    public(script) fun this_is_a_different_script_fun(this_is_an_arg: u64, _another_arg: address) {
        abort this_is_an_arg
    }

    /// This is a comment on a non-callable script function
    public(script) fun this_is_a_noncallable_script_fun(): u64 {
        5
    }

    /// This is a comment on a non-callable script function
    public(script) fun this_is_another_noncallable_script_fun(_blank: NoCall) { }

    public fun foo() { }

    fun bar() { }
}
}
