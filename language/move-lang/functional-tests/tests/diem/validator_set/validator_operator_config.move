//! new-transaction
script {
use DiemFramework::ValidatorOperatorConfig;
fun main() {
    ValidatorOperatorConfig::get_human_name(@{{default}});
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: blessed
script {
use DiemFramework::ValidatorOperatorConfig;
fun main(account: signer) {
    let account = &account;
    ValidatorOperatorConfig::publish(account, account, x"");
}
}
// check: "Keep(ABORTED { code: 2,"
