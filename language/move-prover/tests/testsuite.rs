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

#[allow(unused_imports)]
use log::{debug, warn};
use std::sync::atomic::{AtomicBool, Ordering};

const ENV_FLAGS: &str = "MVP_TEST_FLAGS";
const STDLIB_FLAGS: &[&str] = &["--dependency=../stdlib/modules"];

static NOT_CONFIGURED_WARNED: AtomicBool = AtomicBool::new(false);

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();

    let temp_dir = TempPath::new();
    std::fs::create_dir_all(temp_dir.path())?;
    let (flags, baseline_path) = get_flags(temp_dir.path(), path)?;

    let mut args = vec!["mvp_test".to_string()];
    args.extend(flags);
    args.push("--verbose=warn".to_owned());
    args.push("--num-instances=2".to_owned()); // run two Boogie instances with different seeds
    args.push("--sequential".to_owned());
    args.push(path.to_string_lossy().to_string());

    args.extend(shell_words::split(&read_env_var(ENV_FLAGS))?);

    let mut options = Options::create_from_args(&args)?;
    options.setup_logging_for_test();
    if no_boogie {
        options.prover.generate_only = true;
        if !NOT_CONFIGURED_WARNED.compare_and_swap(false, true, Ordering::Relaxed) {
            warn!(
                "Prover tools are not configured, verification tests will be skipped. \
        See https://github.com/diem/diem/tree/master/language/move-prover/doc/user/install.md \
        for instructions."
            );
        }
    }
    options.prover.stable_test_output = true;

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

    // Run again with cvc4
    if !no_boogie && !read_env_var("CVC4_EXE").is_empty() && !cvc4blacklisted(path) {
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

fn cvc4blacklisted(path: &Path) -> bool {
    let path_str = path.to_str().unwrap();
    if path_str == "../stdlib/modules/ValidatorOperatorConfig.move" ||
        path_str == "../stdlib/modules/Option.move" ||
        path_str == "../stdlib/modules/RegisteredCurrencies.move" ||
        path_str == "../stdlib/modules/AccountFreezing.move" ||
        path_str == "../stdlib/modules/DiemTransactionPublishingOption.move" ||
        path_str == "../stdlib/modules/VASP.move" ||
        path_str == "../stdlib/modules/ValidatorConfig.move" ||
        path_str == "../stdlib/modules/DiemConfig.move" ||
        path_str == "../stdlib/modules/DiemSystem.move" ||
        path_str == "../stdlib/modules/XUS.move" ||
        path_str == "../stdlib/modules/DualAttestation.move" ||
        path_str == "../stdlib/modules/XDX.move" ||
        path_str == "tests/sources/functional/cast.move" ||
        path_str == "../stdlib/modules/RecoveryAddress.move" ||
        path_str == "tests/sources/functional/loops.move" ||
        path_str == "../stdlib/transaction_scripts/add_validator_and_reconfigure.move" ||
        path_str == "../stdlib/transaction_scripts/rotate_authentication_key.move" ||
        path_str == "tests/sources/functional/aborts_if_assume_assert.move" ||
        path_str == "../stdlib/transaction_scripts/remove_validator_and_reconfigure.move" ||
        path_str == "../stdlib/modules/DesignatedDealer.move" ||
        path_str == "tests/sources/functional/marketcap.move" ||
        path_str == "tests/sources/functional/invariants.move" ||
        path_str == "tests/sources/functional/module_invariants.move" ||
        path_str == "tests/sources/functional/ModifiesSchemaTest.move" ||
        path_str == "tests/sources/functional/resources.move" ||
        path_str == "tests/sources/functional/schema_exp.move" ||
        path_str == "tests/sources/functional/marketcap_generic.move" ||
        path_str == "tests/sources/functional/aborts_if_with_code.move" ||
        path_str == "tests/sources/functional/address_serialization_constant_size.move" ||
        path_str == "../stdlib/transaction_scripts/set_validator_config_and_reconfigure.move" ||
        path_str == "tests/sources/functional/global_invariants.move" ||
        path_str == "tests/sources/functional/nested_invariants.move" ||
        path_str == "../stdlib/transaction_scripts/rotate_authentication_key_with_recovery_address.move" ||
        path_str == "tests/sources/functional/marketcap_as_schema_apply.move" ||
        path_str == "tests/sources/functional/global_vars.move" ||
        path_str == "../stdlib/transaction_scripts/rotate_authentication_key_with_nonce.move" ||
        path_str == "tests/sources/functional/specs_in_fun_ref.move" ||
        path_str == "tests/sources/functional/references.move" ||
        path_str == "tests/sources/functional/mut_ref_unpack.move" ||
        path_str == "tests/sources/functional/hash_model.move" ||
        path_str == "tests/sources/functional/ModifiesErrorTest.move" ||
        path_str == "tests/sources/functional/consts.move" ||
        path_str == "tests/sources/functional/type_values.move" ||
        path_str == "tests/sources/functional/pragma.move" ||
        path_str == "tests/sources/functional/exists_in_vector.move" ||
        path_str == "tests/sources/functional/aborts_with_check.move" ||
        path_str == "tests/sources/functional/aborts_with_negative_check.move" ||
        path_str == "tests/sources/functional/opaque.move" ||
        path_str == "tests/sources/functional/marketcap_as_schema.move" ||
        path_str == "tests/sources/functional/aborts_if.move" ||
        path_str == "tests/sources/functional/address_quant.move" ||
        path_str == "tests/sources/functional/hash_model_invalid.move" ||
        path_str == "tests/sources/functional/serialize_model.move" ||
        path_str == "tests/sources/functional/return_values.move" ||
        path_str == "tests/sources/functional/pack_unpack.move" ||
        path_str == "tests/sources/functional/specs_in_fun.move" ||
        path_str == "tests/sources/functional/arithm.move" ||
        path_str == "tests/sources/regression/Escape.move" ||
        path_str == "tests/sources/functional/mut_ref_accross_modules.move" ||
        path_str == "tests/sources/regression/type_param_bug_200228.move" ||
        path_str == "tests/sources/regression/trace200527.move" ||
        path_str == "tests/sources/regression/generic_invariant200518.move" ||
        path_str == "../stdlib/transaction_scripts/rotate_authentication_key_with_nonce_admin.move" ||
        path_str == "tests/sources/functional/simple_vector_client.move" ||
        path_str == "../stdlib/transaction_scripts/publish_shared_ed25519_public_key.move" ||
        path_str == "tests/sources/functional/verify_vector.move" ||
        path_str == "../stdlib/transaction_scripts/create_designated_dealer.move" ||
        path_str == "../stdlib/transaction_scripts/create_parent_vasp_account.move" ||
        path_str == "../stdlib/modules/Diem.move" ||
        path_str == "../stdlib/transaction_scripts/create_child_vasp_account.move" ||
        path_str == "../stdlib/modules/FixedPoint32.move" ||
        path_str == "../stdlib/modules/Genesis.move" ||
        path_str == "../stdlib/modules/DiemAccount.move" ||

        path_str == "../stdlib/modules/AccountLimits.move" || // This one takes over a minute

        true
    {
        return true;
    }
    false
}

fn test_runner_stdlib(path: &Path) -> datatest_stable::Result<()> {
    // Gives the standard test runner a different name in test output, which is useful because
    // the datatest infrastructure drops `..` in test file paths.
    test_runner(path)
}

fn get_flags(temp_dir: &Path, path: &Path) -> anyhow::Result<(Vec<String>, Option<PathBuf>)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();
    let (base_flags, baseline_path, modifier) = if path_str.contains("../stdlib/") {
        (STDLIB_FLAGS, None, "std_")
    } else if path_str.contains("tests/sources/functional/")
        || path_str.contains("tests/sources/regression/")
    {
        (STDLIB_FLAGS, Some(path.with_extension("exp")), "func_")
    } else {
        return Err(anyhow!(
            "do not know how to run tests for `{}` because it's directory is not configured",
            path_str
        ));
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

datatest_stable::harness!(
    // Run tests for the content of our tests directory.
    test_runner,
    "tests/sources",
    r".*\.move$",
    // Run tests for the content of the stdlib directory.
    test_runner_stdlib,
    "../stdlib",
    r".*\.move$"
);
