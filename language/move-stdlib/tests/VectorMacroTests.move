#[test_only]
module Std::VectorMacroTests {
    use Std::Vector as V;

    #[test]
    fun foreach_mut() {
        let v = vector[0, 1, 2];
        V::foreach_mut!(&mut v, |x| *x = *x + 1);
        assert!(*V::borrow(&v, 0) == 1, 0);
        assert!(*V::borrow(&v, 1) == 2, 0);
        assert!(*V::borrow(&v, 2) == 3, 0);
    }

    #[test]
    fun foreach() {
        let v = vector[1, 2, 3];
        let sum = 0;
        V::foreach!(&v, |x| sum = sum + *x);
        assert!(sum == 6, 1);
    }
}