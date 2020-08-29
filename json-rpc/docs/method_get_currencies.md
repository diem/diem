## Method get_currencies

**Description**

Get information about various currencies supported by the Libra blockchain


### Parameters

None


### Returns

Returns array of [CurrencyInfo](type_currency_info.md) objects.

### Example


```
// Request: fetches currencies supported by the system
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_currencies","params":[],"id":1}' https://testnet.libra.org/v1

// Response
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
