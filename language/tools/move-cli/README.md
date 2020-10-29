# Move CLI

This is a tool for experimenting with Move without validators, a blockchain, or transactions. Persistent data is stored on-disk in a directory structure that mimics the Move memory model

## Installation
```
λ cargo install --path libra/language/tools/move-cli
```
or
```
λ cargo install --git https://github.com/libra/libra move-cli
```

## Compiling and running scripts

Here is a simple script that prints its `signer`:

```rust
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

## Adding new modules via `src`

By default, the CLI compiles and includes all files from the Move standard library and Libra Framework. New modules can be added to the `src` directory (or a directory of your choosing specified via `--source-dir`. The `move run` command will compile and publish each module source file in this directory before running the given script.

Try saving this code in `src/modules/Test.move`:

```
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

The CLI has successfully compiled the module, but by default it chooses not to publish the module bytecode under `storage`. Re-running the command with `--commit` (`-c` for short) will produce

```
λ move compile -c
Compiling 1 user module(s)
Committed changes.
```

If we take a look under `storage`, we will now see the published bytecodes for our test module:

```
λ ls storage/0x00000000000000000000000000000002/modules
Test
```

We can also inspect the compiled bytecode using `move view`:

```
λ move view storage/0x00000000000000000000000000000002/modules/Test
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
λ move view storage/0x0000000000000000000000000000000f/resources/0x00000000000000000000000000000002\:\:Test\:\:Resource
resource 00000000::Test::Resource {
    i: 10
}
```


## Using the CLI with Libra modules and genesis state

Take a look at `tests/testsuite/liba_smoke/args.txt`. This test uses the CLI to run a fairly realistic Libra genesis setup and a few basic transactions. Running

```
λ NO_MOVE_CLEAN=1 cargo xtest libra_smoke
```

will execute each command in this file and leave you the resulting `storage` to experiment with.
