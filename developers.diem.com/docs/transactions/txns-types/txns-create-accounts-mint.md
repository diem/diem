---
title: "Create accounts and mint"
slug: "txns-create-accounts-mint"
hidden: false
---
Depending on the type of network you are using, you can use transactions to:
* [Create accounts and mint fake Diem Coins in testnet](/docs/transactions/txns-types/txns-create-accounts-mint#create-accounts-mint-in-testnet)
* [Create a ChildVASP account in mainnet and pre-mainnet](/docs/transactions/txns-types/txns-create-accounts-mint#create-childvasp-accounts-in-mainnet-pre-mainnet)

## Create accounts, mint in testnet

If you are using [testnet](../reference/glossary#tesnet), you can use the [Faucet](../reference/glossary#faucet) service to perform certain actions that only by performed by the Diem Association on [mainnet](../reference/glossary#mainnet). You can do the following by sending requests to the faucet service endpoint:
* Create a [ParentVASP account](../reference/glossary#parentvasp-account).
* Mint and add fake Diem Coins to accounts for testing.

To create a ParentVASP account in testnet, send a request like the code example below to the faucet server:
```http request
http://<faucet server address>/?amount=<amount>&auth_key=<auth_key>&currency_code=<currency_code>
```

In this example request,

* `auth_key` is the authorization key for an account.
* `amount` is the amount of money available as the account balance.
* `currency_code` is the specified currency for the amount.

This request first checks if there is an account available for the authorization key specified. If the account given by `auth_key` doesn’t exist, a ParentVASP account will be created with the balance of the `amount` in the `currency_code` specified. If the account already exists, this will simply give the account the specified amount of `currency_code` Diem Coins.

## Create ChildVASP accounts in mainnet, pre-mainnet

If you are a Regulated VASP, and have been approved by Diem Networks as a participant on the Diem Payment Network (DPN), you first need to complete an application with Diem Networks to have a ParentVASP account created on your behalf.

Once Diem Networks creates your ParentVASP account (let’s call it Account **A**), you can create a [ChildVASP account](../reference/glossary#childvasp-account) if you wish.

To create a ChildVASP account, send the [create_child_vasp_account](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#script-create_child_vasp_account) transaction script from your **Account A** (your ParentVASP account).

With a single ParentVASP account, you can create up to 256 ChildVASP accounts. This transaction script allows you to specify:
* Which currency the new account should hold, or if it should hold all known currencies.
* If you want to initialize the ChildVASP account with a specified amount of coins in a given currency.

A Regulated VASP can purchase Diem Coins from a Designated Dealer.
