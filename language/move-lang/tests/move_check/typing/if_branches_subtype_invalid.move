module M {
    t0(cond: bool, u: &u64, u_mut: &mut u64) {
        let _: &mut u64 = if (cond) u else u_mut;
        let _: &mut u64 = if (cond) u_mut else u;
        let _: &mut u64 = if (cond) u else u;
    }

    t1(cond: bool, u: &u64, u_mut: &mut u64, b: &bool, b_mut: &mut bool) {
        let _: &u64 = if (cond) u else b;
        let _: &u64 = if (cond) b else u;

        let _: &u64 = if (cond) u_mut else b;
        let _: &u64 = if (cond) b else u_mut;

        let _: &u64 = if (cond) u else b_mut;
        let _: &u64 = if (cond) b_mut else u;


        let _: &mut u64 = if (cond) u_mut else b_mut;
        let _: &mut u64 = if (cond) b_mut else u_mut;

    }

    t2(cond: bool, u: &u64, u_mut: &mut u64) {
        let (_, _): (&mut u64, &mut u64) = if (cond) (u, u) else (u_mut, u_mut);
        let (_, _): (&mut u64, &mut u64) = if (cond) (u_mut, u) else (u, u_mut);
        let (_, _): (&mut u64, &mut u64) = if (cond) (u, u_mut) else (u_mut, u);
    }
}
