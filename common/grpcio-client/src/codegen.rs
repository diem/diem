// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use protobuf::{
    compiler_plugin,
    descriptor::{FileDescriptorProto, MethodDescriptorProto, ServiceDescriptorProto},
    descriptorx::{RootScope, WithScope},
};

use super::util;
use protobuf_codegen::code_writer::CodeWriter;

/*
This is mostly copied from grpcio-compiler.
It's copied and not reimplemented or re-used in some way because:
Most methods there are private, and I have to use the same names/structs
for the generated output.
*/

struct MethodGen<'a> {
    proto: &'a MethodDescriptorProto,
    root_scope: &'a RootScope<'a>,
}

impl<'a> MethodGen<'a> {
    fn new(proto: &'a MethodDescriptorProto, root_scope: &'a RootScope<'a>) -> MethodGen<'a> {
        MethodGen { proto, root_scope }
    }

    fn input(&self) -> String {
        format!(
            "super::{}",
            self.root_scope
                .find_message(self.proto.get_input_type())
                .rust_fq_name()
        )
    }

    fn output(&self) -> String {
        format!(
            "super::{}",
            self.root_scope
                .find_message(self.proto.get_output_type())
                .rust_fq_name()
        )
    }

    fn method_type(&self) -> (util::MethodType, String) {
        match (
            self.proto.get_client_streaming(),
            self.proto.get_server_streaming(),
        ) {
            (false, false) => (
                util::MethodType::Unary,
                util::fq_grpc("util::MethodType::Unary"),
            ),
            (true, false) => (
                util::MethodType::ClientStreaming,
                util::fq_grpc("util::MethodType::ClientStreaming"),
            ),
            (false, true) => (
                util::MethodType::ServerStreaming,
                util::fq_grpc("util::MethodType::ServerStreaming"),
            ),
            (true, true) => (
                util::MethodType::Duplex,
                util::fq_grpc("util::MethodType::Duplex"),
            ),
        }
    }

    fn name(&self) -> String {
        util::to_snake_case(self.proto.get_name())
    }

    // Method signatures
    fn unary(&self, method_name: &str) -> String {
        format!(
            "{}(&self, req: &{}) -> {}<{}>",
            method_name,
            self.input(),
            util::fq_grpc("Result"),
            self.output()
        )
    }

    fn unary_async(&self, method_name: &str) -> String {
        format!(
            "{}_async(&self, req: &{}) -> {}<Box<Future<Item={}, Error={}> + Send>>",
            method_name,
            self.input(),
            util::fq_grpc("Result"),
            self.output(),
            util::fq_grpc("Error")
        )
    }

    fn client_streaming(&self, method_name: &str) -> String {
        format!(
            "{}(&self) -> {}<({}<{}>, {}<{}>)>",
            method_name,
            util::fq_grpc("Result"),
            util::fq_grpc("ClientCStreamSender"),
            self.input(),
            util::fq_grpc("ClientCStreamReceiver"),
            self.output()
        )
    }

    fn server_streaming(&self, method_name: &str) -> String {
        format!(
            "{}(&self, req: &{}) -> {}<{}<{}>>",
            method_name,
            self.input(),
            util::fq_grpc("Result"),
            util::fq_grpc("ClientSStreamReceiver"),
            self.output()
        )
    }

    fn duplex_streaming(&self, method_name: &str) -> String {
        format!(
            "{}(&self) -> {}<({}<{}>, {}<{}>)>",
            method_name,
            util::fq_grpc("Result"),
            util::fq_grpc("ClientDuplexSender"),
            self.input(),
            util::fq_grpc("ClientDuplexReceiver"),
            self.output()
        )
    }

    fn write_method(&self, has_impl: bool, w: &mut CodeWriter<'_>) {
        let method_name = self.name();
        let method_name = method_name.as_str();

        // some parts are not implemented yet, account for both
        let (sig, implemented) = match self.method_type().0 {
            util::MethodType::Unary => (self.unary(method_name), true),
            util::MethodType::ClientStreaming => (self.client_streaming(method_name), false),
            util::MethodType::ServerStreaming => (self.server_streaming(method_name), false),
            util::MethodType::Duplex => (self.duplex_streaming(method_name), false),
        };

        if has_impl {
            w.def_fn(sig.as_str(), |w| {
                if implemented {
                    w.write_line(format!("self.{}(req)", method_name));
                } else {
                    w.unimplemented();
                }
            });
        } else {
            w.def_fn(sig.as_str(), |w| {
                w.unimplemented();
            });
        }

        // async variant: only implemented for unary methods for now
        if let util::MethodType::Unary = self.method_type().0 {
            w.def_fn(self.unary_async(method_name).as_str(), |w| {
                if has_impl {
                    w.match_expr(format!("self.{}_async(req)", method_name), |w| {
                        w.case_expr("Ok(f)", "Ok(Box::new(f))");
                        w.case_expr("Err(e)", "Err(e)");
                    });
                } else {
                    w.unimplemented();
                }
            });
        }
    }

    fn write_trait(&self, w: &mut CodeWriter<'_>) {
        self.write_method(false, w);
    }

    fn write_impl(&self, w: &mut CodeWriter<'_>) {
        self.write_method(true, w);
    }
}

struct ClientTraitGen<'a> {
    proto: &'a ServiceDescriptorProto,
    methods: Vec<MethodGen<'a>>,
    base_name: String,
}

