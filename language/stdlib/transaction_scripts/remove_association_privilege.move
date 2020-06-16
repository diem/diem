script {
use 0x1::Association;
fun remove_association_privilege<Privilege>(account: &signer, addr: address) {
    Association::remove_privilege<Privilege>(account, addr)
}
}
