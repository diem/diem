module M {
    fun t0(cond: bool, u: &u64, u_mut: &mut u64) {
        let _: &u64 = if (cond) u else u_mut;
        let _: &u64 = if (cond) u_mut else u;
        let _: &u64 = if (cond) u_mut else u_mut;
    }

    fun t1(cond: bool, u: &u64, u_mut: &mut u64) {
        let (_, _): (&u64, &u64) = if (cond) (u, u) else (u_mut, u_mut);
        let (_, _): (&u64, &u64) = if (cond) (u_mut, u) else (u, u_mut);
        let (_, _): (&u64, &u64) = if (cond) (u, u_mut) else (u_mut, u);
    }
}
