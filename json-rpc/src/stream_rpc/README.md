# Streaming RPC
The goal of the streaming RPC is to solve the following problems:
- There is significant overhead to making a new connection for each request, [compared to WebSockets or Server Side Events](http://www.diva-portal.se/smash/get/diva2:1133465/FULLTEXT01.pdf)
- Polling is wasteful vs server-side streaming, especially for sparse event streams

Simplified flow:
![Overall stream flow](../../docs/images/stream_rpc_overall_flow.png)

This document assumes the perspective of a fullnode. Any relative terms (i.e `incoming`) are therefore intended to mean `incoming to the fullnode`.

## Transports
Transports handle connections to the outside world.
Transports are responsible for mapping external representations of communication to internal representations, and vice versa.

The currently supported transports are:
- Websockets

Planned transports are:
- Server-Side Events

For example: in the case of WebSockets, incoming messages have the form of `Result<warp::filters::ws::Message, warp::Error>`.
Different transports will have different external message representations, and so the `ConnectionManager` expects a `dyn Stream`
with messages in the form of `Result<Option<String>, anyhow::Error>`.

For outgoing messages, still in the case of WebSockets, a similar process is required.
On the WebSocket channel (`warp::filters::ws::WebSocket`) Warp expects a `warp::filters::ws::Message`- but the `ConnectionManager`
only sends a `Result<String, anyhow::Error>`, which the WebSocket transport must wrap into a `Message::text(outbound_string)`.

This allows the transport to hide the specifics of dealing with a WebSocket connection (`ping`/`pong` messages, `close` messages, etc),
while

If the transport layer decides to pass any kind of `Error<_>` to the `ConnectionManager`, the `ConnectionManager` will
terminate all client subscriptions, and ultimately disconnect the client.
Likewise, if a transport sees an outbound `Error` it should expect (and if possible, initiate) a disconnect.
The transport should not attempt to pass this `Error` along to the client (in all likelihood the client would not be able to receive it).
Errors at this level indicate unrecoverable communication errors.
At this transport level, JSON-RPC errors are indistinguishable from any other message (they are all `String`).



## Connection Manager
The `ConnectionManager` is the internal interface with which transports interact.
Put simply, it accepts `String` messages

For the incoming message stream (client->fullnode), the `ConnectionManager` expects a
`Box<dyn Stream<Item = Result<Option<String>, anyhow::Error>> + Send + Unpin>`.
If a transport sends an `Error`, the `ConnectionManager` will terminate all client subscriptions, and end the connection.

Why the `Option<String>`? It's possible for a message to be handled during the process of mapping, and not have

For outgoing messages (fullnode->client)
Box<dyn Stream<Item = Result<Option<String>, anyhow::Error>> + Send + Unpin>
mpsc::Sender<Result<String, anyhow::Error>>;

1. When a new connection reaches transport, the `ClientConnection`


## Client Connections
The `ClientConnection` represents the subscriptions for a given connection and contains helpers for sending data to .

## Subscriptions
For more details on how the `Subscription` trait works, please see [./subscriptions.rs](./subscriptions.rs)
