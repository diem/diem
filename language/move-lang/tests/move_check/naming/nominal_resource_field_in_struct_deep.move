module M {
    resource struct R {}
    struct RCup<T: resource> { x: T }
    struct Cup<T> { x: T }
    struct S {
        f: RCup<R>,
        g: Cup<R>,
    }
    struct S2<T: resource> {
        f1: RCup<RCup<T>>,
        f2: Cup<Cup<T>>,
        g1: RCup<RCup<RCup<R>>>,
        g2: Cup<Cup<Cup<R>>>,
    }
}
