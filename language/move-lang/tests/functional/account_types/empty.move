//! new-transaction
fun main() {
    let addr: address = 0xDEADBEEF;
    0x0::LibraAccount::create_account<0x0::LBR::T>(addr, 0x0::LCS::to_bytes<address>(&addr));
}
