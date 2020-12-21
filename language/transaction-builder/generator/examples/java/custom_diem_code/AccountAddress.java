// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

import java.util.ArrayList;
import java.util.List;

static final int LENGTH = 16;

public static AccountAddress valueOf(byte[] values) {
    if (values.length != LENGTH) {
        throw new java.lang.IllegalArgumentException("Invalid length for AccountAddress");
    }
    java.util.List<Byte> address = new java.util.ArrayList<Byte>(LENGTH);
    for (int i = 0; i < LENGTH; i++) {
        address.add(Byte.valueOf(values[i]));
    }
    return new AccountAddress(address);
}

public byte[] toBytes() {
    byte[] bytes = new byte[LENGTH];
    int i = 0;
    for (Byte item : value) {
        bytes[i++] = item.byteValue();
    }
    return bytes;
}

/**
 * Create AccountAddress from given bytes.
 *
 * @param bytes address bytes
 * @return AccountAddress
 * @throws IllegalArgumentException if given bytes array length is not equal to `LENGTH`
 */
public static AccountAddress create(byte[] bytes) {
    if (bytes.length != LENGTH) {
        throw new IllegalArgumentException(String.format("account address bytes length must be {}", LENGTH));
    }
    List<Byte> address = new ArrayList<Byte>();
    for (int i = 0; i < LENGTH; i++) {
        address.add(Byte.valueOf(bytes[i]));
    }
    return new AccountAddress(address);
}

/**
 * Create AccountAddress from given hex-encoded bytes string
 *
 * @param address hex-encoded bytes string
 * @return AccountAddress
 */
public static AccountAddress create(String address) {
    return create(Hex.decode(address));
}

/**
 * Convert given AccountAddress into hex-encoded bytes string.
 *
 * @param address
 * @return hex-encoded bytes string
 */
public static String hex(AccountAddress address) {
    return Hex.encode(address.value);
}

/**
 * @param address
 * @return byte array of the address
 */
public static byte[] bytes(AccountAddress address) {
    byte[] ret = new byte[LENGTH];
    for (int i = 0; i < LENGTH; i++) {
        ret[i] = address.value.get(i);
    }
    return ret;
}
