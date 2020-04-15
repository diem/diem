use 0x0::Association;
fun main<Privilege>(addr: address) {
    Association::grant_privilege<Privilege>(addr)
}
