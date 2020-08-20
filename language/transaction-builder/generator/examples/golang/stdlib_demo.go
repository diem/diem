// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

package main

import (
	"fmt"
	"github.com/facebookincubator/serde-reflection/serde-generate/runtime/golang/lcs"
	stdlib "testing/librastdlib"
	libra "testing/libratypes"
)

func main() {
	token := &libra.TypeTag__Struct{
		Value: libra.StructTag{
			Address: libra.AccountAddress{
				Value: [16]uint8{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
			},
			Module:     libra.Identifier{Value: "LBR"},
			Name:       libra.Identifier{Value: "LBR"},
			TypeParams: []libra.TypeTag{},
		},
	}
	payee := libra.AccountAddress{
		Value: [16]uint8{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22},
	}
	amount := uint64(1_234_567)
	script := stdlib.EncodePeerToPeerWithMetadataScript(token, payee, amount, []uint8{}, []uint8{})

	call, err := stdlib.DecodeScript(&script)
	if err != nil {
		panic(fmt.Sprintf("failed to decode script: %v", err))
	}
	payment := call.(*stdlib.ScriptCall__PeerToPeerWithMetadata)
	if payment.Amount != amount || payment.Payee != payee {
		panic("wrong script content")
	}

	serializer := lcs.NewSerializer()
	if err := script.Serialize(serializer); err != nil {
		panic("failed to serialize")
	}
	bytes := serializer.GetBytes()
	for _, b := range(bytes) {
		fmt.Printf("%d ", b)
	}
	fmt.Printf("\n")
}
