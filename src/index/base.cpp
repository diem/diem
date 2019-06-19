// Copyright (c) 2017-2018 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <chainparams.h>
#include <index/base.h>
#include <shutdown.h>
#include <tinyformat.h>
#include <ui_interface.h>
#include <util/system.h>
#include <validation.h>
#include <warnings.h>

constexpr char DB_BEST_BLOCK = 'B';

constexpr int64_t SYNC_LOG_INTERVAL = 30; // seconds
constexpr int64_t SYNC_LOCATOR_WRITE_INTERVAL = 30; // seconds

template<typename... Args>
static void FatalError(const char* fmt, const Args&... args)
{
    std::string strMessage = tfm::format(fmt, args...);
    SetMiscWarning(strMessage);
    LogPrintf("*** %s\n", strMessage);
    uiInterface.ThreadSafeMessageBox(
        "Error: A fatal internal error occurred, see debug.log for details",
        "", CClientUIInterface::MSG_ERROR);
    StartShutdown();
}

BaseIndex::DB::DB(const fs::path& path, size_t n_cache_size, bool f_memory, bool f_wipe, bool f_obfuscate) :
    CDBWrapper(path, n_cache_size, f_memory, f_wipe, f_obfuscate)
{}

bool BaseIndex::DB::ReadBestBlock(CBlockLocator& locator) const
{
    bool success = Read(DB_BEST_BLOCK, locator);
    if (!success) {
        locator.SetNull();
    }
    return success;
}

void BaseIndex::DB::WriteBestBlock(CDBBatch& batch, const CBlockLocator& locator)
{
    batch.Write(DB_BEST_BLOCK, locator);
}

BaseIndex::~BaseIndex()
{
    Interrupt();
    Stop();
}

bool BaseIndex::Init()
{
    CBlockLocator locator;
    if (!GetDB().ReadBestBlock(locator)) {
        locator.SetNull();
    }

    LOCK(cs_main);
    if (locator.IsNull()) {
        m_best_block_index = nullptr;
    } else {
        m_best_block_index = FindForkInGlobalIndex(::ChainActive(), locator);
    }
    m_synced = m_best_block_index.load() == ::ChainActive().Tip();
    return true;
}

static const CBlockIndex* NextSyncBlock(const CBlockIndex* pindex_prev) EXCLUSIVE_LOCKS_REQUIRED(cs_main)
{
    AssertLockHeld(cs_main);

    if (!pindex_prev) {
        return ::ChainActive().Genesis();
    }

    const CBlockIndex* pindex = ::ChainActive().Next(pindex_prev);
    if (pindex) {
        return pindex;
    }

    return ::ChainActive().Next(::ChainActive().FindFork(pindex_prev));
}

void BaseIndex::ThreadSync()
{
    const CBlockIndex* pindex = m_best_block_index.load();
    if (!m_synced) {
        auto& consensus_params = Params().GetConsensus();

        int64_t last_log_time = 0;
        int64_t last_locator_write_time = 0;
        while (true) {
            if (m_interrupt) {
                m_best_block_index = pindex;
                // No need to handle errors in Commit. If it fails, the error will be already be
                // logged. The best way to recover is to continue, as index cannot be corrupted by
                // a missed commit to disk for an advanced index state.
                Commit();
                return;
            }

            {
                LOCK(cs_main);
                const CBlockIndex* pindex_next = NextSyncBlock(pindex);
                if (!pindex_next) {
                    m_best_block_index = pindex;
                    m_synced = true;
                    // No need to handle errors in Commit. See rationale above.
                    Commit();
                    break;
                }
                if (pindex_next->pprev != pindex && !Rewind(pindex, pindex_next->pprev)) {
                    FatalError("%s: Failed to rewind index %s to a previous chain tip",
                               __func__, GetName());
                    return;
                }
                pindex = pindex_next;
            }

            int64_t current_time = GetTime();
            if (last_log_time + SYNC_LOG_INTERVAL < current_time) {
                LogPrintf("Syncing %s with block chain from height %d\n",
                          GetName(), pindex->nHeight);
                last_log_time = current_time;
            }

            if (last_locator_write_time + SYNC_LOCATOR_WRITE_INTERVAL < current_time) {
                m_best_block_index = pindex;
                last_locator_write_time = current_time;
                // No need to handle errors in Commit. See rationale above.
                Commit();
            }

            CBlock block;
            if (!ReadBlockFromDisk(block, pindex, consensus_params)) {
                FatalError("%s: Failed to read block %s from disk",
                           __func__, pindex->GetBlockHash().ToString());
                return;
            }
            if (!WriteBlock(block, pindex)) {
                FatalError("%s: Failed to write block %s to index database",
                           __func__, pindex->GetBlockHash().ToString());
                return;
            }
        }
    }

    if (pindex) {
        LogPrintf("%s is enabled at height %d\n", GetName(), pindex->nHeight);
    } else {
        LogPrintf("%s is enabled\n", GetName());
    }
}

bool BaseIndex::Commit()
{
    CDBBatch batch(GetDB());
    if (!CommitInternal(batch) || !GetDB().WriteBatch(batch)) {
        return error("%s: Failed to commit latest %s state", __func__, GetName());
    }
    return true;
}

