// Copyright (c) 2018 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <bench/bench.h>

#include <bech32.h>
#include <util/strencodings.h>

#include <vector>
#include <string>


static void Bech32Encode(benchmark::State& state)
{
    std::vector<uint8_t> v = ParseHex("c97f5a67ec381b760aeaf67573bc164845ff39a3bb26a1cee401ac67243b48db");
    std::vector<unsigned char> tmp = {0};
    tmp.reserve(1 + 32 * 8 / 5);
    ConvertBits<8, 5, true>([&](unsigned char c) { tmp.push_back(c); }, v.begin(), v.end());
    while (state.KeepRunning()) {
        bech32::Encode("bc", tmp);
    }
}


static void Bech32Decode(benchmark::State& state)
{
    std::string addr = "bc1qkallence7tjawwvy0dwt4twc62qjgaw8f4vlhyd006d99f09";
    while (state.KeepRunning()) {
        bech32::Decode(addr);
    }
}


BENCHMARK(Bech32Encode, 800 * 1000);
BENCHMARK(Bech32Decode, 800 * 1000);
