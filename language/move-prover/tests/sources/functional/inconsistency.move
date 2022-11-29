// flag: --check-inconsistency
// separate_baseline: no_opaque
module 0x42::Inconsistency {


    // There is no inconsistency in this function.
    fun dec(x: u64): u64 {
        x - 1
    }
    spec dec {
        aborts_if x == 0;
        ensures result == x - 1;
    }

    // There is an inconsistent assumption in the verification of this function
    // because it assumes false.
    fun assume_false(x: u64): u64 {
        spec {
            assume false;
        };
        dec(x)
    }
    spec assume_false {
        aborts_if x == 0;
        ensures result == x - 1;
        ensures false;
    }

    // This opaque function has the false post-condition, so introduces an inconsistency.
    fun inconsistent_opaque() {
    }
    spec inconsistent_opaque {
        pragma verify=false;
        pragma opaque;
        ensures false;
    }

    // There is an inconsistent assumption in the verification of this function
    // because it calls an inconsistent opaque function.
    fun call_inconsistent_opaque() {
        inconsistent_opaque();
    }
    spec call_inconsistent_opaque {
        ensures false;
    }
}
