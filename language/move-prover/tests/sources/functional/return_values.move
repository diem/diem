module TestReturnValue {

    fun one_two(): (u64, u64) {
        (1, 2)
    }
    spec fun one_two {
        aborts_if false;
        ensures result_1 == 1;
        ensures result_2 == 2;
    }

    fun one_two_incorrect(): (u64, u64) {
        (1, 2)
    }
    spec fun one_two_wrapper_incorrect { // FIXME: `result_0` and `result_1` appear in the counterexample. `result_1` and `result_2` should instead.
        aborts_if false;
        ensures result_1 == 2;
        ensures result_2 == 1;
    }

    fun one_two_wrapper(): (u64, u64) {
        one_two()
    }
    spec fun one_two_wrapper { // FIXME: This is not verified, but should be.
        pragma verify=false; // TODO: remove this line
        aborts_if false;
        ensures result_1 == 1;
        ensures result_2 == 2;
    }

    fun one_two_wrapper_incorrect(): (u64, u64) {
        one_two()
    }
    spec fun one_two_wrapper_incorrect { // FIXME: This is verified, but should not be.
        pragma verify=false; // TODO: remove this line
        aborts_if false;
        ensures result_1 == 2;
        ensures result_2 == 1;
    }

    fun true_one(): (bool, u64) {
        (true, 1)
    }
    spec fun true_one {
        aborts_if false;
        ensures result_1 == true;
        ensures result_2 == 1;
    }

    fun true_one_incorrect(): (bool, u64) {
        (true, 1)
    }
    spec fun true_one_incorrect { // Type checker correctly complains about this spec.
        ensures result_1 == 1;
        ensures result_2 == true;
    }

    fun true_one_wrapper(): (bool, u64) {
        true_one()
    }
    spec fun true_one_wrapper {
        ensures result_1 == true;
        ensures result_2 == 1;
    }

    fun true_one_wrapper_incorrect(): (bool, u64) {
        true_one()
    }
    spec fun true_one_wrapper_incorrect {
        pragma verify=false; // TODO: remove this line
        ensures false; // FIXME: generated boogie code introduces a contradiction among assumptions, thus making "false" to be proved
        ensures result_1 == true;
        ensures result_1 == false;
        ensures result_2 == 0;
        ensures result_2 == 1;
    }

    fun another_true_one_wrapper_incorrect(): (bool, u64) {
        true_one()
    }
    spec fun another_true_one_wrapper_incorrect { // Type checker correctly complains about this spec.
        ensures result_1 == 1;
        ensures result_2 == true;
    }
}
