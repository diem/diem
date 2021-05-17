#[attr]
address 0x42 {
#[attr]
module M {
    #[attr]
    use 0x42::N;

    #[attr]
    struct S {}

    #[attr]
    const C: u64 = 0;

    #[attr]
    public fun foo() { N::bar() }

    #[attr]
    spec foo {}
}
}

#[attr]
module 0x42::N {
    #[attr]
    friend 0x42::M;

    #[attr]
    public fun bar() {}
}

#[attr]
script {
    #[attr]
    use 0x42::M;

    #[attr]
    const C: u64 = 0;

    #[attr]
    fun main() {
        M::foo();
    }

    #[attr]
    spec main { }
}
