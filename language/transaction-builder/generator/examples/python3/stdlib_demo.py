# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# pyre-strict

import libra_types as libra
import serde_types as st
import libra_stdlib as stdlib


def make_address(content: bytes) -> libra.AccountAddress:
    assert len(content) == 16
    # pyre-fixme
    return libra.AccountAddress(value=tuple(st.uint8(x) for x in content))


def main() -> None:
    token = libra.TypeTag__Struct(
        value=libra.StructTag(
            address=make_address(b"\x00" * 15 + b"\x01"),
            module=libra.Identifier("LBR"),
            name=libra.Identifier("LBR"),
            type_params=[],
        )
    )
    payee = make_address(b"\x22" * 16)
    amount = st.uint64(1_234_567)
    script = stdlib.encode_peer_to_peer_with_metadata_script(token, payee, amount, b"", b"")

    call = stdlib.decode_script(script)
    assert isinstance(call, stdlib.ScriptCall__PeerToPeerWithMetadata)
    assert call.amount == amount;
    assert call.payee == payee;

    for b in script.lcs_serialize():
        print("%d " % b, end='')
    print()


if __name__ == "__main__":
    main()
