script {
use 0x1::LibraAccount;
fun add_currency_to_account<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
}
