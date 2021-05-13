script {
// Usage: bisect-transaction <Path_to_this_file> <any_account> <begin_version> <end_version>
// Find the first version where the time exceeds X.
use DiemFramework::DiemTimestamp;
fun main(_dr_account: signer, _sender: signer) {
   let time_to_query = 1598390547040813;
   assert(DiemTimestamp::now_microseconds() < time_to_query, 1);
   return
}
}
