// Copyright (c) 2018-2019 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <memory>

#include <boost/test/unit_test.hpp>

#include <fs.h>
#include <test/setup_common.h>
#include <wallet/db.h>


BOOST_FIXTURE_TEST_SUITE(db_tests, BasicTestingSetup)

BOOST_AUTO_TEST_CASE(getwalletenv_file)
{
    std::string test_name = "test_name.dat";
    fs::path datadir = SetDataDir("tempdir");
    fs::path file_path = datadir / test_name;
    std::ofstream f(file_path.BOOST_FILESYSTEM_C_STR);
    f.close();

    std::string filename;
    std::shared_ptr<BerkeleyEnvironment> env = GetWalletEnv(file_path, filename);
    BOOST_CHECK(filename == test_name);
    BOOST_CHECK(env->Directory() == datadir);
}

BOOST_AUTO_TEST_CASE(getwalletenv_directory)
{
    std::string expected_name = "wallet.dat";
    fs::path datadir = SetDataDir("tempdir");

    std::string filename;
    std::shared_ptr<BerkeleyEnvironment> env = GetWalletEnv(datadir, filename);
    BOOST_CHECK(filename == expected_name);
    BOOST_CHECK(env->Directory() == datadir);
}

BOOST_AUTO_TEST_CASE(getwalletenv_g_dbenvs_multiple)
{
    fs::path datadir = SetDataDir("tempdir");
    fs::path datadir_2 = SetDataDir("tempdir_2");
    std::string filename;

    std::shared_ptr<BerkeleyEnvironment> env_1 = GetWalletEnv(datadir, filename);
    std::shared_ptr<BerkeleyEnvironment> env_2 = GetWalletEnv(datadir, filename);
    std::shared_ptr<BerkeleyEnvironment> env_3 = GetWalletEnv(datadir_2, filename);

    BOOST_CHECK(env_1 == env_2);
    BOOST_CHECK(env_2 != env_3);
}

BOOST_AUTO_TEST_CASE(getwalletenv_g_dbenvs_free_instance)
{
    fs::path datadir = SetDataDir("tempdir");
    fs::path datadir_2 = SetDataDir("tempdir_2");
    std::string filename;

    std::shared_ptr <BerkeleyEnvironment> env_1_a = GetWalletEnv(datadir, filename);
    std::shared_ptr <BerkeleyEnvironment> env_2_a = GetWalletEnv(datadir_2, filename);
    env_1_a.reset();

    std::shared_ptr<BerkeleyEnvironment> env_1_b = GetWalletEnv(datadir, filename);
    std::shared_ptr<BerkeleyEnvironment> env_2_b = GetWalletEnv(datadir_2, filename);

    BOOST_CHECK(env_1_a != env_1_b);
    BOOST_CHECK(env_2_a == env_2_b);
}

BOOST_AUTO_TEST_SUITE_END()
