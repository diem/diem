// flag: --dependency=tests/sources/functional/script_provider.move
script {
use 0x1::ScriptProvider;

fun main<Token>() {
    ScriptProvider::register<Token>();
}

spec fun main {
    include ScriptProvider::RegisterConditions<Token>;
}
}
