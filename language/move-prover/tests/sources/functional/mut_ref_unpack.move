address 0x1 {
module TestMutRefs {

    spec module {
        pragma verify = true;
    }

    resource struct R {
        value: u64
    }

    spec struct R {
        global sum: num;
        invariant pack sum = sum + value;
        invariant unpack sum = sum - value;
    }

    public fun unpack(r: &mut R): u64 {
        let R{value} = r;
        let result = *value;
         // We need to reset the value because we are only borrowing the `r: &mut R`, not consuming. Otherwise
         // we get an error regards the `sum` variable.
        *value = 0;
        result
    }
    spec fun unpack {
        ensures r.value == 0;
        ensures sum == old(sum) - old(r.value);
    }

    public fun unpack_incorrect(r: &mut R): u64 {
         let R{value} = r;
         let result = *value;
         // Here we get the error described above.
         // *value = 0;
         result
     }
     spec fun unpack_incorrect {
         ensures sum == old(sum) - old(r.value);
     }

     public fun unpack_caller(r: &mut R): u64 {
        unpack(r)
     }
     spec fun unpack_caller {
        ensures r.value == 0;
     }
}
}
