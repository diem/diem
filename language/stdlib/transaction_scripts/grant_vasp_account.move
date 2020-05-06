script {
use 0x0::Association;
use 0x0::VASP;
use 0x0::Transaction;
fun main(root_vasp_addr: address) {
    Association::apply_for_privilege<VASP::CreationPrivilege>();
    Association::grant_privilege<VASP::CreationPrivilege>(Transaction::sender());
    VASP::grant_vasp(root_vasp_addr);
}
}
