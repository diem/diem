// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
