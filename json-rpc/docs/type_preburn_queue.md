## PreburnQueue - type

### Attributes

| Name       | Type                                                   | Description                                                  |
| ---------- | --------------------------------                       | -------------------------------------------------------      |
| currency   | string                                                 | The currency that all preburn requests in this queue are in. |
| preburns   | List<[PreburnWithMetadata](#type-preburnwithmetadata)> | A list of preburn requests. All in the same currency.        |


## Type PreburnWithMetadata

| Name       | Type                             | Description                                                                                                      |
| ---------- | -------------------------------- | -------------------------------------------------------                                                          |
| preburn    | [Amount](type_amount.md)         | The value and currency of the preburn request.                                                                   |
| metadata   | string                           | Hex-encoded string of the metadata associated with this preburn request. Null if no metadata associated with it. |
