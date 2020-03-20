use 0x0::ScriptWhitelist;

fun main(args: vector<u8>) {
    ScriptWhitelist::set(args)
}
