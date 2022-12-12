// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use clap::Parser;
use diem::Tool;

#[tokio::test]
async fn test_all_commands() {
    let diem = "diem".to_string();
    let account = "account".to_string();
    run_command(vec![diem, account, "create".to_string()]).await;
}

async fn run_command(arguments: Vec<String>) {
    let tool = Tool::try_parse_from(arguments).unwrap();

    match tool.execute().await {
        Ok(_) => {}
        Err(err) => {
            panic!("Error occurred during test: {}", err)
        }
    }
}
