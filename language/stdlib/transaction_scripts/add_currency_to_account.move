script {
use 0x1::LibraAccount;
fun main<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
}
