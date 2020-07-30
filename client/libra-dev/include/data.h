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
#define LIBRA_AUTHKEY_SIZE 32
#define LIBRA_SIGNATURE_SIZE 64
#define LIBRA_ADDRESS_SIZE 16

enum LibraStatus {
    Ok = 0,
    InvalidArgument = -1,
    InternalError = -255,
};

struct LibraP2PTransferTransactionArgument {
    uint64_t value;
    uint8_t address[LIBRA_ADDRESS_SIZE];
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
    uint64_t expiration_timestamp_secs;
    uint8_t chain_id;
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

/*!
 *  Get serialized signed transaction from a list of transaction parameters
 *
 * To get the serialized transaction in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] sender_private_key is sender's private key
 * @param[in] sequence is the sequence number of this transaction corresponding to sender's account.
 * @param[in] max_gas_amount is the maximal total gas specified by wallet to spend for this transaction.
 * @param[in] gas_unit_price is the maximal price can be paid per gas.
 * @param[in] gas_identifier is the identifier of the coin to be used as gas.
 * @param[in] expiration_timestamp_secs is the time this TX remain valid, the format is unix timestamp.
 * @param[in] chain_id is the chain id for this Transaction.
 * @param[in] script_bytes is the script bytes for given transaction.
 * @param[in] script_len is the length of script_bytes array.
 * @param[out] ptr_buf is the pointer that will be filled with the memory address of the transaction allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] ptr_len is the length of the signed transaction memory buffer.
*/
enum LibraStatus libra_SignedTransactionBytes_from(const uint8_t sender_private_key[LIBRA_PRIVKEY_SIZE], uint64_t sequence, uint64_t max_gas_amount, uint64_t gas_unit_price, const char* gas_identifier, uint64_t expiration_time_secs, uint8_t chain_id, const uint8_t *script_bytes, size_t script_len, uint8_t **ptr_buf, size_t *ptr_len);

/*!
 *  Get script bytes for a P2P transaction
 *
 * To get the serialized script in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] receiver is the receiver's address.
 * @param[in] identifier is the identifier of the coin to be sent.
 * @param[in] num_coins is the amount of money to be sent.
 * @param[in] metadata_bytes is the metadata bytes for given transaction.
 * @param[in] metadata_len is the length of metadata_bytes array.
 * @param[in] metadata_signature_bytes is the metadata signature bytes for given transaction.
 * @param[in] metadata_signature_len is the length of metadata_signature_bytes array.
 * @param[out] ptr_buf is the pointer that will be filled with the memory address of the script allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] ptr_len is the length of the script memory buffer.
*/
enum LibraStatus libra_TransactionP2PScript_from(const uint8_t receiver[LIBRA_ADDRESS_SIZE], const char* identifier, uint64_t num_coins, const uint8_t* metadata_bytes, size_t metadata_len, const uint8_t* metadata_signature_bytes, size_t metadata_signature_len, uint8_t **ptr_buf, size_t *ptr_len);

/*!
 *  Get script bytes for add currency to account transaction
 *
 * To get the serialized script in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] identifier is the identifier of the coin to be sent.
 * @param[out] ptr_buf is the pointer that will be filled with the memory address of the script allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] ptr_len is the length of the script memory buffer.
*/
enum LibraStatus libra_TransactionAddCurrencyScript_from(const char* identifier, uint8_t **ptr_buf, size_t *ptr_len);

/*!
 *  Get script bytes for rotating base url of VASP
 *  Encode a program that rotates `vasp_root_addr`'s base URL to `new_url` and compliance public key to `new_key`.
 *
 * To get the serialized script in a memory safe manner, the client needs to pass in a pointer to a pointer to the allocated memory in rust
 * and call free on the memory address with `libra_free_bytes_buffer`.
 * @param[in] new_url_bytes is the bytes of new base URL for the VASP.
 * @param[in] new_url_len is the length of new_key_bytes array.
 * @param[in] new_key_bytes is the array that contains new key for the VASP.
 * @param[out] ptr_buf is the pointer that will be filled with the memory address of the script allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] ptr_len is the length of the script memory buffer.
*/
enum LibraStatus libra_TransactionRotateDualAttestationInfoScript_from(const uint8_t* new_url_bytes, size_t new_url_len, const uint8_t new_key_bytes[LIBRA_PUBKEY_SIZE], uint8_t **ptr_buf, size_t *ptr_len);

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
 * @param[in] receiver is the receiver's address
 * @param[in] sequence is the sequence number of this transaction corresponding to sender's account.
 * @param[in] num_coins is the amount of money to be sent.
 * @param[in] max_gas_amount is the maximal total gas specified by wallet to spend for this transaction.
 * @param[in] gas_unit_price is the maximal price can be paid per gas.
 * @param[in] expiration_time_secs is the time this TX remain valid, the format is unix timestamp.
 * @param[in] chain_id is the chain id for this Transaction.
 * @param[in] metadata_bytes is the metadata bytes for given transaction.
 * @param[in] metadata_len is the length of metadata_bytes array.
 * @param[out] buf is the pointer that will be filled with the memory address of the transaction allocated in rust. User takes ownership of pointer returned by *buf, which needs to be freed using libra_free_bytes_buffer
 * @param[out] len is the length of the raw transaction memory buffer.
*/
enum LibraStatus libra_RawTransactionBytes_from(const uint8_t sender[LIBRA_ADDRESS_SIZE], const uint8_t receiver[LIBRA_ADDRESS_SIZE], uint64_t sequence, uint64_t num_coins, uint64_t max_gas_amount, uint64_t gas_unit_price, uint64_t expiration_time_secs, uint8_t chain_id, const uint8_t* metadata_bytes, size_t metadata_len, const uint8_t* metadata_signature_bytes, size_t metadata_signature_len, uint8_t **buf, size_t *len);

/*!
 * This function takes in private key in bytes and return the associated public key and address
 * @param[in] private_key_bytes is private key in bytes
 * @param[out] out is a pointer to LibraAccountKey struct client passed in by initializing an empty LibraAccountKey struct
*/
enum LibraStatus libra_LibraAccountKey_from(const uint8_t private_key_bytes[LIBRA_PRIVKEY_SIZE], struct LibraAccountKey *out);

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
