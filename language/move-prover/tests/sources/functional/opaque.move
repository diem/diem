// separate_baseline: no_opaque
module 0x42::TestOpaque {

    spec module {
        pragma verify = true;
    }

    // Function which has a wrong post condition, so verification fails.
    fun opaque_incorrect(): u64 {
        1
    }
    spec opaque_incorrect {
        pragma opaque = true;
        ensures result == 2;
    }

    fun opaque_caller(): u64 {
        opaque_incorrect()
    }
    spec opaque_caller {
        // because we only use the post condition but not the definition, this should verify
        ensures result == 2;
    }
}
