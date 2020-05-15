// Tests of quantification over addresses.
module AddressQuant {
    use 0x0::Transaction;

    resource struct R {
        x: u64
    }

    spec module {
        pragma verify = true;
    }

    spec module {
       // helper functions
       define atMostOne(): bool {
            all(addresses(),
                |a| all(addresses(),
                        |b| exists<R>(a) && exists<R>(b) ==> a == b))

        }
        define atLeastOne(): bool {
            any(addresses(),
                |a| exists<R>(a))
        }
    }

    public fun initialize(special_addr: address) {
        Transaction::assert(Transaction::sender() == special_addr, 0);
        move_to_sender<R>(R{x:1});
    }
    spec fun initialize {
        requires all(addresses(), |a| !exists<R>(a)); // forall a: address :: !exists<R>(a)
        ensures all(addresses(), |a| exists<R>(a) ==> a == special_addr);
        ensures atMostOne();
        ensures atLeastOne();
    }

    // This is tricky. If it doesn't abort on the borrow_global,
    // or on overflow, it will successfully increment a.x
    public fun f(addr: address) acquires R {
        let x_ref = borrow_global_mut<R>(addr);
        x_ref.x = x_ref.x + 1;
    }
    spec fun f {
        ensures global<R>(addr).x == old(global<R>(addr).x) + 1;
    }

    // sender() might be different from special_addr,
    // so this should violate the invariant.
    public fun multiple_copy_incorrect() {
        move_to_sender<R>(R{x:1});
    }

    // This asserts that there is at must one address with an R.
    // Literally, if addresses a and b have an R, then a and b are the same.
    spec schema ExactlyOne{
         invariant atMostOne();
         invariant atLeastOne();
    }

    spec module {
        apply ExactlyOne to * except initialize;
    }

}
