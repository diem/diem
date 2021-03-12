module 0x8675309::M {
    struct Box<T> { x: T }

    fun unbox<T>(b: Box<T>): T {
        let Box { x } = b;
        x
    }

    fun f<T>(n: u64, x: T): T {
        if (n == 0) return x;
        unbox<T>(f<Box<T>>(n - 1, Box<T> { x }))
    }
}

// Function f calls an instance of itself instantiated with a boxed type, but it does
// terminate properly if we allow new types to be created on the fly.
//
// As reference, the equivalent Haskell code compiles & terminates:
//
//   data Boxed a = Box a
//
//   unbox (Box a) = a
//
//   f :: Integer -> a -> a
//   f 0 x = x
//   f n x = unbox (f (n-1) (Box x))
//
//    main = do
//      let x = 42
//      putStrLn (show (f 5 x))
//
// Without the annotation f :: Integer -> a -> a GHC complains about not being able to
// construct the infinite type a ~ Boxed a.
//
// We are taking a conservative approach and forbids such construction in move.

// check: LOOP_IN_INSTANTIATION_GRAPH
