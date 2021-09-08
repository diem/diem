#[test_only]
module DiemFramework::AuthenticatorTests {
    use DiemFramework::Authenticator;
    use Std::Vector;

    #[test]
    fun create_valid_multisig_policies_and_compute_auth_keys() {
        let pubkey1 = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let pubkey2 = x"4b2a60883383be0ba24ed79aa5a6c9379728099a7b0c57edcec193a14ea5fce2";
        let pubkey3 = x"323285d3d4b0f19482730c5f481d9f745c2927d73c231bad47859d9b2f7376f1";

        let keys = Vector::empty<vector<u8>>();
        Vector::push_back(&mut keys, pubkey1);
        let t = Authenticator::create_multi_ed25519(copy keys, 1);
        let auth_key = Authenticator::multi_ed25519_authentication_key(&t);

        Vector::push_back(&mut keys, pubkey2);
        t = Authenticator::create_multi_ed25519(copy keys, 1);
        assert(Authenticator::multi_ed25519_authentication_key(&t) != copy auth_key, 3006);
        t = Authenticator::create_multi_ed25519(copy keys, 2);
        assert(Authenticator::multi_ed25519_authentication_key(&t) != copy auth_key, 3007);

        Vector::push_back(&mut keys, copy pubkey3);
        t = Authenticator::create_multi_ed25519(copy keys, 1);
        assert(Authenticator::multi_ed25519_authentication_key(&t) != copy auth_key, 3008);
        t = Authenticator::create_multi_ed25519(copy keys, 2);
        assert(Authenticator::multi_ed25519_authentication_key(&t) != copy auth_key, 3009);
        // check that auth key matches expect result
        assert(
            Authenticator::multi_ed25519_authentication_key(&t)
            ==
            x"1761bca45f83ecdefe202650ca5ba9518b9c2cc032667a95b275dc3f43173ae0",
        3011
        );

        // duplicate keys are ok
        Vector::push_back(&mut keys, pubkey3);
        t = Authenticator::create_multi_ed25519(copy keys, 3);
        assert(Authenticator::multi_ed25519_authentication_key(&t) != copy auth_key, 3012);

        assert(Authenticator::threshold(&t) == 3, 3013);
        assert(Authenticator::public_keys(&t) == &keys, 3014);
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun empty_policy_should_be_rejected() {
        let keys = Vector::empty<vector<u8>>();
        Authenticator::create_multi_ed25519(keys, 0);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun bad_threshold_should_be_rejected_threshold_1_for_empty_keys() {
        let keys = Vector::empty<vector<u8>>();
        Authenticator::create_multi_ed25519(keys, 1);
    }

    #[test]
    #[expected_failure(abort_code = 519)]
    fun bad_threshold_should_be_rejected_threshold_3_for_34_keys() {
        let pubkey = x"";

        let keys = Vector::empty<vector<u8>>();
        let index = 0;
        while (index < 34) {
            Vector::push_back(&mut keys, copy pubkey);
            index = index + 1;
        };
        let _auth_key =
        Authenticator::create_multi_ed25519(keys, 3);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun bad_threshold_should_be_rejected_threshold_2_for_1_key() {
        let keys = Vector::empty<vector<u8>>();
        Vector::push_back(
            &mut keys,
            x"2000000000000000000000000000000000000000000000000000000000000000"
        );
        Authenticator::create_multi_ed25519(keys, 2);
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun bad_threshold_should_be_rejected_threshold_0_for_1_address() {
        let keys = Vector::empty<vector<u8>>();
        Vector::push_back(
            &mut keys,
            x"2000000000000000000000000000000000000000000000000000000000000000"
        );
        Authenticator::create_multi_ed25519(keys, 0);
    }

    #[test]
    fun one_of_one_multi_ed25519_should_have_different_auth_key_than_ed25519_with_same_public_key() {
        let pubkey = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let keys = Vector::empty<vector<u8>>();
        Vector::push_back(
            &mut keys,
            copy pubkey
        );

        let t = Authenticator::create_multi_ed25519(keys, 1);
        assert(
            Authenticator::multi_ed25519_authentication_key(&t) !=
                Authenticator::ed25519_authentication_key(copy pubkey),
            3011
        );
        assert(
            x"ba10abb6d85ea3897baa1cae457fc724a916d258bd47ab852f200c5851a6d057"
            ==
            Authenticator::ed25519_authentication_key(pubkey),
            3012
        );
    }
}
