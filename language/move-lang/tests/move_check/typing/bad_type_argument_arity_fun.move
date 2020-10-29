address 0x42 {
module M {

    struct S<T> { f: T }

    fun foo<T>(x: T): T {
        x
    }

    fun bar() {
        let x = foo<>(0);
        let b = foo<bool, u64>(false);
        b && false;
        let r = foo<&mut u64, bool>(&mut 0);
        *r = 1;

    }

}
}
