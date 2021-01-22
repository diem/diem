module M {
    fun t(sref: &mut signer, s: signer) {
        *sref = s;
    }
}

// check: WRITEREF_RESOURCE_ERROR
