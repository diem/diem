// flag: --check-inconsistency
// flag: --unconditional-abort-as-inconsistency
module 0x42::Inconsistency {
    // There is an inconsistent assumption in the verification of this function
    // because it always aborts.
    fun always_abort() {
        abort 0
    }
    spec always_abort {
        ensures false;
    }

    // Hiding the function behind some trivial if-else branch does not work
    fun always_abort_if_else(x: u64): bool {
        if (x == x) {
            abort 0
        } else {
            return true
        }
    }
    spec always_abort_if_else {
        ensures result == false;
    }
}