impl<'a> ClientTraitGen<'a> {
    fn new(
        proto: &'a ServiceDescriptorProto,
        file: &FileDescriptorProto,
        root_scope: &'a RootScope<'_>,
    ) -> ClientTraitGen<'a> {
        let methods = proto
            .get_method()
            .iter()
            .map(|m| MethodGen::new(m, root_scope))
            .collect();

        let base = protobuf::descriptorx::proto_path_to_rust_mod(file.get_name());

        ClientTraitGen {
            proto,
            methods,
            base_name: base,
        }
    }

    fn service_name(&self) -> String {
        util::to_camel_case(self.proto.get_name())
    }

    fn trait_name(&self) -> String {
        format!("{}Trait", self.client_name())
    }

    fn client_name(&self) -> String {
        format!("{}Client", self.service_name())
    }

    fn write_trait(&self, w: &mut CodeWriter<'_>) {
        w.pub_trait_extend(self.trait_name().as_str(), "Clone + Send + Sync", |w| {
            // methods that go inside the trait

            for method in &self.methods {
                w.write_line("");
                method.write_trait(w);
            }
        });
    }

    fn write_impl(&self, w: &mut CodeWriter<'_>) {
        let type_name = format!("super::{}_grpc::{}", self.base_name, self.client_name());
        w.impl_for_block(self.trait_name(), type_name, |w| {
            for method in &self.methods {
                w.write_line("");
                method.write_impl(w);
            }
        });
    }

    fn write(&self, w: &mut CodeWriter<'_>) {
        // Client trait definition
        self.write_trait(w);
        w.write_line("");

        // Impl block for client trait
        self.write_impl(w);
    }
}

fn gen_file(
    file: &FileDescriptorProto,
    root_scope: &RootScope<'_>,
) -> Option<compiler_plugin::GenResult> {
    if file.get_service().is_empty() {
        return None;
    }

    let base = protobuf::descriptorx::proto_path_to_rust_mod(file.get_name());

    let mut v = Vec::new();
    {
        let mut w = CodeWriter::new(&mut v);
        w.write_generated();
        w.write_line("#![allow(unused_variables)]");
        w.write_line("use futures::Future;");
        w.write_line("");
        for service in file.get_service() {
            w.write_line("");
            ClientTraitGen::new(service, file, root_scope).write(&mut w);
        }
    }

    Some(compiler_plugin::GenResult {
        name: base + "_client.rs",
        content: v,
    })
}

pub fn gen(
    file_descriptors: &[FileDescriptorProto],
    files_to_generate: &[&str],
) -> Vec<compiler_plugin::GenResult> {
    let files_map: HashMap<&str, &FileDescriptorProto> =
        file_descriptors.iter().map(|f| (f.get_name(), f)).collect();

    let root_scope = RootScope { file_descriptors };

    let mut results = Vec::new();

    for file_name in files_to_generate {
        // not all files need client stubs, some are simple protobufs, no service
        let temp1 = file_name.to_string();
        let file = files_map[&temp1[..]];

        if file.get_service().is_empty() {
            continue;
        }

        results.extend(gen_file(file, &root_scope).into_iter());
    }

    results
}
