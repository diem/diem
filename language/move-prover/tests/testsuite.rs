// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use codespan_reporting::term::termcolor::Buffer;

use anyhow::anyhow;
use diem_temppath::TempPath;
use itertools::Itertools;
use move_prover::{cli::Options, run_move_prover};
use move_prover_test_utils::{
    baseline_test::verify_or_update_baseline, extract_test_directives, read_env_var,
};

use datatest_stable::Requirements;
#[allow(unused_imports)]
use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};

const ENV_FLAGS: &str = "MVP_TEST_FLAGS";
const ENV_TEST_EXTENDED: &str = "MVP_TEST_X";
const MVP_TEST_INCONSISTENCY: &str = "MVP_TEST_INCONSISTENCY";
const INCONSISTENCY_TEST_FLAGS: &[&str] = &[
    "--dependency=../move-stdlib/modules",
    "--dependency=../diem-framework/modules",
    "--check-inconsistency",
];
const REGULAR_TEST_FLAGS: &[&str] = &[
    "--dependency=../move-stdlib/modules",
    "--dependency=../diem-framework/modules",
];

static NOT_CONFIGURED_WARNED: AtomicBool = AtomicBool::new(false);

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    // Use the below + `cargo test -- --test-threads=1` to identify a long running test
    //println!(">>> testing {}", path.to_string_lossy().to_string());
    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();

    let temp_dir = TempPath::new();
    std::fs::create_dir_all(temp_dir.path())?;
    let (flags, baseline_path) = get_flags(temp_dir.path(), path)?;

    let mut args = vec!["mvp_test".to_string()];
    args.extend(flags);
    args.push("--verbose=warn".to_owned());
    // TODO: timeouts aren't handled correctly by the boogie wrapper but lead to hang. Determine
    //   reasons and reactivate.
    // args.push("--num-instances=2".to_owned()); // run two Boogie instances with different seeds
    // args.push("--sequential".to_owned());
    args.push(path.to_string_lossy().to_string());

    args.extend(shell_words::split(&read_env_var(ENV_FLAGS))?);

    let mut options = Options::create_from_args(&args)?;
    options.setup_logging_for_test();
    if no_boogie {
        options.prover.generate_only = true;
        if NOT_CONFIGURED_WARNED
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            warn!(
                "Prover tools are not configured, verification tests will be skipped. \
        See https://github.com/diem/diem/tree/main/language/move-prover/doc/user/install.md \
        for instructions."
            );
        }
    }
    options.prover.stable_test_output = true;
    options.backend.stable_test_output = true;

    let mut error_writer = Buffer::no_color();
    let mut diags = match run_move_prover(&mut error_writer, options) {
        Ok(()) => "".to_string(),
        Err(err) => format!("Move prover returns: {}\n", err),
    };
    if baseline_valid {
        if let Some(ref path) = baseline_path {
            diags += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
            verify_or_update_baseline(path.as_path(), &diags)?
        } else if !diags.is_empty() {
            return Err(anyhow!(
                "Unexpected prover output (expected none): {}{}",
                diags,
                String::from_utf8_lossy(&error_writer.into_inner())
            )
            .into());
        }
    }

    // Run again with cvc4 if TEST_CVC4 is set and UPBL (update baselines) is not set.
    // We do not run CVC4 based tests by default because the way how things are setup,
    // they would always be run in CI and make verification roughly 2x slower because all tools
    // are installed in CI and on user machines and `CVC4_EXE` is always set.
    if !read_env_var("MVP_TEST_CVC4").is_empty()
        && read_env_var("UPBL").is_empty()
        && !no_boogie
        && !read_env_var("CVC4_EXE").is_empty()
        && !cvc4_deny_listed(path)
    {
        info!("running with cvc4");
        args.push("--use-cvc4".to_owned());
        options = Options::create_from_args(&args)?;
        options.setup_logging_for_test();
        options.prover.stable_test_output = true;
        error_writer = Buffer::no_color();
        diags = match run_move_prover(&mut error_writer, options) {
            Ok(()) => "".to_string(),
            Err(err) => format!("Move prover returns: {}\n", err),
        };
        if let Some(path) = baseline_path {
            diags += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
            verify_or_update_baseline(path.as_path(), &diags)?
        } else if !diags.is_empty() {
            return Err(anyhow!(
                "Unexpected prover output (expected none): {}{}",
                diags,
                String::from_utf8_lossy(&error_writer.into_inner())
            )
            .into());
        }
    }

    Ok(())
}

