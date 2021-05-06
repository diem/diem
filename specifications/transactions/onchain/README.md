# On-Chain Data and Transactions

Diem transactions mutate and create state (or resources) within the set of [on-chain modules](https://github.com/diem/diem/tree/main/language/diem-framework/modules), primarily the [Diem Account](https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/DiemAccount.md). The transaction format is defined in the [Move Adapter Specification](https://github.com/diem/diem/blob/main/specifications/move_adapter/README.md). Most participants of the Diem Payment Network (DPN) will submit SignedTransactions containing a [script function](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md). Before release 1.2, clients used scripts. These can be accessed in [compiled form](https://github.com/diem/diem/tree/release-1.1/language/stdlib/compiled/transaction_scripts) and in their [original form](https://github.com/diem/diem/tree/release-1.1/language/stdlib/transaction_scripts). The DPN MainNet only allows script functions and this set of pre-registerd scripts to be submitted. Due to the evolving nature of Move and the Move compiler, compiling existing scripts may not result in the form stored in the directory stored above. Hence, it is recommended to use script functions where available or otherwise the compiled scripts.

## Peer to Peer Payments and Transaction Metadata

Most transactions will use the [peer_to_peer_with_metadata script function](https://github.com/diem/diem/blob/main/language/diem-framework/script_documentation/script_documentation.md#0x1_PaymentScripts_peer_to_peer_with_metadata). This single transaction represents all current transfers between two participants and distinguishes the types of transfers via the embedded metadata.

The metadata is represented by the following `Rust` enum encoded in [Binary Canonical Serialization (BCS)](https://github.com/diem/bcs):

```
enum Metadata {
  Undefined,
  GeneralMetadata(GeneralMetadata),
  TravelRuleMetadata(TravelRuleMetadata),
  UnstructuredByteMetadata(Option<Vec<u8>>),
  RefundMetadata(RefundMetadata),
  CoinTradeMetadata(CoinTradeMetadata),
}
```

Note: This is the canonical list and should be referred to in future DIPs so that authors need not reproduce the list in future DIPs.

## Payments Using GeneralMetadata

```
enum GeneralMetadata {
   GeneralMetadataV0(GeneralMetadataV0),
}

struct GeneralMetadataV0 {
   to_subaddress: Option<Vec<u8>>,
   from_subaddress: Option<Vec<u8>>,
   referenced_event: Option<u64>, // Deprecated
}
```

GeneralMetadata leverages the notion of subaddresses to indicate a source and destination and are stored in the fields `from_subaddress` and `to_subaddress`, respectively.

Subaddresses have the following properties:
* 8-bytes
* Subaddresses should be unique
* The address represented by 8 zero bytes (or None/Null within the GeneralMetadataV0) is reserved to denote the root (VASP owned) account

Lifetime of subaddresses:
* There are no explicit lifetimes of subaddresses
* The same `from_address` may receive multiple payments from distinct `to_subaddress`
* A `from_subaddress` may be the recipient or a `to_subaddress` in an ensuing transaction
* `to_subaddress` should be generated fresh each time upon user request
* `from_subaddress` should be unique for each transaction

Subaddresses should be used with great care to not accidentally leak personally identifiable information (PII). However, implementors must be mindful of the permissive nature of subaddresses as outlined in this specification.

## Payments Using TravelRuleMetadata

```
enum TravelRuleMetadata {
   TravelRuleMetadataVersionV0(OffChainReferenceId),
}

type OffChainReferenceId = Option<String>;
```

The TravelRuleMetadata completes a transaction that began with a pre-flight, or off-chain exchange. During this exchange, the two parties should have agreed on an `OffChainReferenceId`, likley a `reference_id` or a [UUID-128](https://tools.ietf.org/html/rfc4122).

## Coin Trades with Designated Dealers Using CoinTradeMetadata

```
enum CoinTradeMetadata {
    CoinTradeMetadataVersion0(TradeIds),
}

type TradeIds = Vec<String>;
```

As defined in [DIP-12](https://dip.diem.com/dip-12/), a coin trade transaction involves a VASP selling or purchasing coins to or from a DD. Each coin trade is represented by a trade id, which is an identifier agreed to by the VASP and DD off-chain. One or more of these off-chain interactions can be settled on-chain via a CoinTradeMetadata transaction.

## Refunds Using RefundMetadata

```
enum RefundMetadata {
  RefundMetadataV0(Refund),
}

struct Refund {
  transaction_version: u64,
  reason: RefundReason,
}


enum RefundReason {
  OtherReason = 0,
  InvalidSubaddress = 1,
  UserInitiatedPartialRefund = 2,
  UserInitiatedFullRefund = 3,
  InvalidReferenceId = 4,
}
```

Diem supports refund transactions, as defined in [DIP-4](https://dip.diem.com/dip-4/). The refund includes the `transaction_version`, a globally unique value, for the transaction that is being refunded as well as the reason for the refund. For example, an errant transaction to a non-existent destination as well as user-driven refunds. However, use of the refund type is optional.

Participants can be configured to automatically refund invalid transactions but in order to prevent ping-pong or recursive refunds, only a single transaction should be sent per peer per transaction stream. That is if a payment is followed by a invalid refund no follow up refund should be issued. Instead this should be surfaced to directly to the other party as this is an implementation bug.

For transactions that exceed the travel rule limit must use the off-chain travel rule protocol and will likely be refunded by a TravelRuleMetadata. All other transactions, may use the refund transaction but are not strictly required to do so. Usage of the refund transaction, however, makes it clear what is a refund and the intent or reason for it.

## Dual Attestation Credentials

Diem defines a [DualAttestation::Credential](https://github.com/diem/diem/blob/main/language/diem-framework/modules/DualAttestation.move) resource to support off-chain protocols. This resource contains the `human_name`, `base_url`, and `compliance_public_key` for a VASP. The `base_url` specifies where the VASP hosts its off-chain API and the `compliance_public_key` is used to verify signed transaction metadata and establish authentication in off-chain communication. The values can be set and updated via the [rotate_dual_attestation_info](https://github.com/diem/diem/blob/main/language/diem-framework/transaction_scripts/rotate_dual_attestation_info.move) script.
