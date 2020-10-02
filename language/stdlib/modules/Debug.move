address 0x1 {

/// Module providing debug functionality.
module Debug {
    native public fun print<T>(x: &T);

    native public fun print_stack_trace();
}

}
