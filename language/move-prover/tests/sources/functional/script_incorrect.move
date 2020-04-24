// flag: --dep=tests/sources/functional/script_provider.move

use 0x0::ScriptProvider;

fun main<Token>() {
    ScriptProvider::register<Token>();
}

spec fun main {
    aborts_if false;
}
