//! account: bob, 10000LBR
//! account: alice, 10000Coin1
//! account: gary, 10000Coin2
//! account: vivian, 0, 0, validator

//! new-transaction
script {
    use 0x0::LBR;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Transaction;
    use 0x0::LibraAccount;
    fun main() {
        Transaction::assert(LibraAccount::balance<LBR::T>(0xFEE) == 0, 0);
        Transaction::assert(LibraAccount::balance<Coin1::T>(0xFEE) == 0, 1);
        Transaction::assert(LibraAccount::balance<Coin2::T>(0xFEE) == 0, 2);

        Transaction::assert(LibraAccount::balance<LBR::T>({{vivian}}) == 0, 3);
        Transaction::assert(LibraAccount::balance<Coin1::T>({{vivian}}) == 0, 4);
        Transaction::assert(LibraAccount::balance<Coin2::T>({{vivian}}) == 0, 5);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 1
script {
fun main() { }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: Coin1
script {
fun main() { }
}
// check: EXECUTED

//! new-transaction
//! sender: gary
//! gas-price: 1
//! gas-currency: Coin2
script {
fun main() { }
}
//! check: EXECUTED

//! new-transaction
script {
    use 0x0::LBR;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Transaction;
    use 0x0::LibraAccount;
    fun main() {
        Transaction::assert(LibraAccount::balance<LBR::T>(0xFEE) > 0, 6);
        Transaction::assert(LibraAccount::balance<Coin1::T>(0xFEE) > 0, 7);
        Transaction::assert(LibraAccount::balance<Coin2::T>(0xFEE) > 0, 8);

        Transaction::assert(LibraAccount::balance<LBR::T>({{vivian}}) == 0, 9);
        Transaction::assert(LibraAccount::balance<Coin1::T>({{vivian}}) == 0, 10);
        Transaction::assert(LibraAccount::balance<Coin2::T>({{vivian}}) == 0, 11);
    }
}
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
script {
    use 0x0::LBR;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Transaction;
    use 0x0::LibraAccount;
    fun main() {
        Transaction::assert(LibraAccount::balance<LBR::T>({{vivian}}) > 0, 12);
        Transaction::assert(LibraAccount::balance<Coin1::T>({{vivian}}) > 0, 13);
        Transaction::assert(LibraAccount::balance<Coin2::T>({{vivian}}) > 0, 14);
    }
}
// check: EXECUTED
