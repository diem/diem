address 0x0:

module Debug {
    native public fun print<T>(x: &T);

    native public fun print_stack_trace();
}
