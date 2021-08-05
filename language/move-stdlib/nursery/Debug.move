/// Module providing debug functionality.
module Std::Debug {
    native public fun print<T>(x: &T);

    native public fun print_stack_trace();
}
