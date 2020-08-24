# Move CLI

This is a tool for experimenting with Move without validators, a blockchain, or transactions. Persistent data is stored on-disk in a directory structure that mimics the Move memory model

## Installation
```
cargo build --release
export LIBRA_HOME=<path_to_your_libra_dir>
alias move="$LIBRA_HOME/target/release/move-cli $@"
```

## Compiling and running scripts

Here is a simple script that prints its `signer`:

```rust
// TODO: perhaps start with a gentler intro script before going to signers
script {
use 0x1::Debug;
fun main(account: &signer) {
    Debug::print(account)
}
}
```

Place this in a file named `script.move` and try
```
λ move run script.move --signers 0xf
[debug] (&) { 0000000000000000000000000000000f }
```

The `--signers 0xf` part indicates which account address(es) have signed off on the script. Omitting `--signers` or passing multiple signers to this single-`signer` script will trigger a type error.

TODO: show script compilation error

TODO: showcase basic arithmetic, control-flow constructs, `Signer` module, `Vector` and `Option` modules.

## Adding new modules via `move_lib`

By default, the CLI compiles and includes the `Compare`, `Debug`, `Errors`, `Option`, `Signer`, and `Vector` modules from the Move standard library. New modules can be added to the `move_lib` directory. The `move run` command will compile and publish each module source file in `move_lib` before running the given script.

Try saving this code in `move_lib/Test.move`:

```
// TODO: more carefully chosen example + comments to show what's going on
module Test {
    use 0x1::Signer;

    resource struct Resource { i: u64 }

    public fun publish(account: &signer) {
        move_to(account, Resource { i: 10 })
    }

    public fun write(account: &signer, i: u64) acquires Resource {
        borrow_global_mut<Resource>(Signer::address_of(account)).i = i;
    }

    public fun unpublish(account: &signer) acquires Resource {
        let Resource { i: _ } = move_from(Signer::address_of(account));
  }
}
```

Now, try

```
λ move compile
Compiling 1 user module(s)
Discarding changes; re-run with --commit if you would like to keep them.
```

The CLI has successfully compiled the module, but by default it chooses not to publish the module bytecode under `move_state`. Re-running the command with `--commit` (`-c` for short) will produce

```
λ move compile -c
Compiling 1 user module(s)
Committed changes.
```

If we take a look under `move_data`, we will now see the published bytecodes for our test module:

```
λ ls move_data/00000000000000000000000000000002/
Test
```

We can also inspect the compiled bytecode using `move view`:

```
λ move view move_data/00000000000000000000000000000002/Test
module 00000000.Test {
resource Resource {
	i: u64
}

public publish() {
	0: MoveLoc[0](Arg0: &signer)
	1: LdU64(10)
	2: Pack[0](Resource)
	3: MoveTo[0](Resource)
	4: Ret
}
public unpublish() {
	0: MoveLoc[0](Arg0: &signer)
	1: Call[0](address_of(&signer): address)
	2: MoveFrom[0](Resource)
	3: Unpack[0](Resource)
	4: Pop
	5: Ret
}
public write() {
	0: CopyLoc[1](Arg1: u64)
	1: MoveLoc[0](Arg0: &signer)
	2: Call[0](address_of(&signer): address)
	3: MutBorrowGlobal[0](Resource)
	4: MutBorrowField[0](Resource.i: u64)
	5: WriteRef
	6: Ret
}
}
```

## Updating state

Let's exercise our new module by running the following script:

```
script {
use 0x2::Test;
fun main(account: &signer) {
    Test::publish(account)
}
}
```

This script invokes the `publish` function of our `Test` module, which will publish a resource of type `Test::Resource` under the signer's account:

```
λ move run script.move --signers 0xf -c
Compiling 1 user module(s)
Changed resource(s) under 1 address(es):
  Changed 1 resource(s) under address 0000000000000000000000000000000f:
    Added type 00000000::Test::Resource: [U64(10)]
Committed changes.
```

We can also inspect this newly published resource using `move view`:

```
λ move view move_data/0000000000000000000000000000000f/00000000000000000000000000000002\:\:Test\:\:Resource
resource 00000000::Test::Resource {
    i: 10
}
```

TODO: exercise the other functions of the module

## Using the CLI with Libra modules and genesis state

From `libra/language/stdlib`, run the following script via `move --no-use-stdlib --lib modules run script.move --signers 0xA550C18 0xB1E55ED -c`:

```
script {
use 0x1::Genesis;
use 0x1::Vector;
fun main(
    lr_account: &signer,
    tc_account: &signer
) {
    let dummy_auth_key = x"0000000000000000000000000000000000000000000000000000000000000000";
    let lr_auth_key = copy dummy_auth_key;
    let tc_addr = 0xB1E55ED;
    let tc_auth_key = dummy_auth_key;
    let initial_script_allow_list = Vector::empty<vector<u8>>();
    let is_open_module = true;
    let instruction_schedule = Vector::empty<u8>();
    let native_schedule = Vector::empty<u8>();
    let chain_id = 0;

    Genesis::initialize(
        lr_account,
        tc_account,
        lr_auth_key,
        tc_addr,
        tc_auth_key,
        initial_script_allow_list,
        is_open_module,
        instruction_schedule,
        native_schedule,
        chain_id
    );
}
}
```

This will initialize the Libra genesis state and make it possible to invoke Libra Framework scripts and modules that rely on the existence of the genesis state.
