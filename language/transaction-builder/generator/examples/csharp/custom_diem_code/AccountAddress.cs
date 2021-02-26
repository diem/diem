// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

const int LENGTH = 16;

public static AccountAddress valueOf(byte[] values) {
    if (values.Length != LENGTH) {
        throw new ArgumentException("Invalid length for AccountAddress");
    }
    List<byte> address = new List<byte>(LENGTH);
    for (int i = 0; i < LENGTH; i++) {
        address.Add(values[i]);
    }
    return new AccountAddress(new Serde.ValueArray<byte>(address.ToArray()));
}

public byte[] toBytes() {
    byte[] bytes = new byte[LENGTH];
    int i = 0;
    foreach (byte item in value) {
        bytes[i++] = item;
    }
    return bytes;
}
