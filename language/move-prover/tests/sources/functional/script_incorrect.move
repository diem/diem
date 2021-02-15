// flag: --dependency=tests/sources/functional/script_provider.move
script {
use 0x1::ScriptProvider;


fun main<Token>(account: &signer) {
    ScriptProvider::register<Token>(account);
}

spec fun main {
    pragma verify = true;
    aborts_if false;
}
}
