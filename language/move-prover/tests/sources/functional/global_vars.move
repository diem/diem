module 0x42::TestGlobalVars {

    use Std::Signer;

    // ================================================================================
    // Counting

    spec module {
        global sum_of_T: u64 = 0;
    }

    struct T has key {
      i: u64,
    }

    fun add() acquires T {
        borrow_global_mut<T>(@0).i = borrow_global_mut<T>(@0).i + 1
    }
    spec add {
        update sum_of_T = sum_of_T + 1;
    }

    fun sub() acquires T {
        borrow_global_mut<T>(@0).i = borrow_global_mut<T>(@0).i - 1
    }
    spec sub {
        update sum_of_T = sum_of_T - 1;
    }

    fun call_add_sub() acquires T {
        add(); add(); sub();
    }
    spec call_add_sub {
        ensures sum_of_T == 1;
    }

    fun call_add_sub_invalid() acquires T {
        add(); sub(); add();
    }
    spec call_add_sub_invalid {
        ensures sum_of_T == 2;
    }

    // ================================================================================
    // Access Control

    spec module {
        // Indicates whether a specific access has been verified. This is kept intentionally
        // uninitialized so the prover will find situations where this is false but access is required.
        global access_verified: bool;
    }

    fun assert_access(s: &signer) {
        // Do some assertions which validate access
        assert(Signer::address_of(s) == @0, 1);
    }
    spec assert_access {
        aborts_if Signer::spec_address_of(s) != @0;
        update access_verified = true;
    }

    fun requires_access() {
        // Do some things which require access to be validated.
    }
    spec requires_access {
        requires access_verified;
    }

    fun do_privileged(s: &signer) {
        assert_access(s);
        requires_access();
    }

    fun do_privileged_invalid(_s: &signer) {
        requires_access();
    }

    // ================================================================================
    // Generic spec vars

    spec module {
        global type_has_property<X>: bool;
    }

    fun give_property_to<X>() {
    }
    spec give_property_to {
        update type_has_property<X> = true;
    }

    fun expect_property_of_bool() {
        give_property_to<bool>();
    }
    spec expect_property_of_bool {
        ensures type_has_property<bool>;
    }

    fun expect_property_of_u64_invalid() {
        give_property_to<bool>();
    }
    spec expect_property_of_u64_invalid {
        ensures type_has_property<u64>;
    }

    // ================================================================================
    // Invariants and spec vars

    spec module {
        global limit: num = 2;
    }

    struct R has key { v: u64 }

    invariant global<R>(@0).v <= limit;

    fun publish(s: &signer) {
        move_to<R>(s, R{v: 2});
    }
    spec publish {
        // TODO: this hack is currently required because spec_instrumentation does not inject assumptions for
        //   memory used only by invariants which are injected into the code, but not the original code.
        //   This is a general bug not related to spec vars. Since the function `publish` does not directly
        //   reference `limit, but only the invariant which is injected, memory usage analysis does not
        //   report this dependency on limit. The below forces the analysis to see this dependency.
        ensures limit == limit;
    }

    fun update_invalid() acquires R {
        borrow_global_mut<R>(@0).v = 3;
    }
    spec update_invalid {
        // TODO: see above
        ensures limit == limit;
    }

    fun limit_change_invalid(s: &signer) {
        publish(s);
    }
    spec limit_change_invalid {
        update limit = 1;
    }

    // ================================================================================
    // TODO: implement and test opaque
}
