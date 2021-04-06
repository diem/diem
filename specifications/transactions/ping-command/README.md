# PingCommand
The `PingCommand` primarily provides a robust method for health-checking an off-chain service. In addition, its simplicity makes it an ideal candidate as the first milestone for validating a correctly implemented off-chain service. Upon receipt of a `PingCommand` the recipient responds with an empty `CommandResponseObject` completing the exchange. Hence this protocol has no requirements for persistency, so long as the remote service is available and online, it is expected to respond.

`PingCommand` Object:
```
{
    "_ObjectType": "CommandRequestObject",
    "command_type": "PingCommand",
    "command": {
	    "_ObjectType": "PingCommand",
    },
    "cid": "12ce83f6-6d18-0d6e-08b6-c00fdbbf085a",
}
```

| Field | Type | Required? | Description |
|-------|------|-----------|-------------|
|`_ObjectType` | str | Y | The fixed string PingCommand |
