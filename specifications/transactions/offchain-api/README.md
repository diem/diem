# Diem Off-Chain Protocol
As defined in [DIP-1](https://dip.diem.com/dip-1), the Diem Off-Chain Protocol defines an API and payload specification to privately exchange information between two parties that cannot (easily) be achieved directly on a blockchain. This information includes the sensitive details of the parties involved in a payment that should remain off-chain. The information is exchanged within an authenticated and encrypted channel and only made available to the parties that are directly involved.

## Overview
The two parties that participate in the off-chain protocol communicate through HTTP requests and responses within a TLS channel. Each request and response is signed via [JSON Web Signature (JWS)](https://tools.ietf.org/html/rfc7515) to ensure authenticity and integrity. Consider an example in which VASP A sends a command to VASP B:

* VASP A knows VASP B’s on-chain account and obtains the url from VASP B's `DualAttestation::Credential::base_url` resource
* VASP A connects to this endpoint and establishes a tls-encrypted HTTP channel
* VASP A sends a request across the channel
  * VASP A, first, constructs message containing a `CommandRequestObject`
  * VASP A, then, signs the message it with the Ed25519 private key that maps to the public key stored at VASP A’s `DualAttestation::Credential::compliance_public_key`
  * VASP A, finally transmits the signed message across the channel
* VASP B receives the `CommandRequestObject`
  * VASP B, first, validates the request message
  * VASP B, then, evaluates and responds to the message constructing `CommandResponseObject`
  * VASP B signs the response with the Ed25519 private key that maps to the public key stored at VASP B’s `DualAttestation::Credential::compliance_public_key`
* VASP A receives the `CommandResponseObject`, validates the response, and concludes the exchange

## Specification
### On-Chain Account Setup

In order to participate in the an off-chain protocol exchange, a VASP must have a valid Parent VASP account, as defined in [DIP-2](https://dip.diem.com/dip-2). In addition, the VASP must specify a valid [DualAttestation::Credential](https://github.com/diem/diem/blob/release-1.1/language/stdlib/modules/DualAttestation.move)’s `compliance_public_key` and `base_url`.

The `compliance_public_key` maps to an Ed25519 private key with which the VASP uses to authenticate all off-chain requests and responses.

The `base_url` defines the endpoint for handling off-chain requests. The `base_url` should be unique for each Parent VASP account. That is the off-chain framework does not inherently support multiplexing across accounts or chains when distinct `compliance_public_key`s are used for each.

### HTTP End-point

Each VASP exposes an HTTPS POST end point at `https://hostname:<port>/<protocol_version>/command`. The `protocol_version` is v2 for the currente off-chain APIs.

The base url value must be the url without the path `/<protocol_version>/command`: `https://<hostname>:<port>`.

### HTTP Headers

All HTTP requests must contain:

* A header `X-REQUEST-ID` with a unique UUID (according to [RFC4122](https://tools.ietf.org/html/rfc4122) with “-”'s included) for the request, used for tracking requests and debugging. Responses to requests must have the same value as the request's `X-REQUEST-ID` header.
* A header `X-REQUEST-SENDER-ADDRESS` with the HTTP request sender’s VASP [DIP-5](https://dip.diem.com/dip-5/) address used in the command object.

The HTTP request sender must use the compliance key of the VASP account linked with the sender's address to sign the request JWS body. The request receiver uses this address to find the appropriate compliance key to verify the signed request. For example: VASP A transfers funds to VASP B. The HTTP request A sends to B contains `X-REQUEST-SENDER-ADDRESS` as VASP A’s address.

All HTTP responses must contain:

* A header `X-REQUEST-ID` copied from the HTTP request.

The HTTP response sender must use the compliance key of the VASP account linked with the responder's address to sign the response JWS body.

### Payloads

The payloads between two endpoints must:
* Support both single Command request-responses (HTTP 1.0) and pipelined request-responses (HTTP 1.1)
* Conform to the JWS standard with Ed25119 / EdDSA cipher suite, compact encoding, and be signed with the party's on-chain compliance key
* The content within the JWS payload must be valid JSON objects
* The content type of the HTTP request/response is unspecified and may be ignored

## Request and Response

All requests between VASPs are structured as a `CommandRequestObject` and all responses are structured as a `CommandResponseObject`. The resulting request takes a form of the following (prior to JWS signing):

```
{
    "_ObjectType": "CommandRequestObject",
    "command_type": "SomeCommand", // Command type
    "command": SomeCommandObject(), // Object of type as specified by command_type
    "cid": "12ce83f6-6d18-0d6e-08b6-c00fdbbf085a",
}
```

A response would look like the following:

```
{
    "_ObjectType": "CommandResponseObject",
    "status": "success",
    "cid": "12ce83f6-6d18-0d6e-08b6-c00fdbbf085a"
}
```

### CommandRequestObject

All requests between VASPs are structured as a `CommandRequestObject`.

| Field 	   | Type 	| Required? 	| Description 	|
|-------	   |------	|-----------	|-------------	|
| _ObjectType  | str    | Y | Fixed value: `CommandRequestObject`|
| command_type | str    | Y | A string representing the type of Command contained in the request. |
| command      | Command object | Y | The Command to sequence. |
| cid          | str    | Y     | A unique identifier for the Command. Must be a UUID according to [RFC4122](https://tools.ietf.org/html/rfc4122) with "-"'s included. |

```
{
    "_ObjectType": "CommandRequestObject",
    "command_type": CommandType,
    "command": CommandObject(),
    "cid": str,
}
```

### CommandResponseObject

All responses to a CommandRequestObject are in the form of a CommandResponseObject

| Field 	     | Type 	| Required? 	| Description 	|
|-------	     |------	|-----------	|-------------	|
| _ObjectType    | str      | Y             | The fixed string `CommandResponseObject`. |
| status         | str      | Y             | Either `success` or `failure`. |
| error          | [OffChainErrorObject](#offchainerrorobject) | N | Details of the error when status == "failure". |
| result         | Object   | N | An optional JSON object that may be defined when status == "success". |
| cid            | str      | N | The Command identifier to which this is a response. Must be a UUID according to [RFC4122](https://tools.ietf.org/html/rfc4122) with "-"'s included and must match the 'cid' of the CommandRequestObject. This field must be set unless the request to which this is responding is unparseable. |

Failure:
```
{
    "_ObjectType": "CommandResponseObject",
    "error": OffChainErrorObject(),
    "status": "failure"
    "cid": str,
}
```

Success:
```
{
    "_ObjectType": "CommandResponseObject",
    "result": Object(),
    "status": "success",
    "cid": str,
}
```

#### OffChainErrorObject

Represents an error that occurred in response to a Command.

| Field   | Type 	   | Required? | Description |
|---------|------------|-----------|-------------|
| type    | str (enum) | Y         | Either "command_error" or "protocol_error". |
| field   | str        | N         | The field on which this error occurred. |
| code    | str (enum) | Y         | The error code of the corresponding error. |
| message | str        | N         | Additional details about this error. |

```
{
    "type": "protocol_error",
    "field": "cid",
    "code": "missing_field",
    "message": "",
}
```

##### Protocol Errors

Use the type `protocol_error` when:
1. The request has invalid or missing HTTP headers.
2. A request's `CommandRequestObject`'s fields have validation errors. The CommandRequestObject#command_type must also match the CommandRequestObject#command, but further validation errors are flagged as `command_error`s.

##### Command Errors

Use the type command_error when:
1. A reqeusts CommandRequestObject#command fields fail to validate.
2. Unexpected, internal errors.

##### HTTP Responses

* When a validation error occurs, the HTTP response status code should be 400.
* When a server internal error occurs, HTTP response status code should be 500.

For example, if an off-chain service cannot access its Diem service, it should respond with a 500 status code along with the JWS ecndoed CommandResponseObject.

##### List of Error Codes

The following sections list all error codes for various validations when processing an inbound command request.

| Error code              | Description |
|-------------------------|-------------|
| `invalid_http_header`   | `X-REQUEST-SENDER-ADDRESS` is not a valid account address, no such account exists on-chain, or the `compliance_public_key` is not a valid Ed25519 key <br/> `X-REQUEST-ID` is not a valid UUID |
| `missing_http_header`   | Missing a required HTTP header `X-REQUEST-ID` or `X-REQUEST-SENDER-ADDRESS` |
| `invalid_jws`           | Invalid JWS |
| `invalid_jws_signature` | JWS signature verification failed |
| `invalid_json`          | Decoded JWS body is not valid json |
| `invalid_object`        | Command request/response object json is not object, or the command object type does not match `command_type` |
| `missing_field`         | A required field contains no data or is missing |
| `unknown_field`         | An object contains an unknown field |
| `unknown_command_type`  | Received an invalid/unsupported `command_type` |
| `invalid_field_value`   | A field's value does not match the expected type |

## Network Error Handling

In the case of network failure, the sending party for a Command is expected to re-send the Command until it gets a response from the counterparty VASP. An exponential backoff is suggested for Command re-sends.

Upon receipt of a Command that has already been processed (resulting in a success response or a Command error), the receiving side must reply with the same response as was previously issued (successful commands, or commands that fail with a command error, are idempotent). Requests that resulted in protocol errors may result in different responses.

For example:
* VASP A sends a request to VASP B which failed with an unknown network error. VASP A retries the send request to VASP B.
* VASP B receives both requests, and processes them in parallel.
* Both requests attempt to acquire a common lock. The first request that VASP A believes failed acquires the lock and the second request fails.
* VASP B successfully processes the first request and sends back a successful response, however, VASP A doesn’t receive the response due to the previous errors.
* VASP B rejects the second request due to lock contention.
* VASP A receives the failure for the second request, and later, retries the same request. If VASP B processes the request when the lock is not under contention it will respond successfully.

In order to facilitate idempotent functionality, the requesting participant should use the same `cid` on all retry requests, so that the receiving participant can distinguish retries and provide idempotency where appropriate. Upon a `cid` match, the receiving participant should also verify that the `CommandRequestObject` is identical to the previous command.

## JWS scheme

All `CommandRequestObject` and `CommandResponseObject` messages exchanged on the off-chain channel between two services must be signed using a specific configuration of the JWS scheme.

The JSON Web Signature (JWS) scheme is specified in [RFC 7515](https://tools.ietf.org/html/rfc7515). Messages are signed with the following parameters:

* The JWS Signature Scheme is EdDSA as specified in [RFC 8032 (EdDSA)](https://tools.ietf.org/html/rfc8032) and [RFC 8037 (Elliptic Curve signatures for JWS)](https://tools.ietf.org/html/rfc8037).
* The JWS Serialization scheme is Compact as specified in [Section 3.1 of RFC 7515](https://tools.ietf.org/html/rfc7515#section-3.1)
* The Protected Header should contain the JSON object {"alg": "EdDSA"}, indicating the signature algorithm used.

**Test Vector:**

JWK key:

```
{"crv":"Ed25519","d":"vLtWeB7kt7fcMPlk01GhGmpWYTHYqnGRZUUN72AT1K4","kty":"OKP","x":"vUfj56-5Teu9guEKt9QQqIW1idtJE4YoVirC7IVyYSk"}
```

Corresponding verification key (hex, bytes), as the 32 bytes stored on the Diem blockchain:

`"bd47e3e7afb94debbd82e10ab7d410a885b589db49138628562ac2ec85726129"` (len=64)

Sample payload message to sign (str, utf8):

`"Sample signed payload."` (len=22)

Valid JWS Compact Signature (str, utf8):

`"eyJhbGciOiJFZERTQSJ9.U2FtcGxlIHNpZ25lZCBwYXlsb2FkLg.dZvbycl2Jkl3H7NmQzL6P0_lDEW42s9FrZ8z-hXkLqYyxNq8yOlDjlP9wh3wyop5MU2sIOYvay-laBmpdW6OBQ"` (len=138)
