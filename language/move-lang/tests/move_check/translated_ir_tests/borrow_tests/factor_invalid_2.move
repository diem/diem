module M {
    struct S { g: u64 }

    fun t1(root: &mut S, cond: bool) {
        let x1 = 0;
        let eps = if (cond) bar(root) else &x1;
        // Error: root has weak empty borrow and hence a field cannot be borrowed mutably
        &mut root.g;
        eps;
    }

    fun bar(a: &mut S): &u64 {
        &a.g
    }
}

// check: BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR
