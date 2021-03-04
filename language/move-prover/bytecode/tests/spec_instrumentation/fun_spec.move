// Contains tests for treatment of function specifications.
module Test {

	fun implicit_and_explicit_abort(a: u64, b: u64): u64 {
	    if (b != 0) abort(22);
	    a / b
	}
	spec fun implicit_and_explicit_abort {
	    aborts_if b == 0 with 22;
	    aborts_if a == 0;
	    ensures result == a / b;
	}

	fun multiple_results(a: u64, b: u64): (u64, u64) {
	    (a / b, a % b)
	}
	spec fun multiple_results {
	    aborts_if b == 0 with EXECUTION_FAILURE;
	    ensures result_1 == a / b;
	    ensures result_2 == a % b;
	}

	fun branching_result(is_div: bool, a: u64, b: u64): u64 {
	    if (is_div) a / b else a * b
	}
	spec fun branching_result {
	    aborts_if is_div && b == 0 with EXECUTION_FAILURE;
	    ensures is_div ==> result == a / b;
	    ensures !is_div ==> result == a * b;
	}

	resource struct R { v: u64 }

	fun resource_with_old(val: u64) acquires R {
	    if (!exists<R>(0x0)) abort 33;
	    let r = borrow_global_mut<R>(0x0);
	    r.v = r.v + val;
	}
	spec fun resource_with_old {
	    requires val > 0;
	    aborts_if !exists<R>(0x0) with 33;
	    aborts_if global<R>(0x0).v + val >= 18446744073709551615;
	    ensures global<R>(0x0).v == old(global<R>(0x0)).v + val;
	    modifies global<R>(0x0);
	}

	fun ref_param(r: &R): u64 {
	    r.v
	}
	spec fun ref_param {
	    ensures result == r.v;
	}

	fun ref_param_return_ref(r: &R): &u64 {
	    &r.v
	}
	spec fun ref_param_return_ref {
	    ensures result == r.v;
	}

	fun mut_ref_param(r: &mut R): u64 {
	    let x = r.v;
	    r.v = r.v - 1;
	    x
	}
	spec fun mut_ref_param {
	    aborts_if r.v == 0 with EXECUTION_FAILURE;
	    ensures result == old(r.v);
	    ensures r.v == old(r.v) + 1;
	}

}
