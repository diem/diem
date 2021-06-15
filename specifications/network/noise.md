# Noise Layer

Communication between [DiemNet](README.md) peers are encrypted and authenticated via the [Noise protocol framework](https://noiseprotocol.org/noise.html).
This specification documents how we make use of Noise to encrypt communications in three types of network:

1. **Public Full Node Network (PFN)**. Full nodes can connect to validator-operated full nodes.
2. **Validator Full Node Network (VFN)**. Validator-operated full nodes can connect to their own validator.
3. **Validator Network (VN)**. Validators can connect to other validators.

## NetworkAddress Protocol

The Noise protocol is "pre-negotiated" in peers' advertised or configured [`NetworkAddress`](network-address.md)es. Canonical DiemNet addresses will include the following `Protocol` after the base transport `Protocol`s:

```
human-readable format: "/ln-noise-ik/<x25519-public-key>"
```

where `<x25519-public-key>` is the advertising peer's remote static public key in Noise terminology.

## Depedendencies

For context, we use the following functions defined in the [noise specification](https://noiseprotocol.org/noise.html):

* `Initialize(handshake_pattern, initiator, prologue, s, e, rs, re)`
* `WriteMessage(payload, message_buffer)`
* `ReadMessage(message, payload_buffer)`
* `EncryptWithAd(ad, plaintext)`
* `DecryptWithAd(ad, ciphertext)`

Specifically, we use the noise IK handshake pattern:

```
IK:
  <- s
  ...
  -> e, es, s, ss
  <- e, ee, se
```

This handshake pattern is a one-round trip protocol where:

* the client knows the server's static public key in advance
* the client sends its static public key as part of the handshake

The protocol has been formally verified in [noiseexplorer](https://noiseexplorer.com/patterns/IK/).

We also use Noise with:

* the [X25519](https://tools.ietf.org/html/rfc7748) key exchange
* the [AES-GCM](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf) authenticated encryption algorithm
* the [SHA-256](https://www.nist.gov/publications/secure-hash-standard) hash function

## Constants

The following constants are known to an implementation:

* **`PROTOCOL_NAME = "Noise_IK_25519_AESGCM_SHA256"`**. The protocol name that we use with Noise.
* **`HANDSHAKE_MSG_1 = 32 + (32 + 16) + (8 + 16)`**. The size of the first handshake message which includes a public key, an encrypted public key, and an encrypted 8-byte payload.
* **`HANDSHAKE_MSG_2 = 32 + 16`**. The size of the second handshake message which contains a public key and an encrypted 0-byte payload.

## Peer State

A peer maintains a few variables:

* **`peer_id`**. The peer's id, a 16-byte value which is either:

  * in the VN, the account address of the peer
  * in other networks, the last 16-byte of the peer's public key in case the peer does not have an account address

* **`private_key`**. The peer's X25519 private key, a 32-byte value.
* **`public_key`**. The peer's X25519 public key, a 32-byte value.

A validator maintains the follow additional variables for the VN:

* **`trusted_peers`**. A mapping of peer ids to public keys which represents the current validator set.
* **`timestamps`**. A mapping of peer ids to the last 8-byte timestamp seen from them. This value can be treated as a stateless and strictly increasing counter to avoid replay attacks (see [security considerations](noise.md#security-considerations) section).

Note that no client authentication is done in the VFN as it is currently a private network.

## Handshake

Our noise wrapper exposes two functions to create a noise session with a peer by dialing or listening to a socket:

* **`upgrade_outbound(remote_public_key)`**

  * Send the `prologue` in clear as `peer_id` followed by `remote_public_key` to the server.
  * Call noise's `Initialize(PROTOCOL_NAME, true, prologue, private_key, null, remote_public_key, null)`.
  * Create an 8-byte `payload` of the current epoch unix time in milliseconds.
  * Call noise's `WriteMessage(payload, message_buffer)` with the created `payload`.
  * Send the created `message_buffer` to the server.
  * Receive the `server_response` of size `HANDSHAKE_MSG_2` bytes.
  * Call noise's `ReadMessage(server_response, null)` and return the two noise `CipherState`s obtained. Refer to the post-handshake session on how to use these.

* **`upgrade_inbound()`**

  * Receive the client's `prologue`, it should contain a 32-byte `initiator_peer_id` followed with a 32-byte `responder_expected_public_key`.
  * Verify that the received `initiator_peer_id` is either:

    * if we are in the VN or VFN: in the trusted peer set
    * if we are in the PFN: derived correctly form the public key

  * Verify that the received `responder_expected_public_key` is our public key.
  * Receive the client's `client_message` of size `HANDSHAKE_MSG_1` bytes, it should be of size large enough to contain a payload of 8-byte.
  * Call noise's `Initialize(PROTOCOL_NAME, true, prologue, private_key, null, remote_public_key, null)`.
  * Call noise's `ReadMessage(client_message, payload_buffer)`.
  * If in the VN:

    * enforce that the `initiator_public_key` received in the noise handshake message is in our `trusted_peers` set under the `initiator_peer_id`.
    * enforce that the `payload` is greater than any `timestamp` received so far for that `initiator_peer_id`.
    * store the `payload` as the last `timestamp` seen for that `initiator_peer_id`.

  * If in the PFN and VFN, enforce that the `initiator_peer_id` is correctly derived from the `initiator_public_key`.
  * call noise's `WriteMessage(null, message_buffer)` and store the two noise `CipherState`s obtained.
  * Send the constructed `message_buffer` to the server.
  * Return the two noise `CipherState`s. Refer to the post-handshake session on how to use these.

## Post-handshake

Once two `CipherState`s from noise have been obtained, we define the two wrapper functions:

* **`encrypt(message)`**

  * Call noise's `EncryptWithAd(null, message)` with the first `CipherState` to construct a `ciphertext`.
  * Send the length of the `ciphertext` as a 2-byte value to the peer.
  * Send the `ciphertext` to the peer.

* **`decrypt(message)`**

  * Receive 2 bytes from the peer, and interpret that as a `length`.
  * Receive `length` bytes from the peer as `ciphertext`.
  * Call noise's `DecryptWithAd(null, ciphertext)` and return the result.

## Security Considerations

### Replay Attacks

In the mutually-authenticated VN, an man-in-the-middle observer can in theory replay the first handshake message in order to:

1. evict a legitimate on-going connection
2. force the server to compute useless cryptographic operations

As noise's IK handshake pattern does not provide [key confirmation](https://csrc.nist.gov/glossary/term/Key_confirmation), it is necessary to prevent the first attack to wait for another client message before confirming the connection.

To prevent the second attack, we add a counter in the client's first noise message payload.
A replay will be detected if the counter is not striclty increasing.
In order to avoid keeping track of this counter on the client side we use an 8-byte unix timestamp, and in order to avoid connection issues from preventing a client to connect we set the precision to milliseconds.
Thanks to this countermeasure, a server can halve (from 4 to 2) the number of Diffie-Hellman key exchanges they have to perform when detecting a replay.

Note that if a validator crashes, they will lose track of timestamps seen from clients.
This would allow an attacker to replay all handshakes from a client in order.
But this would not allow the attacker to prevent valid connection attempts.

We do not protect against this in the FN since an attacker can just create as many valid handshakes as they want.

## Rekey

We currently do not rekey session.
We thus have long-lived sessions without forward and backward secrecy.
This is currently not foreseen to be an issue as no critically confidential data is exchanged between validators, and important messages are further signed on the application layer.

## Payload security property

From section [Payload security properties](https://noiseprotocol.org/noise.html#payload-security-properties) of the noise specification, we observe that the sender's authentication is vulnerable to KCI: if the server's key is compromised, the attacker can impersonate anyone to the server.

We accept the risk.

## Identity hiding

Section [Identity hiding](https://noiseprotocol.org/noise.html#identity-hiding) of the noise specification outlines identity hiding risks, which we accept as well as the identities of our peers are not private.
