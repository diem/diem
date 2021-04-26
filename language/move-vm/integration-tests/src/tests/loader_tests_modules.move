address 0x2 {
    module A {
        struct S has copy, drop {
            f1: 0x2::B::S,
            f2: 0x2::C::S,
        }

        public fun new(f1: 0x2::B::S, f2: 0x2::C::S): S {
            Self::S { f1, f2 }
        }

        public fun destroy(v: S): (0x2::B::S, 0x2::C::S) {
            let S { f1: v1, f2: v2 } = v;
            (v1, v2)
        }

        public fun entry_a() {
            let i = 0;
            while (i < 10) {
                let b = 0x2::B::new(20, 100);
                let c = 0x2::C::new(@0x42, true);
                let another_b = 0x2::B::b_and_c(&b, c);
                let (_, _) = 0x2::B::destroy(another_b);
                let another_c = 0x2::C::new(@0x42, false);
                0x2::C::destroy(another_c);
                i = i + 1;
            }
        }

        public fun get_field_1(s: &S): 0x2::B::S {
            *&s.f1
        }
    }

    module B {
        struct S has copy, drop {
            f1: u64,
            f2: u128,
        }

        public fun new(v1: u64, v2: u128): S { S { f1: v1, f2: v2 } }

        public fun destroy(v: S): (u64, u128) {
            let S { f1: val1, f2: val2 } = v;
            (val1, val2)
        }

        public fun b_and_c(b: &S, c: 0x2::C::S): S {
            let _ = 0x2::C::destroy(c);
            let another_b = S {
                f1: 0,
                f2: b.f2,
            };
            another_b
        }
    }

    module C {
        struct S has copy, drop {
            f1: address,
            f2: bool,
        }

        public fun new(v1: address, v2: bool): S {
            Self::S {
                f1: v1,
                f2: v2,
            }
        }

        public fun destroy(v: S): address {
            let S { f1: v1, f2: _ } = v;
            v1
        }

        public fun just_c() {
            let i = 0;
            while (i < 10) {
                let c = new(@0x0, false);
                let S { f1: _, f2: _ } = c;
                i = i + 1;
            }
        }
    }


    module D {
        struct S has copy, drop {
            f1: 0x2::B::S,
        }

        public fun new(): 0x2::D::S {
            Self::S {
                f1: 0x2::B::new(20, 100),
            }
        }

        public fun entry_d() {
            let i = 0;
            while (i < 10) {
                let b = 0x2::B::new(20, 100);
                let c = 0x2::C::new(@0x45, false);
                let another_b = 0x2::B::b_and_c(&b, c);
                let (_, _) = 0x2::B::destroy(another_b);
                let another_c = 0x2::C::new(@0x46, true);
                0x2::C::destroy(another_c);
                i = i + 1;
            }
        }
    }

    module E {
        struct S {
            f1: u64,
        }

        public fun new(): 0x2::E::S { Self::S { f1: 20 } }

        public fun entry_e() {
            let i = 0;
            while (i < 10) {
                let b = 0x2::B::new(20, 100);
                let c = 0x2::C::new(@0x100, false);
                let another_b = 0x2::B::b_and_c(&b, c);
                let (_, _) = 0x2::B::destroy(another_b);
                let another_c = 0x2::C::new(@0x101, true);
                0x2::C::destroy(another_c);
                i = i + 1;
            };
        }
    }

    module F {
        struct S {
            f1: u64,
        }

        public fun new(): 0x2::F::S { Self::S { f1: 20 } }

        public fun entry_f() {
            0x2::A::entry_a();
        }
    }
}
