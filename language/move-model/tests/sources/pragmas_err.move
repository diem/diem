module 0x42::M {
    spec module {
        // Should be fine
        pragma verify = false;

        // Should produce error that bar is not valid (not defined)
        pragma bar = true;

        // Should be ok (we do not check types of values)
        pragma verify = a_valid_id;

        // Should be ok (we do not check types of values)
        pragma verify = 0x1::M::a_valid_id;

        // Should be ok (we do not check types of values)
        pragma verify = Self::a_valid_id;

        // The below produces an error from move_lang because the relative module name cannot be resolved.
        // We leave it here for illustration.
        // pragma verify = M::a_valid_id;
    }
}
