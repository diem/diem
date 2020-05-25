// Test for Reflection Module in Move
//! account: alice
//! account: bob

//! sender: alice
module M {
    use 0x0::Transaction;
    use 0x0::Reflection;
    use 0x0::Signer;

    struct T{
    }

    public fun init(account: &signer){
        let (module_address, _module_name, _struct_name) = Reflection::name_of<T>();
        // only module publisher can init this module.
        Transaction::assert(module_address == Signer::address_of(account), 8000);
    }

    public fun coin_name<CoinType>():vector<u8>{
        let (_module_address, _module_name, struct_name) = Reflection::name_of<CoinType>();
        struct_name
    }

}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::M;

fun main(account: &signer) {
    M::init(account);
}
}

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use {{alice}}::M;

fun main(account: &signer) {
    M::init(account);
}
}

// check: ABORTED
// check: 8000

//! new-transaction
//! sender: bob
script {
use {{alice}}::M;
use 0x0::Reflection;
use 0x0::Transaction;

fun main() {

    let (address, module_name, struct_name) = Reflection::name_of<M::T>();
    Transaction::assert(address == {{alice}}, 8001);
    Transaction::assert(module_name == b"M", 8002);
    Transaction::assert(struct_name == b"T", 8003);
}
}

// check: EXECUTED


//! new-transaction
//! sender: bob
script {
use {{alice}}::M;
use 0x0::Transaction;
use 0x0::LBR;
use 0x0::Coin1;

fun main() {

    let lbr_name = M::coin_name<LBR::LBR>();
    Transaction::assert(lbr_name == b"LBR", 8004);
    let coin1_name = M::coin_name<Coin1::Coin1>();
    Transaction::assert(coin1_name == b"Coin1", 8005);

}
}

// check: EXECUTED