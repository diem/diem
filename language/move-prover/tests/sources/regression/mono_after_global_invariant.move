address 0x2 {
module Base {
    struct B has key {}

    public fun BASE_ADDR(): address {
        @0x2
    }

    public fun put_b(s: &signer) {
        move_to(s, B {});
        // the global invariants in 0x2::Test is instrumented here
    }

    spec module {
        fun has_b(): bool {
            exists<B>(BASE_ADDR())
        }
    }
}

module Test {
    use 0x2::Base;

    struct R<T: store> has key {
         f: T,
    }

    public fun put_r<T: store>(s: &signer, v: T) {
        Base::put_b(s);
        move_to(s, R { f: v });
    }

    spec module {
        fun has_r<T>(): bool {
            exists<R<T>>(Base::BASE_ADDR())
        }
    }

    spec module {
        invariant update
            Base::has_b() ==>
                (forall t: type where has_r<t>(): old(has_r<t>()));

        // TODO: the above invariant should not verify, here is a counterexample:
        // suppose we starts with an empty state,
        // put_r(@0x2, false) will violate the invariant, because
        // - Base::has_b() is true,
        // - has_r<bool>() is true, but
        // - old(has_r<bool>()) is false

        /*
            note the difference on the global invariant before and after mono analysis

            ==== mono-analysis result ====
            struct Test::R = {
              <#0>
            }

            [variant verification - after "mono_analysis"]
            public fun Base::put_b($t0|s: signer) {
                 var $t1: bool
                 var $t2: Base::B
                 var $t3: num
              0: assume WellFormed($t0)
              1: assume forall $rsc: ResourceDomain<Base::B>(): WellFormed($rsc)
              2: trace_local[s]($t0)
              3: $t1 := false
              4: $t2 := pack Base::B($t1)
              5: move_to<Base::B>($t2, $t0) on_abort goto 9 with $t3
                 # global invariant at mono_after_global_invariant.move:40:9+114
                 # VC: global memory invariant does not hold at mono_after_global_invariant.move:40:9+114
          >>> 6: assert Implies(Base::has_b(), true)
              7: label L1
              8: return ()
              9: label L2
             10: abort($t3)
            }

             [variant verification - after "global_invariant_instrumenter_v2"]
             public fun Base::put_b($t0|s: signer) {
                  var $t1: bool
                  var $t2: Base::B
                  var $t3: num
               0: assume WellFormed($t0)
               1: assume forall $rsc: ResourceDomain<Base::B>(): WellFormed($rsc)
               2: trace_local[s]($t0)
               3: $t1 := false
               4: $t2 := pack Base::B($t1)
                  # state save for global update invariants
               5: @1 := save_mem(Test::R<t>)
               6: move_to<Base::B>($t2, $t0) on_abort goto 10 with $t3
                  # global invariant at mono_after_global_invariant.move:40:9+114
                  # VC: global memory invariant does not hold at mono_after_global_invariant.move:40:9+114
           >>> 7: assert Implies(Base::has_b(), forall t: TypeDomain<type>() where Test::has_r<t>(): Test::has_r[@1]<t>())
               8: label L1
               9: return ()
              10: label L2
              11: abort($t3)
             }
        */

    }
}
}
