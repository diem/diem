## Type CurrencyInfo

| Name                            | Type           | Description                                                                |
|---------------------------------|----------------|----------------------------------------------------------------------------|
| code                            | string         | Currency Code                                                              |
| fractional_part                 | unsigned int64 | Max fractional part of single unit of currency allowed in a transaction    |
| scaling_factor                  | unsigned int64 | Factor by which the amount is scaled before it is stored in the blockchain |
| to_lbr_exchange_rate            | float32        | Exchange rate of the currency to LBR currency                              |
| mint_events_key                 | string         | Unique key for the mint events stream of this currency                     |
| burn_events_key                 | string         | Unique key for the burn events stream of this currency                     |
| preburn_events_key              | string         | Unique key for the preburn events stream of this currency                  |
| cancel_burn_events_key          | string         | Unique key for the cancel burn events stream of this currency              |
| exchange_rate_update_events_key | string         | Unique key for the exchange rate update events stream of this currency     |


### Example


``` json
{
  "id": 1,
  "jsonrpc": "2.0",
  "libra_chain_id": 2,
  "libra_ledger_timestampusec": 1596680410015647,
  "libra_ledger_version": 3252698,
  "result": [
    {
      "burn_events_key": "02000000000000000000000000000000000000000a550c18",
      "cancel_burn_events_key": "04000000000000000000000000000000000000000a550c18",
      "code": "Coin1",
      "exchange_rate_update_events_key": "05000000000000000000000000000000000000000a550c18",
      "fractional_part": 100,
      "mint_events_key": "01000000000000000000000000000000000000000a550c18",
      "preburn_events_key": "03000000000000000000000000000000000000000a550c18",
      "scaling_factor": 1000000,
      "to_lbr_exchange_rate": 0.5
    },
    {
      "burn_events_key": "07000000000000000000000000000000000000000a550c18",
      "cancel_burn_events_key": "09000000000000000000000000000000000000000a550c18",
      "code": "Coin2",
      "exchange_rate_update_events_key": "0a000000000000000000000000000000000000000a550c18",
      "fractional_part": 100,
      "mint_events_key": "06000000000000000000000000000000000000000a550c18",
      "preburn_events_key": "08000000000000000000000000000000000000000a550c18",
      "scaling_factor": 1000000,
      "to_lbr_exchange_rate": 0.5
    },
    {
      "burn_events_key": "0c000000000000000000000000000000000000000a550c18",
      "cancel_burn_events_key": "0e000000000000000000000000000000000000000a550c18",
      "code": "LBR",
      "exchange_rate_update_events_key": "0f000000000000000000000000000000000000000a550c18",
      "fractional_part": 1000,
      "mint_events_key": "0b000000000000000000000000000000000000000a550c18",
      "preburn_events_key": "0d000000000000000000000000000000000000000a550c18",
      "scaling_factor": 1000000,
      "to_lbr_exchange_rate": 1
    }
  ]
}

```
