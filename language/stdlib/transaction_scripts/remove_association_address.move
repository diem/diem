script {
use 0x0::Association;
fun main(addr: address) {
    Association::remove_privilege<Association::T>(addr)
}
}
