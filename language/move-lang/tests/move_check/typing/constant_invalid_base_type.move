address 0x42 {
module M {
    struct S {}
    struct R {}

    const C1: signer = abort 0;
    const C2: S = S{};
    const C3: R = R{};
    const C4: vector<S> = abort 0;
    const C5: vector<R> = abort 0;
    const C6: vector<vector<S>> = abort 0;
    const C7: vector<vector<R>> = abort 0;
}
}
