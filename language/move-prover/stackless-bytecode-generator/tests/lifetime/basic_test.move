module TestLifetime {

    struct R {
        x: u64
    }


    fun lifetime_R() : R {
        let r = R {x: 3};
        let r_ref = &mut r;
        let x_ref = &mut r_ref.x;
        *x_ref = 0; // r_ref goes out of scope here

        r_ref = &mut r;
        x_ref = &mut r_ref.x;
        *x_ref = 2;

        r
    }

    fun lifetime_R_2() : R {
        let r = R {x: 4};
        let r_ref = &mut r;
        let x_ref = &mut r_ref.x;
        *x_ref = 0;
        *x_ref = 2; // r_ref goes out of scope here

        r_ref = &mut r;
        x_ref = &mut r_ref.x;
        *x_ref = 3;

        r
    }

    struct T {
        x: u64
    }

    struct S {
        y: u64
    }

    fun lifetime_(cond: bool): (T, S) {
      let a = T {x: 3};
      let b = S {y: 4};
      let a_ref = &mut a;
      let b_ref = &mut b;
      let x_ref = if (cond) { &mut a_ref.x } else { &mut b_ref.y };

      if (cond) {
          *x_ref = 2;
      } else {
          *x_ref = 0;
      };

      (a, b)
    }
}
