---
id: move-overview
title: Getting Started With Move
---

Move is a new programming language developed to provide a safe and programmable foundation for the Libra Blockchain. An account in the Libra Blockchain is a container for an arbitrary number of Move resources and Move modules. Every transaction submitted to the Libra Blockchain uses a transaction script written in Move to encode its logic. The transaction script can call procedures declared by a module to update the global state of the blockchain.

In the first part of this guide, we will provide a high-level introduction to the key features of the Move language:

1. [Move Transaction Scripts Enable Programmable Transactions](#move-transaction-scripts-enable-programmable-transactions)
2. [Move Modules Allow Composable Smart Contracts](#move-modules-allow-composable-smart-contracts)
3. [Move Has First Class Resources](#move-has-first-class-resources)

For the curious reader, the [Move technical paper](move-paper.md) contains much more detail about the language.

In the second part of this guide, we will “look under the hood” and show you how to write your own Move programs in the [Move source language](#move-source-language). Custom Move programs are not supported in the initial testnet release, but these features are available for you to try out locally.

## Key Features of Move

### Move Transaction Scripts Enable Programmable Transactions

- Each Libra transaction includes a **Move transaction script** that encodes the logic a validator should perform on the client's behalf (for example, to transfer Libra from Alice's account to Bob's account).
- The transaction script interacts with [Move resources](#move-has-first-class-resources) published in the global storage of the Libra Blockchain by calling the procedures of one or more [Move modules](#move-modules-allow-composable-smart-contracts).
- A transaction script is not stored in the global state, and it cannot be invoked by other transaction scripts. It is a single-use program.
- We present several examples of transaction scripts in [Writing Transaction Scripts](#writing-transaction-scripts).

### Move Modules Allow Composable Smart Contracts

Move modules define the rules for updating the global state of the Libra Blockchain. Modules fill the same niche as smart contracts in other blockchain systems. Modules declare [resource](#move-has-first-class-resources) types that can be published under user accounts. Each account in the Libra Blockchain is a container for an arbitrary number of Move resources and Move modules.

- A module declares both struct types (including resources, which are a special kind of struct) and procedures.
- The procedures of a Move module define the rules for creating, accessing, and destroying the types it declares.
- Modules are reusable. A struct type declared in one module can use struct types declared in another module, and a procedure declared in one module can invoke public procedures declared in another module. A module can invoke procedures declared in other Move modules. Transaction scripts can invoke any public procedure of a published module.
- Eventually, we expect that Libra users will be able to publish modules under their own accounts.

### Move Has First Class Resources

- The key feature of Move is the ability to define custom resource types. Resource types are used to encode safe digital assets with rich programmability.
- Resources are ordinary values in the language. They can be stored as data structures, passed as arguments to procedures, returned from procedures, and so on.
- The Move type system provides special safety guarantees for resources. Move resources can never be duplicated, reused, or discarded. A resource type can only be created or destroyed by the module that defines the type. These guarantees are enforced statically by the [Move virtual machine](reference/glossary.md#move-virtual-machine-mvm) via bytecode verification. The Move virtual machine will refuse to run code that has not passed through the bytecode verifier.
- All Libra currencies are implemented using the generic `Libra` type. For example: the LBR currency is represented as `Libra<LBR>` and a hypothetical USD currency would be represented as `Libra<USD>`. `Libra` has no special status in the language; every Move resource enjoys the same protections.

## Move: Under the Hood

### Move Source Language

This section describes how to write [transaction scripts](#writing-transaction-scripts) and [modules](#writing-modules) in the Move source language. We will proceed by presenting snippets of heavily-commented Move code. We encourage readers to follow along with the examples by compiling, running, and modifying them locally. The README files `libra/language/README.md` and `libra/language/move-lang/README.md` explain how to do this.

### Writing Transaction Scripts

As we explained in [Move Transaction Scripts Enable Programmable Transactions](#move-transaction-scripts-enable-programmable-transactions), users write transaction scripts to request updates to the global storage of the Libra Blockchain. There are two important building blocks that will appear in almost any transaction script: the `LibraAccount` and `Libra` resource types.

When we say that a user "has an account at address `0xff` on the Libra Blockchain", what we mean is that the address `0xff` holds an instance of the `LibraAccount` resource. Every nonempty address has a `LibraAccount` resource. This resource stores account data, such as the sequence number, authentication key, and balance. Any part of the Libra system that wants to interact with an account must do so by reading data from the `LibraAccount` resource or invoking procedures of the `LibraAccount` module.

The account balance is a generic resource of type `LibraAccount::Balance`. (`LibraAccount` is the name of the module, and `Balance` is the name of a resource declared by that module, in addition to the `LibraAccount` resource.) A single Libra account may hold balances in multiple currencies. For example, an account that holds both LBR and USD would have a `LibraAccount::Balance<LBR>` resource and a `LibraAccount::Balance<USD>` resource. However, every account must have at least one balance.

A `Balance<Token>` resource holds a resource of type `Libra<Token>`. As we explained in [Move Has First Class Resources](#move-has-first-class-resources), this is the generic type of a Libra coin. This type is a "first-class citizen" in the language just like any other Move resource. Resources of type `Libra` can be stored in program variables, passed between procedures, and so on.

We encourage the interested reader to examine the Move definitions of these two key resources in the `LibraAccount` and `Libra` modules under the `libra/language/stdlib/modules/` directory.

Now let us see how a programmer can interact with these modules and resources in a transaction script.

```move
// Simple peer-peer payment example.

script {
// Use the LibraAccount module published on the blockchain at account address
// 0x0...1. 0x1 is shorthand that the language pads out to
// 16 bytes by adding leading zeroes.
use 0x1::LibraAccount;
// Use the LBR resource from the LBR module. Naming the resource here makes
// it possible to reference the resource without the module name.
use 0x1::LBR::LBR;

fun peer_to_peer_lbr_payment(payer: &signer, payee: address, amount: u64) {
  // Acquire the capability to withdraw from the payer's account.
  let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
  // Withdraw a value of `amount` LBR from the payer's account and deposit
  // it into the account at the address `payee`. This will fail if the
  // payer's balance is less than `amount` or if there is no account at the
  // address `payee`. The `metadata` and `metadata_signature` arguments are
  // not specified, so this could also fail if the transaction requires
  // that information.
  LibraAccount::pay_from<LBR>(&payer_withdrawal_cap, payee, amount, x"", x"");
  // Restore the capability back to the payer's account.
  LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}
}
```

For more examples, including the transaction scripts supported in the initial testnet, refer to `libra/language/stdlib/transaction_scripts`.

### Writing Modules

We will now turn our attention to writing our own Move modules instead of just reusing the existing `LibraAccount` and `Libra` modules. Consider this situation:
Bob is going to create an account at address _a_ at some point in the future. Alice wants to "earmark" some of her coins for Bob so that he can pull them into his account once it is created. But she also wants to be able to remove the earmark from her coins if, for example, Bob never creates the account.

To solve this problem for Alice, we will write a module `EarmarkedLibra` which:

- Declares a new resource type `EarmarkedLibra` that wraps a `Libra` and recipient address.
- Allows Alice to create such a type and publish it under her account (the `create` procedure).
- Allows Bob to claim the resource (the `claim_for_recipient` procedure).
- Allows anyone with an `EarmarkedLibra` to destroy it and acquire the underlying coin (the `unwrap` procedure).

```move
// The address where the module will be published:
address 0xcc2219df031a68115fad9aee98e051e9 {

// A module for earmarking a coin for a specific recipient
module EarmarkedLibraCoin {
  use 0x1::Libra::Libra;
  use 0x1::Signer;

  // A wrapper containing a generic Libra and the address of the recipient the
  // coin is earmarked for.
  resource struct EarmarkedLibraCoin<Token> {
    coin: Libra<Token>,
    recipient: address
  }

  // Error codes
  const EWRONG_RECIPIENT: u64 = 0;

  // Create a new earmarked coin with the given `recipient`.
  // Publish the coin under the transaction sender's account address.
  public fun create<Token>(sender: &signer, coin: Libra<Token>, recipient: address) {
    // Construct or "pack" a new EarmarkedLibraCoin resource. Only procedures of the
    // `EarmarkedLibraCoin` module can create that resource.
    let t = EarmarkedLibraCoin { coin, recipient };

    // Publish the earmarked coin under the transaction sender's account
    // address. Each account can contain at most one resource of a given type;
    // this call will fail if the sender already has a resource of this type.
    move_to(sender, t);
  }

  // Allow the transaction sender to claim a coin that was earmarked for her.
  public fun claim_for_recipient<Token>(
    sender: &signer,
    earmarked_coin_address: address
  ): Libra<Token> acquires EarmarkedLibraCoin {
    // Remove the earmarked coin resource published under `earmarked_coin_address`
    // and "unpack" it to remove `coin and `recipient`.
    // If there is no resource of type EarmarkedLibraCoin<Token> published under
    // the address, this will fail.
    let EarmarkedLibraCoin { coin, recipient } =
      move_from<EarmarkedLibraCoin<Token>>(earmarked_coin_address);

    // Ensure that the transaction sender is the recipient. If this assertion
    // fails, the transaction will fail and none of its effects (e.g.,
    // removing the earmarked coin) will be committed.
    assert(recipient == Signer::address_of(sender), EWRONG_RECIPIENT);

    // Return the coin
    coin
  }

  // Allow the creator of the earmarked coin to reclaim it.
  public fun claim_for_creator<Token>(
    sender: &signer
  ): Libra<Token> acquires EarmarkedLibraCoin {
    let EarmarkedLibraCoin { coin, recipient:_ } =
      move_from<EarmarkedLibraCoin<Token>>(Signer::address_of(sender));
    coin
  }
}
}
```

Alice can create an earmarked coin for Bob by creating a transaction script that invokes `create` on Bob's address _a_ and a `Libra` that she owns. Once _a_ has been created, Bob can claim the coin by sending a transaction from _a_. This invokes `claim_for_recipient` to claim a `Libra` for Bob that he can store in whichever address he wishes. If Bob takes too long to create an account under _a_ and Alice wants to remove the earmark from her coins, she can do so by using `claim_for_creator`.
