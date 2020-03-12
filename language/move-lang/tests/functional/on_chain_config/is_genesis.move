use 0x0::LibraTimestamp;
use 0x0::Transaction;

fun main() {
    Transaction::assert(!LibraTimestamp::is_genesis(), 10)
}
