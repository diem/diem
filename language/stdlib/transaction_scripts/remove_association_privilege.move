script {
use 0x1::Association;
fun main<Privilege>(account: &signer, addr: address) {
    Association::remove_privilege<Privilege>(account, addr)
}
}
