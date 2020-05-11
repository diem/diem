script {
use 0x0::LibraAccount;
fun main<Currency>() {
    LibraAccount::add_currency<Currency>();
}
}
