# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# pyre-strict

import diem_types as diem
import serde_types as st
import diem_stdlib as stdlib


def main() -> None:
    token = diem.TypeTag__Struct(
        value=diem.StructTag(
            address=diem.AccountAddress.from_bytes(b"\x00" * 15 + b"\x01"),
            module=diem.Identifier("XDX"),
            name=diem.Identifier("XDX"),
            type_params=[],
        )
    )
    payee = diem.AccountAddress.from_bytes(b"\x22" * 16)
    amount = st.uint64(1_234_567)
    script = stdlib.encode_peer_to_peer_with_metadata_script(token, payee, amount, b"", b"")

    call = stdlib.decode_script(script)
    assert isinstance(call, stdlib.ScriptCall__PeerToPeerWithMetadata)
    assert call.amount == amount;
    assert call.payee == payee;

    for b in script.bcs_serialize():
        print("%d " % b, end='')
    print()


if __name__ == "__main__":
    main()
