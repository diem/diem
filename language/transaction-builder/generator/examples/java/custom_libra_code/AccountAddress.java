// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

static final int LENGTH = 16;

public static AccountAddress valueOf(byte[] values) {
    if (values.length != LENGTH) {
        throw new java.lang.IllegalArgumentException("Invalid length for AccountAddress");
    }
    Byte[] address = new Byte[LENGTH];
    for (int i = 0; i < LENGTH; i++) {
        address[i] = Byte.valueOf(values[i]);
    }
    return new AccountAddress(address);
}

public byte[] toBytes() {
    byte[] bytes = new byte[LENGTH];
    for (int i = 0; i < LENGTH; i++) {
        bytes[i] = value[i].byteValue();
    }
    return bytes;
}
