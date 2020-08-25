script {
// Usage: bisect-transaction <Path_to_this_file> <Account_to_query> <begin_version> <end_version>
// Find the first version where the account is created.
use 0x1::LibraAccount;
use 0x1::Signer;
fun main(_lr_account: &signer, sender: &signer) {
    let addr = Signer::address_of(sender);
    if(LibraAccount::exists_at(addr)) {
        abort 1
    };
    return
}
}
