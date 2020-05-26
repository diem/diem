script {
use 0x0::Transaction;

fun main() {
    Transaction::assert(b"" == x"", 0);
    Transaction::assert(b"Libra" == x"4c69627261", 1);
    Transaction::assert(b"\x4c\x69\x62\x72\x61" == x"4c69627261", 2);
    Transaction::assert(b"\"Hello\tlibra.\"\n \r \\Null=\0" == x"2248656c6c6f096c696272612e220a200d205c4e756c6c3d00", 3);
}
}
