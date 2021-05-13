module 0x42::TestSet {
    use Std::Vector;

    // Issue #4872.
    public fun remove_everything(v: &mut vector<u64>) {
        let i = Vector::length(v);
        while ({
            spec {
                // The following doesn't work
                // assert 0 <= i && i <= len(v);
                // The following works
                assert i == len(v);
                assert is_set(v);
            };
            (i > 0)
        })
        {
            i = i - 1;
            _  = Vector::swap_remove(v, i);
        }
    }

    spec remove_everything {
        // TODO(refactoring): reactivate once loop invariants are implemented.
        pragma verify = false;
        requires is_set(v);
        aborts_if false;
    }

    spec module {
        fun is_set(v: vector<u64>): bool {
            forall ii: u64, jj: u64 where
                0 <= ii && ii < len(v) &&
                0 <= jj && jj < len(v):
                    v[ii] == v[jj] ==> ii == jj
        }
    }
}
