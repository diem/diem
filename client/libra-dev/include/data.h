#ifndef LIBRA_DEV_H
#define LIBRA_DEV_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define LIBRA_PUBKEY_SIZE 32
#define LIBRA_PRIVKEY_SIZE 32
#define LIBRA_SIGNATURE_SIZE 64
#define LIBRA_ADDRESS_SIZE 16
#define LIBRA_EVENT_KEY_SIZE 24

enum LibraStatus {
    Ok = 0,
    InvalidArgument = -1,
    InternalError = -255,
};

struct LibraEventHandle {
    uint64_t count;
    uint8_t key[LIBRA_EVENT_KEY_SIZE];
};

struct LibraAccountResource {
    uint64_t balance;
    uint64_t sequence;
    uint8_t authentication_key[LIBRA_PUBKEY_SIZE];
    bool delegated_key_rotation_capability;
    bool delegated_withdrawal_capability;
    struct LibraEventHandle sent_events;
    struct LibraEventHandle received_events;
};

struct LibraP2PTransferTransactionArgument {
    uint64_t value;
    uint8_t address[LIBRA_ADDRESS_SIZE];
    uint8_t auth_key_prefix[LIBRA_PUBKEY_SIZE - LIBRA_ADDRESS_SIZE];
    const uint8_t* metadata_bytes;
    size_t metadata_len;
    const uint8_t* metadata_signature_bytes;
    size_t metadata_signature_len;
};

enum TransactionType {
    PeerToPeer = 0,
    Mint = 1,
    Unknown = -1,
};

struct LibraTransactionPayload {
    enum TransactionType txn_type;
    struct LibraP2PTransferTransactionArgument args;
};

struct LibraRawTransaction {
    uint8_t sender[LIBRA_ADDRESS_SIZE];
    uint64_t sequence_number;
    struct LibraTransactionPayload payload;
    uint64_t max_gas_amount;
    uint64_t gas_unit_price;
    uint64_t expiration_time_secs;
};

struct LibraSignedTransaction {
    struct LibraRawTransaction raw_txn;
    uint8_t public_key[LIBRA_PUBKEY_SIZE];
    uint8_t signature[LIBRA_SIGNATURE_SIZE];
};

struct LibraAccountKey {
    uint8_t address[LIBRA_ADDRESS_SIZE];
    uint8_t private_key[LIBRA_PRIVKEY_SIZE];
    uint8_t public_key[LIBRA_PUBKEY_SIZE];
};

enum LibraEventType {
    SentPaymentEvent = 1,
    ReceivedPaymentEvent = 2,
    UndefinedEvent = -1,
};

struct LibraPaymentEvent {
    uint8_t sender_address[LIBRA_ADDRESS_SIZE];
    uint8_t receiver_address[LIBRA_ADDRESS_SIZE];
    uint64_t amount;
    uint8_t* metadata;
    size_t metadata_len;
};

struct LibraEvent {
    enum LibraEventType event_type;
    // TODO: address
    uint8_t module[255];
    uint8_t name[255];
    // TODO: type_params
    struct LibraPaymentEvent payment_event_data;
    // TODO: other type of event_data
};

/*!
 * Decode LibraAccountResource from bytes in AccountStateBlob.
 *
 * @param[in] buf contains encoded bytes of AccountStateBlob
 * @param[in] len is the length of the signed AccountStateBlob memory buffer.
 * @param[out] caller allocated LibraAccountResource to write into.
 *
 * @returns status code, one of LibraAPIStatus
*/
enum LibraStatus libra_LibraAccountResource_from(const uint8_t *buf, size_t len, struct LibraAccountResource *out);

/*!
 *  Get serialized signed transaction from a list of transaction parameters
 *
 * To get the serialized transaction in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] sender_private_key is sender's private key
 * @param[in] receiver is the receiver's authentication key.
 * @param[in] sequence is the sequence number of this transaction corresponding to sender's account.
 * @param[in] num_coins is the amount of money to be sent.
 * @param[in] max_gas_amount is the maximal total gas specified by wallet to spend for this transaction.
 * @param[in] gas_unit_price is the maximal price can be paid per gas.
 * @param[in] expiration_time_secs is the time this TX remain valid, the format is unix timestamp.
 * @param[in] metadata_bytes is the metadata bytes for given transaction.
 * @param[in] metadata_len is the length of metadata_bytes array.
 * @param[out] ptr_buf is the pointer that will be filled with the memory address of the transaction allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] ptr_len is the length of the signed transaction memory buffer.
*/
enum LibraStatus libra_SignedTransactionBytes_from(const uint8_t sender_private_key[LIBRA_PRIVKEY_SIZE], const uint8_t receiver[LIBRA_PUBKEY_SIZE], uint64_t sequence, uint64_t num_coins, uint64_t max_gas_amount, uint64_t gas_unit_price, uint64_t expiration_time_secs, const uint8_t* metadata_bytes, size_t metadata_len,  const uint8_t* metadata_signature_bytes, size_t metadata_signature_len, uint8_t **ptr_buf, size_t *ptr_len);