bool BaseIndex::CommitInternal(CDBBatch& batch)
{
    LOCK(cs_main);
    GetDB().WriteBestBlock(batch, ::ChainActive().GetLocator(m_best_block_index));
    return true;
}

bool BaseIndex::Rewind(const CBlockIndex* current_tip, const CBlockIndex* new_tip)
{
    assert(current_tip == m_best_block_index);
    assert(current_tip->GetAncestor(new_tip->nHeight) == new_tip);

    // In the case of a reorg, ensure persisted block locator is not stale.
    m_best_block_index = new_tip;
    if (!Commit()) {
        // If commit fails, revert the best block index to avoid corruption.
        m_best_block_index = current_tip;
        return false;
    }

    return true;
}

void BaseIndex::BlockConnected(const std::shared_ptr<const CBlock>& block, const CBlockIndex* pindex,
                               const std::vector<CTransactionRef>& txn_conflicted)
{
    if (!m_synced) {
        return;
    }

    const CBlockIndex* best_block_index = m_best_block_index.load();
    if (!best_block_index) {
        if (pindex->nHeight != 0) {
            FatalError("%s: First block connected is not the genesis block (height=%d)",
                       __func__, pindex->nHeight);
            return;
        }
    } else {
        // Ensure block connects to an ancestor of the current best block. This should be the case
        // most of the time, but may not be immediately after the sync thread catches up and sets
        // m_synced. Consider the case where there is a reorg and the blocks on the stale branch are
        // in the ValidationInterface queue backlog even after the sync thread has caught up to the
        // new chain tip. In this unlikely event, log a warning and let the queue clear.
        if (best_block_index->GetAncestor(pindex->nHeight - 1) != pindex->pprev) {
            LogPrintf("%s: WARNING: Block %s does not connect to an ancestor of " /* Continued */
                      "known best chain (tip=%s); not updating index\n",
                      __func__, pindex->GetBlockHash().ToString(),
                      best_block_index->GetBlockHash().ToString());
            return;
        }
        if (best_block_index != pindex->pprev && !Rewind(best_block_index, pindex->pprev)) {
            FatalError("%s: Failed to rewind index %s to a previous chain tip",
                       __func__, GetName());
            return;
        }
    }

    if (WriteBlock(*block, pindex)) {
        m_best_block_index = pindex;
    } else {
        FatalError("%s: Failed to write block %s to index",
                   __func__, pindex->GetBlockHash().ToString());
        return;
    }
}

void BaseIndex::ChainStateFlushed(const CBlockLocator& locator)
{
    if (!m_synced) {
        return;
    }

    const uint256& locator_tip_hash = locator.vHave.front();
    const CBlockIndex* locator_tip_index;
    {
        LOCK(cs_main);
        locator_tip_index = LookupBlockIndex(locator_tip_hash);
    }

    if (!locator_tip_index) {
        FatalError("%s: First block (hash=%s) in locator was not found",
                   __func__, locator_tip_hash.ToString());
        return;
    }

    // This checks that ChainStateFlushed callbacks are received after BlockConnected. The check may fail
    // immediately after the sync thread catches up and sets m_synced. Consider the case where
    // there is a reorg and the blocks on the stale branch are in the ValidationInterface queue
    // backlog even after the sync thread has caught up to the new chain tip. In this unlikely
    // event, log a warning and let the queue clear.
    const CBlockIndex* best_block_index = m_best_block_index.load();
    if (best_block_index->GetAncestor(locator_tip_index->nHeight) != locator_tip_index) {
        LogPrintf("%s: WARNING: Locator contains block (hash=%s) not on known best " /* Continued */
                  "chain (tip=%s); not writing index locator\n",
                  __func__, locator_tip_hash.ToString(),
                  best_block_index->GetBlockHash().ToString());
        return;
    }

    // No need to handle errors in Commit. If it fails, the error will be already be logged. The
    // best way to recover is to continue, as index cannot be corrupted by a missed commit to disk
    // for an advanced index state.
    Commit();
}

bool BaseIndex::BlockUntilSyncedToCurrentChain()
{
    AssertLockNotHeld(cs_main);

    if (!m_synced) {
        return false;
    }

    {
        // Skip the queue-draining stuff if we know we're caught up with
        // ::ChainActive().Tip().
        LOCK(cs_main);
        const CBlockIndex* chain_tip = ::ChainActive().Tip();
        const CBlockIndex* best_block_index = m_best_block_index.load();
        if (best_block_index->GetAncestor(chain_tip->nHeight) == chain_tip) {
            return true;
        }
    }

    LogPrintf("%s: %s is catching up on block notifications\n", __func__, GetName());
    SyncWithValidationInterfaceQueue();
    return true;
}

void BaseIndex::Interrupt()
{
    m_interrupt();
}

void BaseIndex::Start()
{
    // Need to register this ValidationInterface before running Init(), so that
    // callbacks are not missed if Init sets m_synced to true.
    RegisterValidationInterface(this);
    if (!Init()) {
        FatalError("%s: %s failed to initialize", __func__, GetName());
        return;
    }

    m_thread_sync = std::thread(&TraceThread<std::function<void()>>, GetName(),
                                std::bind(&BaseIndex::ThreadSync, this));
}

void BaseIndex::Stop()
{
    UnregisterValidationInterface(this);

    if (m_thread_sync.joinable()) {
        m_thread_sync.join();
    }
}
