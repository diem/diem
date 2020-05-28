script {
use 0x0::LibraAccount;
    // Script for un-freezing account by authorized initiator
    fun main(to_unfreeze_account: address) {
        LibraAccount::unfreeze_account(to_unfreeze_account);
    }
}
