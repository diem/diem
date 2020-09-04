---
id: run-move-locally
title: Run Move Programs Locally
---
<blockquote class="block_note">

**Note:** Currently, you can run custom Move modules and scripts on a local network only, and not on the Libra testnet.
</blockquote>

This tutorial guides you through publishing a Move module and executing a Move transaction script on a local blockchain. To perform operations that are not natively supported by the existing Move transaction scripts, you can create and publish Move modules and write scripts to use these modules. For basic information on Move, refer to [Getting Started with Move](move-overview.md). For deeper technical understanding of Move, refer to the [technical paper](move-paper.md). For guidance on running a local network of nodes, refer to [Run a Local Network](run-local-network.md). The Libra CLI client provides the `dev` command to compile, publish, and execute Move programs locally. Refer to the [CLI Guide - dev command](reference/libra-cli#dev-d-mdash-operations-related-to-move-transaction-scripts-and-modules) for command usage. To see the list of subcommands,  enter `dev` on the CLI.

To create, compile, and publish Move modules to an account on the local blockchain, follow the instructions in [compile and publish Move modules](#compile-and-publish-move-modules). To compile and execute a Move transaction script, follow the instructions in [compile and execute transaction scripts](#compile-and-execute-transaction-scripts).

## Compile and Publish Move Modules

### Start a Local Network of Validator Nodes

To run a local network with one validator node and create a local blockchain, change to the `libra` directory and run `libra-swarm`, as shown below:

```
$ cd libra
$ cargo run -p libra-swarm -- -s
```

This command will take some time to run and it will perform the following actions:

* Spawn a local network with one validator node on your computer.
* Generate genesis transaction, mint key, and bootstrap configuration of the node.
* Start an instance of the Libra CLI client; the client is connected to the local network.

Upon successful execution of the command, you'll see the CLI client menu and the `libra%` prompt as shown below:

```
usage: <command> <args>

Use the following commands:

account | a
  Account operations
query | q
  Query operations
transfer | transferb | t | tb
  <sender_account_address>|<sender_account_ref_id> <receiver_account_address>|<receiver_account_ref_id> <number_of_coins> <currency_code> [gas_unit_price_in_micro_libras (default=0)] [max_gas_amount_in_micro_libras (default 400_000)] Suffix 'b' is for blocking.
  Transfer coins from one account to another.
info | i
  Print cli config and client internal information
dev
  Local Move development
help | h
  Prints this help
quit | q!
  Exit this client


Please, input commands:

libra%
```

For detailed instructions on working with a local cluster of validator nodes, refer to [Run a Local Network](run-local-network.md).

### Create an Account

Each Move module and resource type is hosted by a specific account address. For example, the `Libra` module is hosted by the account at address `0x1`. To import the `Libra` module in other modules or transaction scripts, your Move code would specify `use 0x1::Libra`.

Before publishing a Move module, you first need to create an account to host it:

```
libra% account create
>> Creating/retrieving next local account from wallet
Created/retrieved local account #0 address 717da70a461fef6307990847590ad7af

```

In the above output, 0 is the index of the account you just created, and the hex string is the address of that account. The index is just a convenient way to refer to this account locally, and it's also called `ref_id`. For more information on account creation, refer to [My First Transaction](my-first-transaction.md).

The `create` command generates a local keypair. To create the account on the local blockchain, you'll need to mint money into the account, as shown below:

```
libra% account mintb 0 76 LBR
>> Creating recipient account before minting from faucet
waiting ....
transaction executed!
no events emitted
>> Sending coins from faucet
waiting ....
transaction executed!
Finished sending coins from faucet!
```
To check whether the account was successfully created on the local blockchain, query the account balance.

```
libra% query balance 0
Balance is: 76.000000LBR
```

### Create Move Module

Let’s start with a simple module called `SimpleFee`. This module implements a per-account
processing fee that can be added to payment transactions.
An account can specify an on-chain Move resource with the fee amount that should be charged for
each coin type. The account owner can set the fee amounts, and anyone can retrieve the
fees to add them to payments.
The Move code for this module is provided below. Change the address in the first line to be the address of the account you just created and then save it in a file named `SimpleFee.move.` (Be sure to keep the "0x" prefix on the account address.)

```
address 0x717da70a461fef6307990847590ad7af {

module SimpleFee {
  // A simple fixed-price per-account processing fee that can be added
  // to payment transactions. There is a different fee for each CoinType.
  resource struct Fee<CoinType> {
    fee: u64
  }

  // Set the fee amount.
  public fun set_fee<CoinType>(account: &signer, fee: u64) {
    move_to(account, Fee<CoinType> { fee })
  }

  // Get the fee for an account.
  public fun get_fee<CoinType>(account: address): u64 acquires Fee {
    if (exists<Fee<CoinType>>(account))
      borrow_global<Fee<CoinType>>(account).fee
    else
      0
  }
}
}
```

### Compile Move Module

To compile `SimpleFee.move`, use the [dev compile](reference/libra-cli#dev-d-mdash-operations-related-to-move-transaction-scripts-and-modules) command.

```
libra% dev compile 0 <path to SimpleFee.move> <path to language/stdlib/modules>
```
* 0 &mdash; Index/ref_id of the account that the module will be published under.
* Arguments listed after the source file name specify dependencies, and since this module depends on the Move standard library, you need to specify the path to that directory.

The Move code gets fed into the compiler in a `.move` file and the compiler outputs the corresponding bytecode file. When you are ready to publish this module into an account on the blockchain,  use this bytecode file and not the `.move` file.

After the module is successfully compiled, you'll see the following message in the output, it contains the path to the bytecode file produced by compiling `SimpleFee.move`.

```
Successfully compiled a program at:
  /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/b8639bd9fe2403874bbfde5643486bde/modules/0_SimpleFee.mv
```

### Publish Compiled Module

To publish the module bytecode on your local blockchain, run the [dev publish](reference/libra-cli#dev-d-mdash-operations-related-to-move-transaction-scripts-and-modules) command and use the path to the compiled module bytecode file as shown below:

```
libra% dev publish 0 /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/b8639bd9fe2403874bbfde5643486bde/modules/0_SimpleFee.mv

waiting .....
transaction executed!
no events emitted.
Successfully published module
```
Upon successful execution of the `dev publish` command, the bytecode for `SimpleFee` is published under the sender’s account. To use the procedures and types declared in `SimpleFee`, other transaction scripts and modules can import it with `use <sender_address>::SimpleFee`.

 Subsequent modules published under `<sender_address>` must not be named `SimpleFee`. Each account may hold at most one module with a given name. Attempting to publish a second module named `SimpleFee` under `<sender_address>` will result in a failed transaction.

## Compile and Execute Transaction Scripts

### Create Transaction Scripts

<blockquote class="block_note">

**Note**: You'll find samples of transaction scripts in the [libra/language/stdlib/transaction_scripts](https://github.com/libra/libra/tree/master/language/stdlib/transaction_scripts) directory.
</blockquote>

To use the `SimpleFee` module, let's first create a transaction script to set the fee amount.
Edit the SimpleFee account address in the following script
to match the account that you created and then save it as `set_lbr_fee.move`:

```
script {
use 0x1::LBR::LBR;
use 0x717da70a461fef6307990847590ad7af::SimpleFee;

fun set_lbr_fee(account: &signer, fee: u64) {
  SimpleFee::set_fee<LBR>(account, fee)
}
}
```

(The CLI client does not currently translate currency codes to coin types for custom scripts,
so this example script is hardcoded to use LBR coins.)

You'll also want a custom script to send a payment with an added fee, so in the same way, edit the account address in the following script and save it as `pay_lbr_with_fee.move`:

```
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
use 0x717da70a461fef6307990847590ad7af::SimpleFee;

fun pay_lbr_with_fee(payer: &signer, payee: address, amount: u64) {
  let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
  let total = amount + SimpleFee::get_fee<LBR>(payee);
  LibraAccount::pay_from<LBR>(&payer_withdrawal_cap, payee, total, x"", x"");
  LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}
}
```

### Compile Transaction Scripts

To compile your transaction scripts, use the [dev compile](reference/libra-cli#dev-d-mdash-operations-related-to-move-transaction-scripts-and-modules) command.

```
libra% dev compile 0 <path to set_lbr_fee.move> <path to SimpleFee.move> <path to language/stdlib/modules>
```

 `set_lbr_fee.move` is the Move source file, and upon successful compilation of `set_lbr_fee.move` the compiler will output the corresponding bytecode file. You'll use this bytecode file (not the `.move` file) when you execute this script. After the script is successfully compiled, you'll see the path to the bytecode file in your output:

```
Successfully compiled a program at:
  /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/set_lbr_fee.mv
```

Repeat the compilation steps for the `pay_lbr_with_fee.move` script:

```
libra% dev compile 0 <path to pay_lbr_with_fee.move> <path to SimpleFee.move> <path to language/stdlib/modules>
>> Compiling program
Successfully compiled a program at:
  /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/pay_lbr_with_fee.mv
```

### Execute Transaction Scripts

To execute a custom script, use the [dev execute](reference/libra-cli#dev-d-mdash-operations-related-to-move-transaction-scripts-and-modules) command on the bytecode output from [Compile Transaction Script](#compile-transaction-script) step above. First let's use the `set_lbr_fee` script to specify a fee amount:

<blockquote class="block_note">

**Note:** The exact set of arguments passed to the `dev execute` command will depend on the parameters a particular script expects.
</blockquote>

```
libra% dev execute 0 /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/set_lbr_fee.mv 10000
waiting .....
transaction executed!
no events emitted
Successfully finished execution
```

* `0` &mdash; Index/ref_id of the sender account. For this example, it is the same account which compiled and published the module.
* `/var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/set_lbr_fee.mv` &mdash; Path to the compiled script.
* `10000` &mdash; Amount of the fee in units of micro-Libra (0.01 Libra).

Next, we can set up another account and use the `pay_lbr_with_fee` script to send a payment with the added fee:

```
libra% account create
>> Creating/retrieving next local account from wallet
Created/retrieved local account #1 address aed273e4e7b36276e1442656cc16eb31
libra% account mintb 1 10 LBR
>> Creating recipient account before minting from faucet
waiting ....
transaction executed!
no events emitted
>> Sending coins from faucet
waiting ....
transaction executed!
Finished sending coins from faucet!
libra% dev execute 1 /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/pay_lbr_with_fee.mv 0x717da70a461fef6307990847590ad7af 1000000
waiting ....
transaction executed!
Successfully finished execution
```

* `1` &mdash; Index/ref_id of the sender account, which is the newly created account that will send the payment.
* `/var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/pay_lbr_with_fee.mv` &mdash; Path to the compiled script.
* `0x717da70a461fef6307990847590ad7af` &mdash; The payee account address (account index 0).
* `1000000` &mdash; Amount of the payment in units of micro-Libra (1.0 Libra).

The results of this transaction can be observed by querying the account balances:

```
libra% query balance 0
Balance is: 77.010000LBR
libra% query balance 1
Balance is: 8.990000LBR
```

As expected, the 1.0 Libra payment was increased by the 0.01 Libra fee,
so that 1.01 Libra was transferred from account 1 to account 0.

## Troubleshooting

### Compile Move Program

If the client cannot locate your Move source file, you'll see something like this error:

```
libra% dev compile 0 ~/my-tscripts/set_lbr_fee.move
>> Compiling program
Error: No such file or directory '~/my-tscripts/set_lbr_fee.move'
compilation failed
```

This may happen because the client does not currently perform tilde expansion,
so you need to list the path to your home directory instead.

If you see the following error, refer to the usage of the [dev compile](reference/libra-cli#dev-d-mdash-operations-related-to-move-transaction-scripts-and-modules) command, specify all the required arguments and try compiling again.

```
Invalid number of arguments for compilation
```

### Publish Compiled Module

If you compile a module using one account (e.g., `dev compile` 0 ...) and try to publish it to a different account (e.g., `dev publish` 1 ...), you'll see the following error:

```
libra% dev publish 1 /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/b8639bd9fe2403874bbfde5643486bde/modules/0_SimpleFee.mv

transaction failed to execute; status: VerificationError!

```

A compiled module contains the address of the account where the module is to be published, and the [Move Virtual Machine (VM)](https://developers.libra.org/docs/crates/vm) only allows a transaction sender to publish a module under the sender’s own account address. If this was not true, another user could publish modules under your account! To fix this error, recompile the module using the desired sender address.

If you do not provide the correct path to your compiled module, you'll see this error:

```
libra% dev publish 0 incorrect-path-to-compiled-module
No such file or directory (os error 2)
```
If the account with index 1 does not exist, trying to publish the module to 1 will result in the following error:

```
Unable to find account by account reference id: 1, to see all existing accounts, run: 'account list'
```
Republishing/updating an existing module under the same sender account address does not have any effect on the blockchain. It’s a failed transaction, but it deducts gas from the sender account and increments the sequence number of the sender account by one. It’s possible to publish the same module at a different sender account address, provided the module was compiled using that account address.

### Execute Transaction Script

If the sender account index is invalid, you'll see this error:

```
libra% dev execute 2 /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/set_lbr_fee.mv 10000
Unable to find account by account reference id: 2, to see all existing accounts, run: 'account list'
```

The following error indicates that either the arguments to the transaction script are missing or one or more of the arguments are of the wrong type.

```
libra% dev execute 0 /var/folders/tq/8gxrrmhx16376zxd5r4h9hhn_x1zq3/T/5fa11d0acf5d53e8d257ab31534b2017/scripts/set_lbr_fee.mv
transaction failed to execute; status: VerificationError!

```

## Reference

* [Run a Local Network](run-local-network.md) &mdash; Provides information on running a local network.
* [Getting Started with Move](move-overview.md) &mdash; Introduces you to Move, a new blockchain programming language.
* [Move Technical Paper](move-paper.md).
* Move READMEs:
    * [Move Language](https://developers.libra.org/docs/crates/move-language).
    * [Move IR Compiler](https://developers.libra.org/docs/crates/ir-to-bytecode).
    * [Bytecode Verifier](https://l.facebook.com/l.php?u=https%3A%2F%2Fdevelopers.libra.org%2Fdocs%2Fcrates%2Fbytecode-verifier&h=AT22hXPt7Fjx80GBMVQ5NOZaVAvQRzD-W4QLZK3j44-Jk11H7EzR7RpTqJpaWX0FMSWFcMdhlvfSTw7TVYk15xAC2fd520s8erlICkc4F_AMTOWrMowCqqG5Qv8RLXROLXZ1MTxGMGq4L1J7czZSas5l).
    * [Virtual Machine](https://developers.libra.org/docs/crates/vm).
* [CLI Guide](reference/libra-cli.md) — Lists the commands of the Libra CLI client.
* [My First Transaction](my-first-transaction.md) &mdash; Guides you through executing your very first transaction on the Libra Blockchain using the Libra CLI client.