fn cvc4_deny_listed(path: &Path) -> bool {
    static DENY_LIST: &[&str] = &[
        "diem-framework/modules/ValidatorOperatorConfig.move",
        "diem-framework/modules/Option.move",
        "diem-framework/modules/RegisteredCurrencies.move",
        "diem-framework/modules/AccountFreezing.move",
        "diem-framework/modules/DiemTransactionPublishingOption.move",
        "diem-framework/modules/VASP.move",
        "diem-framework/modules/ValidatorConfig.move",
        "diem-framework/modules/DiemConfig.move",
        "diem-framework/modules/DiemSystem.move",
        "diem-framework/modules/XUS.move",
        "diem-framework/modules/DualAttestation.move",
        "diem-framework/modules/XDX.move",
        "tests/sources/functional/cast.move",
        "diem-framework/modules/RecoveryAddress.move",
        "tests/sources/functional/loops.move",
        "diem-framework/transaction_scripts/add_validator_and_reconfigure.move",
        "diem-framework/transaction_scripts/rotate_authentication_key.move",
        "tests/sources/functional/aborts_if_assume_assert.move",
        "diem-framework/transaction_scripts/remove_validator_and_reconfigure.move",
        "diem-framework/modules/DesignatedDealer.move",
        "tests/sources/functional/marketcap.move",
        "tests/sources/functional/invariants.move",
        "tests/sources/functional/invariants_resources.move",
        "tests/sources/functional/module_invariants.move",
        "tests/sources/functional/ModifiesSchemaTest.move",
        "tests/sources/functional/resources.move",
        "tests/sources/functional/schema_exp.move",
        "tests/sources/functional/marketcap_generic.move",
        "tests/sources/functional/aborts_if_with_code.move",
        "tests/sources/functional/address_serialization_constant_size.move",
        "diem-framework/transaction_scripts/set_validator_config_and_reconfigure.move",
        "diem-framework/burn.move",
        "tests/sources/functional/global_invariants.move",
        "tests/sources/functional/nested_invariants.move",
        "diem-framework/transaction_scripts/rotate_authentication_key_with_recovery_address.move",
        "tests/sources/functional/marketcap_as_schema_apply.move",
        "tests/sources/functional/global_vars.move",
        "diem-framework/transaction_scripts/rotate_authentication_key_with_nonce.move",
        "tests/sources/functional/specs_in_fun_ref.move",
        "tests/sources/functional/references.move",
        "tests/sources/functional/mut_ref_unpack.move",
        "tests/sources/functional/hash_model.move",
        "tests/sources/functional/ModifiesErrorTest.move",
        "tests/sources/functional/consts.move",
        "tests/sources/functional/type_values.move",
        "tests/sources/functional/pragma.move",
        "tests/sources/functional/exists_in_vector.move",
        "tests/sources/functional/aborts_with_check.move",
        "tests/sources/functional/aborts_with_negative_check.move",
        "tests/sources/functional/opaque.move",
        "tests/sources/functional/marketcap_as_schema.move",
        "tests/sources/functional/aborts_if.move",
        "tests/sources/functional/address_quant.move",
        "tests/sources/functional/hash_model_invalid.move",
        "tests/sources/functional/serialize_model.move",
        "tests/sources/functional/return_values.move",
        "tests/sources/functional/pack_unpack.move",
        "tests/sources/functional/specs_in_fun.move",
        "tests/sources/functional/arithm.move",
        "tests/sources/regression/Escape.move",
        "tests/sources/functional/mut_ref_accross_modules.move",
        "tests/sources/regression/type_param_bug_200228.move",
        "tests/sources/regression/trace200527.move",
        "tests/sources/regression/generic_invariant200518.move",
        "diem-framework/transaction_scripts/rotate_authentication_key_with_nonce_admin.move",
        "tests/sources/functional/simple_vector_client.move",
        "diem-framework/transaction_scripts/publish_shared_ed25519_public_key.move",
        "tests/sources/functional/verify_vector.move",
        "diem-framework/transaction_scripts/create_designated_dealer.move",
        "diem-framework/transaction_scripts/create_parent_vasp_account.move",
        "diem-framework/modules/Diem.move",
        "diem-framework/transaction_scripts/create_child_vasp_account.move",
        "diem-framework/modules/FixedPoint32.move",
        "diem-framework/modules/Genesis.move",
        "diem-framework/modules/DiemAccount.move",
        "diem-framework/transaction_scripts/update_exchange_rate.move",
        "tests/sources/functional/script_incorrect.move",
        "tests/sources/functional/emits.move",
        "tests/sources/functional/friend.move",
        "tests/sources/regression/set_200701.move",
        "diem-framework/modules/DiemBlock.move",
        "diem-framework/modules/ChainId.move",
        "diem-framework/modules/DiemVMConfig.move",
        "diem-framework/modules/SlidingNonce.move",
        "diem-framework/modules/TransactionFee.move",
        "diem-framework/modules/Roles.move",
        "diem-framework/modules/DiemTimestamp.move",
        "diem-framework/modules/DiemVersion.move",
        "diem-framework/modules/AccountLimits.move", // This one takes over a minute
    ];

    let path_str = path.to_str().unwrap();
    for entry in DENY_LIST {
        if path_str.contains(entry) {
            return true;
        }
    }
    false
}

