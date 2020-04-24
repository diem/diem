use 0x0::Association;
fun main<Privilege>() {
    Association::apply_for_privilege<Privilege>();
}
