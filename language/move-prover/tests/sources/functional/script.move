// flag: --dep=tests/sources/functional/script_provider.move
script {
use 0x0::ScriptProvider;

fun main<Token>() {
    ScriptProvider::register<Token>();
}

spec fun main {
    include ScriptProvider::RegisterConditions<Token>;
}
}
