module 0x42::TestReturnValue {

    public fun one_two(): (u64, u64) {
        (1, 2)
    }
    spec one_two {
        aborts_if false;
        ensures result_1 == 1;
        ensures result_2 == 2;
    }

    public fun one_two_incorrect(): (u64, u64) {
        (1, 2)
    }
    spec one_two_wrapper_incorrect {
        aborts_if false;
        ensures result_1 == 2;
        ensures result_2 == 1;
    }

    public fun one_two_wrapper(): (u64, u64) {
        one_two()
    }
    spec one_two_wrapper {
        aborts_if false;
        ensures result_1 == 1;
        ensures result_2 == 2;
    }

    public fun one_two_wrapper_incorrect(): (u64, u64) {
        one_two()
    }
    spec one_two_wrapper_incorrect {
        aborts_if false;
        ensures result_1 == 2;
        ensures result_2 == 1;
    }

    public fun true_one(): (bool, u64) {
        (true, 1)
    }
    spec true_one {
        aborts_if false;
        ensures result_1 == true;
        ensures result_2 == 1;
    }

    public fun true_one_wrapper(): (bool, u64) {
        true_one()
    }
    spec true_one_wrapper {
        ensures result_1 == true;
        ensures result_2 == 1;
    }

    public fun true_one_wrapper_incorrect(): (bool, u64) {
        true_one()
    }
    spec true_one_wrapper_incorrect {
        ensures false;
        ensures result_1 == true;
        ensures result_1 == false;
        ensures result_2 == 0;
        ensures result_2 == 1;
    }
}
