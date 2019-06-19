// Copyright (c) 2009-2010 Satoshi Nakamoto
// Copyright (c) 2009-2017 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <banman.h>

#include <netaddress.h>
#include <ui_interface.h>
#include <util/system.h>
#include <util/time.h>


BanMan::BanMan(fs::path ban_file, CClientUIInterface* client_interface, int64_t default_ban_time)
    : m_client_interface(client_interface), m_ban_db(std::move(ban_file)), m_default_ban_time(default_ban_time)
{
    if (m_client_interface) m_client_interface->InitMessage(_("Loading banlist..."));

    int64_t n_start = GetTimeMillis();
    m_is_dirty = false;
    banmap_t banmap;
    if (m_ban_db.Read(banmap)) {
        SetBanned(banmap);        // thread save setter
        SetBannedSetDirty(false); // no need to write down, just read data
        SweepBanned();            // sweep out unused entries

        LogPrint(BCLog::NET, "Loaded %d banned node ips/subnets from banlist.dat  %dms\n",
            banmap.size(), GetTimeMillis() - n_start);
    } else {
        LogPrintf("Invalid or missing banlist.dat; recreating\n");
        SetBannedSetDirty(true); // force write
        DumpBanlist();
    }
}

BanMan::~BanMan()
{
    DumpBanlist();
}

void BanMan::DumpBanlist()
{
    SweepBanned(); // clean unused entries (if bantime has expired)

    if (!BannedSetIsDirty()) return;

    int64_t n_start = GetTimeMillis();

    banmap_t banmap;
    GetBanned(banmap);
    if (m_ban_db.Write(banmap)) {
        SetBannedSetDirty(false);
    }

    LogPrint(BCLog::NET, "Flushed %d banned node ips/subnets to banlist.dat  %dms\n",
        banmap.size(), GetTimeMillis() - n_start);
}

void BanMan::ClearBanned()
{
    {
        LOCK(m_cs_banned);
        m_banned.clear();
        m_is_dirty = true;
    }
    DumpBanlist(); //store banlist to disk
    if (m_client_interface) m_client_interface->BannedListChanged();
}

int BanMan::IsBannedLevel(CNetAddr net_addr)
{
    // Returns the most severe level of banning that applies to this address.
    // 0 - Not banned
    // 1 - Automatic misbehavior ban
    // 2 - Any other ban
    int level = 0;
    auto current_time = GetTime();
    LOCK(m_cs_banned);
    for (const auto& it : m_banned) {
        CSubNet sub_net = it.first;
        CBanEntry ban_entry = it.second;

        if (current_time < ban_entry.nBanUntil && sub_net.Match(net_addr)) {
            if (ban_entry.banReason != BanReasonNodeMisbehaving) return 2;
            level = 1;
        }
    }
    return level;
}

bool BanMan::IsBanned(CNetAddr net_addr)
{
    auto current_time = GetTime();
    LOCK(m_cs_banned);
    for (const auto& it : m_banned) {
        CSubNet sub_net = it.first;
        CBanEntry ban_entry = it.second;

        if (current_time < ban_entry.nBanUntil && sub_net.Match(net_addr)) {
            return true;
        }
    }
    return false;
}

bool BanMan::IsBanned(CSubNet sub_net)
{
    auto current_time = GetTime();
    LOCK(m_cs_banned);
    banmap_t::iterator i = m_banned.find(sub_net);
    if (i != m_banned.end()) {
        CBanEntry ban_entry = (*i).second;
        if (current_time < ban_entry.nBanUntil) {
            return true;
        }
    }
    return false;
}

void BanMan::Ban(const CNetAddr& net_addr, const BanReason& ban_reason, int64_t ban_time_offset, bool since_unix_epoch)
{
    CSubNet sub_net(net_addr);
    Ban(sub_net, ban_reason, ban_time_offset, since_unix_epoch);
}

void BanMan::Ban(const CSubNet& sub_net, const BanReason& ban_reason, int64_t ban_time_offset, bool since_unix_epoch)
{
    CBanEntry ban_entry(GetTime(), ban_reason);

    int64_t normalized_ban_time_offset = ban_time_offset;
    bool normalized_since_unix_epoch = since_unix_epoch;
    if (ban_time_offset <= 0) {
        normalized_ban_time_offset = m_default_ban_time;
        normalized_since_unix_epoch = false;
    }
    ban_entry.nBanUntil = (normalized_since_unix_epoch ? 0 : GetTime()) + normalized_ban_time_offset;

    {
        LOCK(m_cs_banned);
        if (m_banned[sub_net].nBanUntil < ban_entry.nBanUntil) {
            m_banned[sub_net] = ban_entry;
            m_is_dirty = true;
        } else
            return;
    }
    if (m_client_interface) m_client_interface->BannedListChanged();

    //store banlist to disk immediately if user requested ban
    if (ban_reason == BanReasonManuallyAdded) DumpBanlist();
}

bool BanMan::Unban(const CNetAddr& net_addr)
{
    CSubNet sub_net(net_addr);
    return Unban(sub_net);
}

bool BanMan::Unban(const CSubNet& sub_net)
{
    {
        LOCK(m_cs_banned);
        if (m_banned.erase(sub_net) == 0) return false;
        m_is_dirty = true;
    }
    if (m_client_interface) m_client_interface->BannedListChanged();
    DumpBanlist(); //store banlist to disk immediately
    return true;
}

void BanMan::GetBanned(banmap_t& banmap)
{
    LOCK(m_cs_banned);
    // Sweep the banlist so expired bans are not returned
    SweepBanned();
    banmap = m_banned; //create a thread safe copy
}

void BanMan::SetBanned(const banmap_t& banmap)
{
    LOCK(m_cs_banned);
    m_banned = banmap;
    m_is_dirty = true;
}

void BanMan::SweepBanned()
{
    int64_t now = GetTime();
    bool notify_ui = false;
    {
        LOCK(m_cs_banned);
        banmap_t::iterator it = m_banned.begin();
        while (it != m_banned.end()) {
            CSubNet sub_net = (*it).first;
            CBanEntry ban_entry = (*it).second;
            if (now > ban_entry.nBanUntil) {
                m_banned.erase(it++);
                m_is_dirty = true;
                notify_ui = true;
                LogPrint(BCLog::NET, "%s: Removed banned node ip/subnet from banlist.dat: %s\n", __func__, sub_net.ToString());
            } else
                ++it;
        }
    }
    // update UI
    if (notify_ui && m_client_interface) {
        m_client_interface->BannedListChanged();
    }
}

bool BanMan::BannedSetIsDirty()
{
    LOCK(m_cs_banned);
    return m_is_dirty;
}

void BanMan::SetBannedSetDirty(bool dirty)
{
    LOCK(m_cs_banned); //reuse m_banned lock for the m_is_dirty flag
    m_is_dirty = dirty;
}
