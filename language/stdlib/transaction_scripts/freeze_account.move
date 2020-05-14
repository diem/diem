script {
use 0x0::LibraAccount;
    // Script for freezing account by authorized initiator
    fun main(to_freeze_account: address) {
        LibraAccount::freeze_account(to_freeze_account)
    }
}
