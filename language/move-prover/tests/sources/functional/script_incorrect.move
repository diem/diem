// flag: --dependency=tests/sources/functional/script_provider.move
script {
use 0x1::ScriptProvider;

// TODO: This file inherits an error from ScriptProvider.

fun main<Token>(account: &signer) {
    ScriptProvider::register<Token>(account);
}

spec fun main {
    aborts_if false;
}
}
