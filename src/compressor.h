// Copyright (c) 2009-2010 Satoshi Nakamoto
// Copyright (c) 2009-2018 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef BITCOIN_COMPRESSOR_H
#define BITCOIN_COMPRESSOR_H

#include <primitives/transaction.h>
#include <script/script.h>
#include <serialize.h>
#include <span.h>

class CKeyID;
class CPubKey;
class CScriptID;

bool CompressScript(const CScript& script, std::vector<unsigned char> &out);
unsigned int GetSpecialScriptSize(unsigned int nSize);
bool DecompressScript(CScript& script, unsigned int nSize, const std::vector<unsigned char> &out);

uint64_t CompressAmount(uint64_t nAmount);
uint64_t DecompressAmount(uint64_t nAmount);

/** Compact serializer for scripts.
 *
 *  It detects common cases and encodes them much more efficiently.
 *  3 special cases are defined:
 *  * Pay to pubkey hash (encoded as 21 bytes)
 *  * Pay to script hash (encoded as 21 bytes)
 *  * Pay to pubkey starting with 0x02, 0x03 or 0x04 (encoded as 33 bytes)
 *
 *  Other scripts up to 121 bytes require 1 byte + script length. Above
 *  that, scripts up to 16505 bytes require 2 bytes + script length.
 */
class CScriptCompressor
{
private:
    /**
     * make this static for now (there are only 6 special scripts defined)
     * this can potentially be extended together with a new nVersion for
     * transactions, in which case this value becomes dependent on nVersion
     * and nHeight of the enclosing transaction.
     */
    static const unsigned int nSpecialScripts = 6;

    CScript &script;
public:
    explicit CScriptCompressor(CScript &scriptIn) : script(scriptIn) { }

    template<typename Stream>
    void Serialize(Stream &s) const {
        std::vector<unsigned char> compr;
        if (CompressScript(script, compr)) {
            s << MakeSpan(compr);
            return;
        }
        unsigned int nSize = script.size() + nSpecialScripts;
        s << VARINT(nSize);
        s << MakeSpan(script);
    }

    template<typename Stream>
    void Unserialize(Stream &s) {
        unsigned int nSize = 0;
        s >> VARINT(nSize);
        if (nSize < nSpecialScripts) {
            std::vector<unsigned char> vch(GetSpecialScriptSize(nSize), 0x00);
            s >> MakeSpan(vch);
            DecompressScript(script, nSize, vch);
            return;
        }
        nSize -= nSpecialScripts;
        if (nSize > MAX_SCRIPT_SIZE) {
            // Overly long script, replace with a short invalid one
            script << OP_RETURN;
            s.ignore(nSize);
        } else {
            script.resize(nSize);
            s >> MakeSpan(script);
        }
    }
};

/** wrapper for CTxOut that provides a more compact serialization */
class CTxOutCompressor
{
private:
    CTxOut &txout;

public:
    explicit CTxOutCompressor(CTxOut &txoutIn) : txout(txoutIn) { }

    ADD_SERIALIZE_METHODS;

    template <typename Stream, typename Operation>
    inline void SerializationOp(Stream& s, Operation ser_action) {
        if (!ser_action.ForRead()) {
            uint64_t nVal = CompressAmount(txout.nValue);
            READWRITE(VARINT(nVal));
        } else {
            uint64_t nVal = 0;
            READWRITE(VARINT(nVal));
            txout.nValue = DecompressAmount(nVal);
        }
        CScriptCompressor cscript(REF(txout.scriptPubKey));
        READWRITE(cscript);
    }
};

#endif // BITCOIN_COMPRESSOR_H
