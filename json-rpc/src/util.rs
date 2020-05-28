// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Helper macros. Used to simplify adding new RpcHandler to Registry
/// `registry` - name of local registry variable
/// `name`  - name for the rpc method
/// `method` - method name of new rpc method
/// `num_args` - number of method arguments
macro_rules! register_rpc_method {
    ($registry:expr, $name: expr, $method: expr, $num_args: expr) => {
        $registry.insert(
            stringify!($name).to_string(),
            Box::new(move |service, request| {
                Box::pin(async move {
                    ensure!(
                        request.params.len() == $num_args,
                        "Invalid number of arguments"
                    );
                    Ok(serde_json::to_value($method(service, request).await?)?)
                })
            }),
        );
    };
}
