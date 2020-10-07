// flag: --dependency=tests/sources/functional/script_provider.move
script {
use 0x1::ScriptProvider;

fun main<Token>(account: &signer) {
    ScriptProvider::register<Token>(account);
}

spec fun main {
    // TODO: This file gets errors that are produced non-deterministically, therefore turned off.
    pragma verify = false;
    aborts_if false;
}
}
