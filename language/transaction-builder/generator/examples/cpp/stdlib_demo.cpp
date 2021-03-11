// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "diem_framework.hpp"
#include "diem_types.hpp"
#include <memory>

using namespace diem_framework;
using namespace diem_types;

void demo_p2p_script() {
    auto token = TypeTag{TypeTag::Struct{StructTag{
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
        {"XDX"},
        {"XDX"},
        {},
    }}};
    auto payee = AccountAddress{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                                0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22};
    uint64_t amount = 1234567;
    auto script =
        encode_peer_to_peer_with_metadata_script(token, payee, amount, {}, {});

    auto output = script.bcsSerialize();
    for (uint8_t o : output) {
        printf("%d ", o);
    };
    printf("\n");
}

void demo_p2p_script_function() {
    auto token = TypeTag{TypeTag::Struct{StructTag{
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
        {"XDX"},
        {"XDX"},
        {},
    }}};
    auto payee = AccountAddress{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                                0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22};
    uint64_t amount = 1234567;
    auto payload =
        encode_peer_to_peer_with_metadata_script_function(token, payee, amount, {}, {});

    auto output = payload.bcsSerialize();
    for (uint8_t o : output) {
        printf("%d ", o);
    };
    printf("\n");
}

int main() {
    demo_p2p_script();
    demo_p2p_script_function();
    return 0;
}
