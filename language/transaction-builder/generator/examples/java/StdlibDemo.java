// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

import java.util.Arrays;
import java.util.ArrayList;

import com.novi.serde.Bytes;
import com.novi.serde.Unsigned; // used as documentation.
import org.libra.stdlib.Helpers;
import org.libra.stdlib.ScriptCall;;
import org.libra.types.AccountAddress;
import org.libra.types.Identifier;
import org.libra.types.Script;
import org.libra.types.StructTag;
import org.libra.types.TypeTag;

public class StdlibDemo {

    static AccountAddress make_address(byte[] values) {
        assert values.length == 16;
        Byte[] address = new Byte[16];
        for (int i = 0; i < 16; i++) {
            address[i] = Byte.valueOf(values[i]);
        }
        return new AccountAddress(address);
    }

    public static void main(String[] args) throws Exception {
        StructTag.Builder builder = new StructTag.Builder();
        builder.address = make_address(new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1});
        builder.module = new Identifier("LBR");
        builder.name = new Identifier("LBR");
        builder.type_params = new ArrayList<org.libra.types.TypeTag>();
        StructTag tag = builder.build();

        TypeTag token = new TypeTag.Struct(tag);

        AccountAddress payee = make_address(
            new byte[]{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22});

        @Unsigned Long amount = Long.valueOf(1234567);
        Script script =
            Helpers.encode_peer_to_peer_with_metadata_script(token, payee, amount, new Bytes(new byte[]{}), new Bytes(new byte[]{}));

        ScriptCall.PeerToPeerWithMetadata call = (ScriptCall.PeerToPeerWithMetadata)Helpers.decode_script(script);
        assert(call.amount.equals(amount));
        assert(call.payee.equals(payee));

        byte[] output = script.lcsSerialize();
        for (byte o : output) {
            System.out.print(((int) o & 0xFF) + " ");
        };
        System.out.println();
    }

}
