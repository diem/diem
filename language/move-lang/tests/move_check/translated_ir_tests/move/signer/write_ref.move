module 0x8675309::M {
    fun t(sref: &mut signer, s: signer) {
        *sref = s;
    }
}

// check: WRITEREF_RESOURCE_ERROR
