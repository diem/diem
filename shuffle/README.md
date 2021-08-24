## Code under this subtree is experimental.

# Custom Diem Node Starter Kit

This folder contains what you need to start a Diem node locally with a custom genesis module set, along with
some sample Move code and a sample application sending transactions to the local "blockchain" you
just started.

## Folder Structure

```
.
├── genesis                   # CLI for building and starting a validator node
├── move/src                  # Move source code
│   ├── diem                  # Diem Framework code
│   ├── SampleModule.move     # A sample Move module
├── sample-app                # A sample app talking to the blockchain
├── transaction-builder       # Auto-generated transaction builders in Rust
```
## Step 0: Install Dependencies

- Install Diem dependencies including Rust, Clang, etc, by running the following script in `diem` root directory:
```
./scripts/dev_setup.sh
```
- Install `df-cli` inside `diem` repo. We use this tool in step 2 to compile Move code:
```
cargo install --git https://github.com/diem/diem df-cli --branch main
```

## Step 1: Write Move Code

Write some Move modules you wish to include in your custom genesis and put them under `./move/src/modules`,
such as the `SampleModule` we have included. This sample module contains a simple script function `mint_coin` that
publishes a resource called `Coin` as well as a unit test `test_mint_coin` that tests this function.

Inside `move` folder, run `sh compile.sh` to compile the Move code. No output means everything is good.

Running `cargo test` inside `./move` runs the unit test with output like this:
```
running 1 test
Running Move unit tests
[ PASS    ] 0x1::SampleModule::test_mint_coin
Test result: OK. Total tests: 1; passed: 1; failed: 0
```

## Step 2: Start a Validator Node

Inside `./genesis`, run the following command:
```bash
RUST_LOG=WARN cargo run -- --node-config-dir <directory_name_that_doesnt_exist>
```

Add `--open-publishing` to the command above if you would like your validator to allow module publishing.

The output of this command should look like this:
```
Building Move code ... (took 0.545s)
Generating script ABIs ... (took 3.687s)
Generating Rust script builders ... (took 0.383s)
Creating genesis with 39 modules
Running a Diem node with custom modules ...

```

This code completes the following tasks in order:
1. Compiles Move code in `./move/src`, auto-generates ABIs and transaction builders for script functions.
2. Builds a validator config that includes all the binaries of Move code from previous step in the genesis writeset,
   and save this config in the directory specified on the command line (what comes after `--node-config-dir`)
3. Starts a Diem node with the validator config built in previous step on `http://0.0.0.0:8080`.

### Troubleshooting

If you see this error:
```
Building Move code ... thread 'main' panicked at 'Automatically building Move code failed.
Need to manually resolve the issue using the CLI', shuffle/genesis/src/lib.rs:75:13
```
Go back to `./move` folder and recompile by running `compile.sh` again.

## Step 3: Send Transactions to the Validator

Make sure your node from step 2 is running before starting this step.

Inside `./sample-app`, run the following command:
```bash
cargo run -- --node-config-dir <config-dir-from-previous-step> --account-key-path new_account.key
```

The output of this command should look like this:
```
Connecting to http://0.0.0.0:8080...
Create a new account...Success!
Mint a coin to the new account...Success!
Mint another coin to the new account (this should fail)...
Error: TransactionExecutionFailed(
.... some detailed error message
```

This sample app submits three transactions to the validator node:
(1) creates a new account with the specified key at `account-key-path`, (2) publishes a `SampleModule::Coin` resource under the address of the
newly created account, (3) publish another `SampleModule::Coin` resource to the address of the same account.

The first two transactions should succeed and the last one should fail because you can only publish
one resource of each type under an address.
