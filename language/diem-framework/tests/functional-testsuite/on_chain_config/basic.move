//! new-transaction
script {
    use 0x1::DiemConfig::{Self};
    fun main(account: signer) {
    let account = &account;
        DiemConfig::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
script {
    use 0x1::DiemConfig;
    fun main() {
        let _x = DiemConfig::get<u64>();
    }
}
// check: "Keep(ABORTED { code: 261,"

//! new-transaction
script {
    use 0x1::DiemConfig;
    fun main(account: signer) {
    let account = &account;
        DiemConfig::set(account, 0);
    }
}
// check: "Keep(ABORTED { code: 516,"

//! new-transaction
script {
    use 0x1::DiemConfig::{Self};
    fun main(account: signer) {
    let account = &account;
        DiemConfig::publish_new_config(account, 0);
    }
}
// check: "Keep(ABORTED { code: 2,"
