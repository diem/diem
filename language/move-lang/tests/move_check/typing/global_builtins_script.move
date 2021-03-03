address 0x42 {
module M {
    resource struct R {}
    public fun new(): R {
        R {}
    }
}
}


script {
use 0x42::M;

fun test<Token>(account: &signer) {
    let r = M::new();
    borrow_global<M::R>(0x1);
    move_to(account, r);
}
}
