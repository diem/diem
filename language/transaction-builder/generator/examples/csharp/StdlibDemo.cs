using System;
using System.Collections.Generic;
using Diem.Stdlib;
using Diem.Types;
using Serde;
using System.Diagnostics;

public class StdlibDemo {
    public static void Main() {
        DemoP2PScript();
        DemoP2PScriptFunction();
    }

    static void DemoP2PScript() {
        StructTag tag = new StructTag(
            AccountAddress.valueOf(new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1}),
            new Identifier("XDX"),
            new Identifier("XDX"),
            new ValueArray<TypeTag>(new List<TypeTag>().ToArray())
        );

        TypeTag token = new TypeTag.Struct(tag);

        AccountAddress payee = AccountAddress.valueOf(
            new byte[]{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22});

        ulong amount = 1234567;
        Script script =
            Helpers.encode_peer_to_peer_with_metadata_script(
                token,
                payee,
                amount,
                new ValueArray<byte>(new List<byte>().ToArray()),
                new ValueArray<byte>(new List<byte>().ToArray())
        );

        ScriptCall.PeerToPeerWithMetadata call = (ScriptCall.PeerToPeerWithMetadata)Helpers.DecodeScript(script);
        Debug.Assert(call.amount.Equals(amount), string.Format("call.amount is {0}. Expecting {1}", call.amount, amount));
        Debug.Assert(call.payee.Equals(payee), string.Format("call.payee is {0}. Expecting {1}", call.payee, payee));

        byte[] output = script.BcsSerialize();
        foreach (byte o in output) {
            Console.Write(((int) o & 0xFF) + " ");
        }
        Console.WriteLine();
    }

    static void DemoP2PScriptFunction() {
        StructTag tag = new StructTag(
            AccountAddress.valueOf(new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1}),
            new Identifier("XDX"),
            new Identifier("XDX"),
            new ValueArray<TypeTag>(new List<TypeTag>().ToArray())
        );

        TypeTag token = new TypeTag.Struct(tag);

        AccountAddress payee = AccountAddress.valueOf(
            new byte[]{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22});

        ulong amount = 1234567;
        TransactionPayload payload =
            Helpers.encode_peer_to_peer_with_metadata_script_function(
                token,
                payee,
                amount,
                new ValueArray<byte>(new List<byte>().ToArray()),
                new ValueArray<byte>(new List<byte>().ToArray())
        );

        ScriptFunctionCall.PeerToPeerWithMetadata call = (ScriptFunctionCall.PeerToPeerWithMetadata)Helpers.DecodeScriptFunctionPayload(payload);
        Debug.Assert(call.amount.Equals(amount), string.Format("call.amount is {0}. Expecting {1}", call.amount, amount));
        Debug.Assert(call.payee.Equals(payee), string.Format("call.payee is {0}. Expecting {1}", call.payee, payee));

        byte[] output = payload.BcsSerialize();
        foreach (byte o in output) {
            Console.Write(((int) o & 0xFF) + " ");
        }
        Console.WriteLine();
    }
}
