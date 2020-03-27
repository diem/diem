// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Helper macros. Used to simplify adding new RpcHandler to Registry
/// `registry` - name of local registry variable
/// `method` - name of new rpc method
/// `num_args` - number of method arguments
macro_rules! register_rpc_method {
    ($registry:expr, $method: expr, $num_args: expr) => {
        $registry.insert(
            stringify!($method).to_string(),
            Box::new(move |service, parameters| {
                Box::pin(async move {
                    ensure!(parameters.len() == $num_args, "Invalid number of arguments");
                    Ok(serde_json::to_value($method(service, parameters).await?)?)
                })
            }),
        );
    };
}