fn get_flags(temp_dir: &Path, path: &Path) -> anyhow::Result<(Vec<String>, Option<PathBuf>)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();

    let stdlib_test_flags = if read_env_var(MVP_TEST_INCONSISTENCY).is_empty() {
        REGULAR_TEST_FLAGS
    } else {
        INCONSISTENCY_TEST_FLAGS
    };

    let (base_flags, baseline_path, modifier) =
        if path_str.contains("diem-framework/") || path_str.contains("move-stdlib/") {
            (stdlib_test_flags, None, "std_")
        } else {
            (
                REGULAR_TEST_FLAGS,
                Some(path.with_extension("exp")),
                "prover_",
            )
        };
    let mut flags = base_flags.iter().map(|s| (*s).to_string()).collect_vec();
    // Add any flags specified in the source.
    flags.extend(extract_test_directives(path, "// flag:")?);

    // Create a temporary file for output. We inject the modifier to potentially prevent
    // any races between similar named files in different directories, as it appears TempPath
    // isn't working always.
    let base_name = format!(
        "{}{}.bpl",
        modifier,
        path.file_stem().unwrap().to_str().unwrap()
    );
    let output = temp_dir.join(base_name).to_str().unwrap().to_string();
    flags.push(format!("--output={}", output));
    Ok((flags, baseline_path))
}

// Test entry point based on datatest runner.
fn main() {
    let mut reqs = vec![];
    if read_env_var(ENV_TEST_EXTENDED) == "1" {
        reqs.push(Requirements::new(
            test_runner,
            "extended".to_string(),
            "tests/xsources".to_string(),
            r".*\.move$".to_string(),
        ));
    } else {
        reqs.push(Requirements::new(
            test_runner,
            "functional".to_string(),
            "tests/sources".to_string(),
            r".*\.move$".to_string(),
        ));
        reqs.push(Requirements::new(
            test_runner,
            "fx".to_string(),
            "../move-stdlib".to_string(),
            r".*\.move$".to_string(),
        ));
        reqs.push(Requirements::new(
            test_runner,
            "fx".to_string(),
            "../diem-framework".to_string(),
            r".*\.move$".to_string(),
        ));
    }
    datatest_stable::runner(&reqs);
}
