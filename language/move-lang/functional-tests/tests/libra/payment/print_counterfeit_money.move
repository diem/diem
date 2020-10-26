//! account: default, 100000

module M {
    use 0x1::Coin1::Coin1;
    use 0x1::Libra;
    use 0x1::Signer;

    resource struct R<T: resource> { x: T }
    resource struct FakeCoin { value: u64 }

    fun fetch<T: resource>(account: &signer): T acquires R {
        let R { x } = move_from<R<T>>(Signer::address_of(account));
        x
    }

    fun store<T: resource>(account: &signer, x: T) {
        move_to(account, R { x });
    }

    fun transmute<T1: resource, T2: resource>(account: &signer, x: T1): T2 acquires R {
        store(account, x);
        fetch(account)
    }

    public fun become_rich(account: &signer) acquires R {
        let fake = FakeCoin { value: 400000 };
        let real = transmute(account, fake);
        Libra::destroy_zero<Coin1>(real);
    }
}

//! new-transaction
script {
use {{default}}::M;
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::Signer;

fun main(account: &signer) {
    M::become_rich(account);
    assert(LibraAccount::balance<Coin1>(Signer::address_of(account)) == 500000, 42);
}
}
// check: MISSING_DATA
