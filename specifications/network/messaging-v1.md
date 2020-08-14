# Messaging Protocol (v1)

This document defines the messages and protocols for [LibraNet](spec.md) v1.

## Versioning

The messaging protocol is versioned using the [`MessagingProtocolVersion`](handshake-v1.md#data-structures). This version is then negotiated in the [LibraNet handshake protocol](handshake-v1.md) during connection establishment and upgrade.

## Messages

LibraNet messages are defined below in the form of Rust structs. On the wire, they are encoded using [lcs]. All LibraNet endpoints must be able to handle receiving all of these messages.

```rust
/// Most primitive message type set on the network. Note this can only support up to 127 message
/// types. The first byte in any message is the message type itself starting from 0.
enum NetworkMessage {
    Error(ErrorCode),
    Ping(Nonce),
    Pong(Nonce),
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    DirectSendMsg(DirectSendMsg),
}

/// Unique identifier associated with each application protocol.
#[repr(u8)]
enum ProtocolId {
    ConsensusRpc = 0,
    ConsensusDirectSend = 1,
    MempoolDirectSend = 2,
    StateSynchronizerDirectSend = 3,
    DiscoveryDirectSend = 4,
    HealthCheckerRpc = 5,
    IdentityDirectSend = 6,
    OnchainDiscoveryRpc = 7,
}

/// Enum representing various error codes that can be embedded in NetworkMessage.
enum ErrorCode {
    /// Failed to parse NetworkMessage, the entries are the first two bytes of the message:
    /// NetworkMessage type and possibly the ProtocolId
    ParsingError(u8, u8),
    /// A message was received for a message / protocol that is not supported over this connection:
    /// The NetworkMessage type is encoded as a u8.
    NotSupported(u8, ProtocolId),
}

/// Nonces used by Ping and Pong message types.
struct Nonce(u32);

/// Create alias RequestId for u32.
type RequestId = u32;

/// Create alias Priority for u8.
type Priority = u8;

struct RpcRequest {
    /// `protocol_id` is a variant of the ProtocolId enum.
    protocol_id: ProtocolId,
    /// RequestId for the RPC Request.
    request_id: RequestId,
    /// Request priority in the range 0..=255.
    priority: Priority,
    /// Request payload. This will be parsed by the application-level handler.
    raw_request: Vec<u8>,
}

struct RpcResponse {
    /// RequestId for corresponding request. This is copied as is from the RpcRequest.
    request_id: RequestId,
    /// Response priority in the range 0..=255\. This will likely be same as the priority of
    /// corresponding request.
    priority: Priority,
    /// Response payload.
    raw_response: Vec<u8>,
}

struct DirectSendMsg {
    /// `protocol_id` is a variant of the ProtocolId enum.
    protocol_id: ProtocolId,
    /// Message priority in the range 0..=255.
    priority: Priority,
    /// Message payload.
    raw_msg: Vec<u8>,
}
```

## Protocol: RPC

The RPC protocol starts with the requester sending a `NetworkMessage::RpcRequest` to the responder with a certain `request_id`. The responder sends the response in a message of type `NetworkMessage::RpcResponse` with the same `request_id`.

The `protocol_id` field in the request indicates the application protocol identifier. The response object does not contain this field.

Any application errors in handling should be wrapped in the `RpcResponse` message itself.

## Protocol: DirectSend

The DirectSend protocol provides one-way fire-and-forget-style message delivery. The sender sends the message payload inside a `NetworkMessage::DirectSendMsg`. The `protocol_id` field in `DirectSendMsg` indicates the application protocol identifier.

## Protocol: Ping-Pong

The Ping-Pong protocol is used for checking liveness and measuring latency between two connected end-points. The only content of the `Ping` and `Pong` message is a nonce that is sent by the side sending the `Ping` and should be echo-ed in the `Pong` sent by the other end. These are sent on the wire as messages of type `NetworkMessage::Ping` and `NetworkMessage::Pong`.

## Message Priority

The `RpcRequest` , `RpcResponse` and `DirectSendMsg` structs also have a `priority` field. The message priority is a best-effort signal on how to prioritize (higher means more urgent) the message on both the sending and receiving ends. In case of RPC, the receiver could respect the request priority and attach the same priority value to the outbound response.

Pending inbound and outbound messages MAY be reordered and dropped according to their `priority`, though the LibraNet reference implementation does not currently respect `priority`.

## Errors

Errors are sent as messages of type `NetworkMessage::Error`, with the `ErrorCode` indicating the type of error. For example, if an `RpcRequest` is received for a `ProtocolId` that was not advertised to a node, we send an error message with the code `ErrorCode::NotSupported(3, ProtocolId)`, where 3 represents the index for RpcRequest's in NetworkMessage.

Responding to errors is not required. A message must be of at least length 2 in order to trigger an error response, otherwise an error would have insufficient data to be meaningful.

### Flow control

LibraNet does not define any mechanism or policy for back-pressure/flow-control. Each end-point is free to implement a local policy to safe-guard against chatty neighbors by not issuing TCP window updates.

## Framing

Each serialized LibraNet message is framed by a big-endian encoded `u32` (4-bytes) length prefix. These message frames are then sent over a Noise-wrapped socket (which has its own internal framing, encryption, and decryption). Consequently, a single message frame may span multiple Noise frames. Likewise, a single Noise frame may contain multiple message frames.

The serialized `NetworkMsg`s over-the-wire then look like a sequence of length-prefix + message pairs (ignoring crypto and framing from lower layers):

```
[u32-length-prefix] || [serialized-message-bytes] || ..
```

(TODO(philiphayes): add streaming RPC protocol when supported)
(TODO(philiphayes): currently we can't send or handle `Error`, `Ping`, or `Pong` messages...)
