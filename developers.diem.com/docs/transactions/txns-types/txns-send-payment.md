---
title: "Send a payment"
slug: "txns-send-payment"
hidden: false
---
An account can send a payment to another account by sending a transaction. 

## Introduction

If an account **A** wishes to send a payment to another account **B,** it can do so by sending a [peer_to_peer_with_metadata](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#script-peer_to_peer_with_metadata) script transaction. 

When a payment is made, the sender must specify:
* The currency the payment is being made in
* The amount to send
* The account the payment is being made to, which in this example is account **B**. 

When constructing a transaction, account **A** can also specify the metadata parameter. This parameter can be of any form as long as **A** and **B** agree on it, subject to certain rules specified in the agreement between a [Regulated VASP](../reference/glossary# and Diem Networks, and a  `metadata_signature` used for dual attestation.

## Dual attestation

Every transaction sending payments between two distinct Regulated VASP accounts must perform dual attestation whenever the amount sent exceeds a certain threshold (which is currently $1,000) in order to comply with the Travel Rule. 

"Dual attestation" means the VASP sending the transaction (sending VASP) must send the VASP receiving (receiving VASP) the transaction certain data. 
* This data is sent to the endpoint given by the recipient’s `base_url`. 
* The receiving VASP performs checks on this data. 
* Once the checks are completed, the receiving VASP then signs the data with the compliance private key that corresponds to their `compliance_public_key` held on-chain, and sends it back to the sending VASP. 
* The sending VASP must then include this signed data in the payment transaction and the signature will be checked on-chain.
 
### Update dual attestation information

If a Regulated VASP wishes to update their `base_url` or `compliance_public_key`, it can do so by sending a [rotate_dual_attestation_info](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#script-rotate_dual_attestation_info) transaction. The Regulated VASP has to send this transaction from their ParentVASP account (this is the account that holds dual attestation data). For example, if a Regulated VASP wishes to change the private key they use to sign metadata for dual attestation, they can send this transaction to do so. 

Once this transaction is executed, all transactions subject to dual attestation will be checked using the new `compliance_public_key`. Because of this, Regulated VASPs should be careful to communicate this change and ensure that there are no outstanding payment transactions that they have previously signed but that have not yet been committed on-chain, since these will be rejected if the `compliance_public_key` has changed.