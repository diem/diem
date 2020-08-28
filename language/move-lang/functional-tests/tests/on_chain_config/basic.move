//! new-transaction
script {
    use 0x1::LibraConfig::{Self};
    fun main(account: &signer) {
        LibraConfig::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
script {
    use 0x1::LibraConfig;
    fun main() {
        let _x = LibraConfig::get<u64>();
    }
}
// check: "Keep(ABORTED { code: 261,"

//! new-transaction
script {
    use 0x1::LibraConfig;
    fun main(account: &signer) {
        LibraConfig::set(account, 0);
    }
}
// check: "Keep(ABORTED { code: 516,"

//! new-transaction
script {
    use 0x1::LibraConfig::{Self};
    fun main(account: &signer) {
        LibraConfig::publish_new_config(account, 0);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
module Holder {
    resource struct Holder<T> { x: T }
    public fun hold<T>(account: &signer, x: T)  {
        move_to(account, Holder<T> { x })
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraConfig::{Self};
    use {{default}}::Holder;
    use 0x1::LibraTimestamp;
    fun main(account: &signer) {
        LibraTimestamp::reset_time_has_started_for_test();
        Holder::hold(account, LibraConfig::publish_new_config_and_get_capability(account, 0));
        LibraConfig::set(account, 1);
    }
}
// check: "Keep(ABORTED { code: 516,"

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraConfig::{Self};
    use {{default}}::Holder;
    use 0x1::LibraTimestamp;
    fun main(account: &signer) {
        LibraTimestamp::reset_time_has_started_for_test();
        Holder::hold(account, LibraConfig::publish_new_config_and_get_capability(account, 0));
        LibraConfig::set(account, 1);
    }
}
// check: "Keep(ABORTED { code: 2,"
