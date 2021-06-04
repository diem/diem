module 0x42::M {

    // This schema creates an error.
    spec schema ErroneousSchema {
        ensures x > 0;
    }

    // This function includes the schema with the error, but also has its own error. We expect the own error
    // to be reported and no issue with the use of erroneous schema.
    fun foo(x: u64): u64 { x }
    spec foo {
        include ErroneousSchema;
        ensures 2 < true;
    }
}
