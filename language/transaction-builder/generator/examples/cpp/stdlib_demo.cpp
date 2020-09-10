// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "libra_stdlib.hpp"
#include "libra_types.hpp"
#include <memory>

using namespace libra_stdlib;
using namespace libra_types;

int main() {
    auto token = TypeTag{TypeTag::Struct{StructTag{
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
        {"LBR"},
        {"LBR"},
        {},
    }}};
    auto payee = AccountAddress{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                                0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22};
    uint64_t amount = 1234567;
    auto script =
        encode_peer_to_peer_with_metadata_script(token, payee, amount, {}, {});

    auto output = script.lcsSerialize();
    for (uint8_t o : output) {
        printf("%d ", o);
    };
    printf("\n");
    return 0;
}
