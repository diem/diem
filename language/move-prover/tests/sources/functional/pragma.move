// Tests pragmas.
module 0x42::TestPragma {


    spec module {
        // if not specified otherwise, verification is off.
        pragma verify=false;
    }

    fun always_aborts_with_verify_incorrect(_c: bool) {
        abort(1)
    }
    spec always_aborts_with_verify_incorrect {
        pragma verify=true;
        aborts_if _c;
    }

    fun always_aborts_without_verify(_c: bool) {
        abort(1)
    }
    spec always_aborts_without_verify {
        // Will not be flagged because we have verify=false on module level
        aborts_if _c;
    }
}
