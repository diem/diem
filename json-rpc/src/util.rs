// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Helper macros. Used to simplify adding new RpcHandler to Registry
/// `registry` - name of local registry variable
/// `name`  - name for the rpc method
/// `method` - method name of new rpc method
/// `required_num_args` - number of required method arguments
/// `opt_num_args` - number of optional method arguments
macro_rules! register_rpc_method {
    ($registry:expr, $name: expr, $method: expr, $required_num_args: expr, $opt_num_args: expr) => {
        $registry.insert(
            $name.to_string(),
            Box::new(move |service, request| {
                Box::pin(async move {
                    if request.params.len() < $required_num_args
                        || request.params.len() > $required_num_args + $opt_num_args
                    {
                        anyhow::bail!(JsonRpcError::invalid_params(Some(
                            ErrorData::InvalidArguments(InvalidArguments {
                                required: $required_num_args,
                                optional: $opt_num_args,
                                given: request.params.len(),
                            })
                        )));
                    }

                    Ok(serde_json::to_value($method(service, request).await?)?)
                })
            }),
        );
    };
}
