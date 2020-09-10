module TestTypeValues {

    spec module {
        pragma verify = true;
    }

    fun simple_type_equality<T>(): bool {
        true
    }
    spec fun simple_type_equality {
        ensures result == (type<T>() == type<T>());
    }

    fun simple_type_equality_incorrect<T1, T2>(): bool {
        true
    }
    spec fun simple_type_equality_incorrect {
        ensures result == (type<T1>() != type<T2>());
    }

    resource struct R<T> { x: u64 }

    spec module {
        // Quantify over the domain of types, passing the type value to a helper function.
        invariant forall t: type : resource_invariant_globally_defined(t);

        // Quantify over the domain of addresses, and take the type parameter to check a resource.
        define resource_invariant_globally_defined(t: type): bool {
            forall addr: address where exists<R<t>>(addr) : global<R<t>>(addr).x >= 1
        }
    }

    public fun add_R<T>(account: &signer) {
        move_to(account, R<T>{x: 1})
    }

    public fun add_R_incorrect<T>(account: &signer) {
        move_to(account, R<T>{x: 0})
    }
}
