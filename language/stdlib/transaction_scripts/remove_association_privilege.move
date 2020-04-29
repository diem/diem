script {
use 0x0::Association;
fun main<Privilege>(addr: address) {
    Association::remove_privilege<Privilege>(addr)
}
}
