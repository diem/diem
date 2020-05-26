script {
use 0x0::LibraAccount;
fun main<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
}
