script {
use 0x0::Association;
fun main<Privilege>(account: &signer, addr: address) {
    Association::remove_privilege<Privilege>(account, addr)
}
}
