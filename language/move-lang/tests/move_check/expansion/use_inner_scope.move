address 0x2 {
module M {
    struct S1 has drop { b: bool }
    struct S2 has drop { u: u64 }
    fun check(): bool {
        false
    }
    fun num(): u64 {
        0
    }

    fun t() {
        use 0x2::M::{check as num, num as check, S1 as S2, S2 as S1};
        num() && false;
        check() + 1;
        S1 { u: 0 };
        S2 { b: false };
        {
            use 0x2::M::{check, num, S1, S2};
            check() && false;
            num() + 1;
            S2 { u: 0 };
            S1 { b: false };
        }
    }
}
}
