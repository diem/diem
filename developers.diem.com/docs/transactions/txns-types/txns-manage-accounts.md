---
title: "Manage accounts"
slug: "txns-manage-accounts"
hidden: false
---
Once an account has been created, you can use different types of transactions to manage the following account administrative tasks:
* [Create a recovery address account](/docs/transactions/txns-types/txns-manage-accounts#create-a-recovery-address-account)
* [Rotate authentication key](/docs/transactions/txns-types/txns-manage-accounts#rotate-authentication-key)
* [Add a currency to an account](/docs/transactions/txns-types/txns-manage-accounts#add-a-currency-to-an-account)

## Create a recovery address account

If you are a Regulated VASP, you can designate one of your accounts as a recovery address account. The recovery address account should be a cold account (i.e., no transactions should be planned to be sent from that account). Use the recovery address account only for rotating the authentication key of an account that has registered itself with it and whose private key has been lost. 

For this example, let's call this account **R**. To create recovery address account **R**, send a [create_recovery_address](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#function-create_recovery_address) transaction from account **R**. 

After the recovery address **R** has been created, other accounts that belong to the VASP can register themselves with R by sending a [add_recovery_rotation_capability](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#function-add_recovery_rotation_capability) transaction and specifying the recovery address as **R**.


## Rotate authentication key

If an account **A** wishes to update the authentication key needed to access it, it can do so by sending one of two transactions, depending on whether A has been registered with, or is, an account recovery address.

* If **A** has not registered itself with a recovery address, it can change its authentication key by sending a [rotate_authentication_key](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#function-rotate_authentication_key) transaction with its new auth key. 
* If **A** is part of a recovery address, then it can rotate its key by sending a [rotate_authentication_key_with_recovery_address](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#function-rotate_authentication_key_with_recovery_address) transaction with its new authentication key, and itself as the `to_recover` address.


## Add a currency to an account 

If an account **A** wants to hold a balance of Diem Coins in a currency **C** that it currently doesn’t hold, it can add a balance for **C** by sending an [add_currency_to_account](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#0x1_AccountAdministrationScripts_add_currency_to_account) transaction specifying **C** as the currency, from **A**. 

It’s important to note that **C** must be a recognized currency on-chain, and **A** cannot hold a balance in **C** already; otherwise, this transaction will fail to execute.