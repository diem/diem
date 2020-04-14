// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "serde.hpp"

#include <iostream>

/* ---------- */

struct AccountAddress {
    std::array<uint8_t, 16> value;
};

template <>
template <typename Serializer>
void Serializable<AccountAddress>::serialize(const AccountAddress &obj, Serializer &serializer) {
    Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

struct AccessPath {
    std::unique_ptr<AccountAddress> address;
    std::vector<uint8_t> path;
};

template <>
template <typename Serializer>
void Serializable<AccessPath>::serialize(const AccessPath &obj, Serializer &serializer) {
    Serializable<decltype(obj.address)>::serialize(obj.address, serializer);
    Serializable<decltype(obj.path)>::serialize(obj.path, serializer);
}

/* ---------- */

struct MySerializer {
    void serialize_len(uint32_t len) {
        std::cout << len << "\n";
    }

    void serialize_u32(uint32_t value) {
        std::cout << value << "\n";
    }

    void serialize_u8(uint8_t value) {
        std::cout << (int32_t)value << "\n";
    }
};

int main() {
    AccountAddress address = {{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15}};
    std::vector<uint8_t> path = {1,2,3,4};
    AccessPath value = { std::make_unique<AccountAddress>(address), path };

    MySerializer serializer{};
    Serializable<decltype(value)>::serialize(value, serializer);

    Serializable<decltype(path)>::serialize(path, serializer);
}