/*!
 * Function to free the allocation memory in rust for bytes
 * @param buf is the pointer to the bytes allocated in rust, and needs to be freed from client side
 */
void libra_free_bytes_buffer(const uint8_t* buf);

/*!
 * Decode LibraSignedTransaction from bytes in SignedTransaction proto.
 *
 * @param[in] buf contains encoded bytes of txn_bytes
 * @param[in] len is the length of the signed transaction memory buffer.
 * @param[out] caller allocated LibraSignedTransaction to write into.
 *
 * @returns status code, one of LibraAPIStatus
*/
enum LibraStatus libra_LibraSignedTransaction_from(const uint8_t *buf, size_t len, struct LibraSignedTransaction *out);

/*!
 * Get serialized raw transaction from a list of transaction parameters
 *
 * To get the serialized raw transaction in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] sender is the sender's address
 * @param[in] receiver is the receiver's authentication key.
 * @param[in] sequence is the sequence number of this transaction corresponding to sender's account.
 * @param[in] num_coins is the amount of money to be sent.
 * @param[in] max_gas_amount is the maximal total gas specified by wallet to spend for this transaction.
 * @param[in] gas_unit_price is the maximal price can be paid per gas.
 * @param[in] expiration_time_secs is the time this TX remain valid, the format is unix timestamp.
 * @param[in] metadata_bytes is the metadata bytes for given transaction.
 * @param[in] metadata_len is the length of metadata_bytes array.
 * @param[out] buf is the pointer that will be filled with the memory address of the transaction allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] len is the length of the raw transaction memory buffer.
*/
enum LibraStatus libra_RawTransactionBytes_from(const uint8_t sender[LIBRA_ADDRESS_SIZE], const uint8_t receiver[LIBRA_PUBKEY_SIZE], uint64_t sequence, uint64_t num_coins, uint64_t max_gas_amount, uint64_t gas_unit_price, uint64_t expiration_time_secs, const uint8_t* metadata_bytes, size_t metadata_len, const uint8_t* metadata_signature_bytes, size_t metadata_signature_len, uint8_t **buf, size_t *len);

/*!
 * This function takes in a raw transaction, public key and signature in bytes, and return a signed transaction in bytes.
 * To get the serialized signed transaction in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] buf_raw_txn is raw transaction in bytes
 * @param[in] len_raw_txn is length of raw transaction bytes
 * @param[in] buf_public_key is public key in bytes
 * @param[in] len_public_key is length of public key bytes
 * @param[in] buf_signature is signature in bytes
 * @param[in] len_signature is length of signature bytes
 * @param[out] buf_result is the pointer that will be filled with the memory address of the transaction allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] len_result is the length of the signed transaction memory buffer.
*/
enum LibraStatus libra_RawTransaction_sign(const uint8_t *buf_raw_txn, size_t len_raw_txn, const uint8_t *buf_public_key, size_t len_public_key, const uint8_t *buf_signature, size_t len_signature, uint8_t **buf_result, size_t *len_result);

/*!
 * This function takes in private key in bytes and return the associated public key and address
 * @param[in] private_key_bytes is private key in bytes
 * @param[out] out is a pointer to LibraAccountKey struct client passed in by initializing an empty LibraAccountKey struct
*/
enum LibraStatus libra_LibraAccountKey_from(const uint8_t private_key_bytes[LIBRA_PRIVKEY_SIZE], struct LibraAccountKey *out);

/*!
 * This function takes in an event key, event data and event type tag in bytes, and return LibraEvent.
 * To get the event in a memory safe manner, the client needs to call free on the output with `libra_LibraEvent_free`.
 * @param[in] buf_key is event key in bytes
 * @param[in] len_key is length of event key bytes
 * @param[in] buf_data is event data in bytes
 * @param[in] len_data is length of event data bytes
 * @param[in] buf_type_tag is type tag in bytes
 * @param[in] len_type_tag is length of type tag bytes
 * @param[out] rust allocated LibraEvent to write into
 * @returns status code, one of LibraStatus
*/
enum LibraStatus libra_LibraEvent_from(const uint8_t *buf_key, size_t len_key, const uint8_t *buf_data, size_t len_data, const uint8_t *buf_type_tag, size_t len_type_tag, struct LibraEvent **out);

/*!
 * Function to free the allocation event memory in rust
 * @param out is the pointer to the event struct allocated, and needs to be freed from client side
 */
void libra_LibraEvent_free(struct LibraEvent *out);

/*!
 * This function returns the string message of the most recent error in Rust.
 * Error will be in UTF8 string encoding and client does not need to free the string from their side.
 * @returns error message string
*/
const char *libra_strerror();

#ifdef __cplusplus
};
#endif

#endif // LIBRA_DEV_H
