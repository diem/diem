// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(deprecated)]
mod syntax;

/// # Grammar
/// ## Identifiers
/// ```text
/// f ∈ FieldName     // [a-zA-Z$_][a-zA-Z0-9$_]*
/// p ∈ ProcedureName // [a-zA-Z$_][a-zA-Z0-9$_]*
/// m ∈ ModuleName    // [a-zA-Z$_][a-zA-Z0-9$_]*
/// n ∈ StructName    // [a-zA-Z$_][a-zA-Z0-9$_]*
/// x ∈ Var           // [a-zA-Z$_][a-zA-Z0-9$_]*
/// ```
///
/// ## Types
/// ```text
/// k ∈ Kind ::=
///   | R // Linear resource struct value. Must be used, cannot be copied
///   | V // Non-resource struct value. Can be silently discarded, can be copied
///
/// g ∈ GroundType ::=
///   | bool
///   | u64       // unsigned 64 bit integer
///   | address   // 32 byte account address
///   | bytearray // immutable, arbitrarily sized array of bytes
///
/// d ∈ ModuleAlias ::=
///   | m         // module name that is an alias to a declared module, addr.m
///   | Self      // current module
///
/// t ∈ BaseType ::=
///   | g     // ground type
///   | k#d.n // struct 'n' declared in the module referenced by 'd' with kind 'k'
///           // the kind 'k' cannot differ from the declared kind
///
/// 𝛕 ∈ Type ::=
///   | t      // base type
///   | &t     // immutable reference to a base type
///   | &mut t // mutable reference to a base type
///
/// 𝛕-list ∈ [Type] ::=
///   | unit            // empty type list.
///                     // in the actual syntax, it is represented by the abscense of a type
///   | 𝛕_1 * ... * 𝛕_j // 'j' >= 1. list of multiple types. used for multiple return values
/// ```
///
/// ## Values
/// ```text
/// u ∈ Unsigned64        // Unsigned, 64-bit Integer
/// addr ∈ AccountAddress // addresses of blockchain accounts
/// bytes ∈ ByteArray     // byte array of arbitrary length
/// v ∈ Value ::=
///   | true
///   | false
///   | u        // u64 literal
///   | 0xaddr   // 32 byte address literal
///   | b"bytes" // arbitrary length bytearray literal
/// ```
///
/// ## Expressions
/// ```text
/// o ∈ VarOp ::=
///   | copy(x) // returns value bound to 'x'
///   | move(x) // moves the value out of 'x', i.e. returns the value and makes 'x' unusable
///
/// r ∈ ReferenceOp ::=
///   | &x        // type: 't -> &mut t'
///               // creates an exclusive, mutable reference to a local
///   | &e.f      // type: '&t_1 -> &t_2' or '&mut t_1 -> &mut t_2'
///               // borrows a new reference to field 'f' of the struct 't_1'. inherits exclusive or shared from parent
///               // 't_1' must be a struct declared in the current module, i.e. 'f' is "private"
///   | *e        // type: '&t -> t' or '&mut t -> t'. Dereferencing. Not valid for resources
///
/// e ∈ Exp ::=
///   | v
///   | o
///   | r
///   | n { f_1: e_1, ... , f_j: e_j } // type: '𝛕-list -> k#Self.n'
///                                    // "constructor" for 'n'
///                                    // "packs" the values, binding them to the fields, and creates a new instance of 'n'
///                                    // 'n' must be declared in the current module
///   // boolean operators
///   | !e_1
///   | e_1 || e_2
///   | e_1 && e_2
///   // u64 operators
///   | e_1 >= e_2
///   | e_1 <= e_2
///   | e_1 > e_2
///   | e_1 < e_2
///   | e_1 + e_2
///   | e_1 - e_2
///   | e_1 * e_2
///   | e_1 / e_2
///   | e_1 % e_2
///   | e_1 ^ e_2
///   | e_1 | e_2
///   | e_g & e_2
///   // operators over any ground type
///   | e_1 == e_2
///   | e_1 != e_2
/// ```
/// ## Commands
/// ```text
/// // module operators are available only inside the module that declares n.
/// mop ∈ ModuleOp ::=
///   | move_to_sender<n>(e) // type: 'R#Self.n -> unit'
///                          // publishes resource struct 'n' under sender's address
///                          // fails if there is already a resource present for 'Self.n'
///   | move_from<n>(e)      // type: 'address -> R#Self.n'
///                          // removes the resource struct 'n' at the specified address
///                          // fails if there is no resource present for 'Self.n'
///   | borrow_global<n>(e)  // type: 'address -> &mut R#Self.n'
///                          // borrows a mutable reference to the resource struct 'n' at the specified address
///                          // fails if there is no resource
///                          // fails if it is already borrowed in this transaction's execution
///   | exists<n>(e)         // type: 'address -> bool', s.t. 'n' is a resource struct
///                          // returns 'true' if the resource struct 'n' at the specified address exists
///                          // returns 'false' otherwise
///
/// builtin ∈ Builtin ::=
///   | create_account(e)         // type: 'addr -> unit'
///                               // creates new account at the specified address, failing if it already exists
///   | release(e)                // type: '&t -> unit' or '&mut t -> unit'
///                               // releases the reference given
///   | freeze(x)                 // type: '&mut t -> &t'
///                               // coerce a mutable reference to an immutable reference
///   | get_txn_gas_unit_price()  // type: 'unit -> u64'
///                               // gives the price specified per gas unit
///   | get_txn_max_gas_units()   // type: 'unit -> u64'
///                               // gives the tranaction's maximum amount of usable gas units
///   | get_txn_public_key()      // type: 'unit -> bytearray'
///                               // gives the transaction's public key
///   | get_txn_sender()          // type: 'unit -> address'
///                               // gives the transaction's sender's account address
///   | get_txn_sequence_number() // type: 'unit -> u64'
///                               // gives the sequence number for this transaction
///   | get_gas_remaining()       // type: 'unit -> u64'
///                               // gives the amount of gas gas units remaining before the transaction execution will be forced to halt execution
///
/// call ∈ Call ::=
///   | mop
///   | builtin
///   | d.p(e_1, ..., e_j) // procedure 'p' defined in the module referenced by 'd'
///
/// c ∈ Cmd ::=
///   | x = e                               // assign the result of evaluating 'e' to 'x'
///   | x_1, ..., x_j = call                // Invokes 'call', assigns result to 'x_1' to 'x_j'
///   | call                                // Invokes 'call' that has a return type of 'unit'
///   | *x = e                              // mutation, s.t. 'x: &mut t' and 'e: t' and 't' is not of resource kind
///   | assert(e_1, e_2)                    // type: 'bool * u64 -> unit'
///                                         // halts execution with error code 'e_2' if 'e_1' evaluates to 'false'
///   | break                               // exit a loop
///   | continue                            // return to the top of a loop
///   | return e_1, ..., e_n                // return values from procedure
///   | n { f_1: x_1, ... , f_j: x_j } = e  // "de-constructor" for 'n'
///                                         // "unpacks" a struct value 'e: _#Self.n'
///                                         // value for 'f_i' is bound to local 'x_i'
/// ```
///
/// ## Statements
/// ```text
/// s ∈ Stmt ::=
///   | if (e) { s_1 } else { s_2 } // conditional
///   | if (e) { s }                // conditional without else branch
///   | while (e) { s }             // while loop
///   | loop { s }                  // loops forever
///   | c;                          // command
///   | s_1 s_2                     // sequencing
/// ```
///
/// ## Imports
///```text
/// idecl ∈ Import ::=
///   | import addr.m_1 as m_2; // imports 'addr.m_1' with the alias 'm_2'
///   | import addr.m_1;        // imports 'addr.m_1' with the alias 'm_1'
/// ```
/// ## Modules
/// ```text
/// sdecl ∈ StructDecl ::=
///   | resource n { f_1: t_1, ..., f_j: t_j } // declaration of a resource struct
///   | struct n { f_1: t_1, ..., f_j: t_j }   // declaration of a non-resource (value) struct
///                                            // s.t. any 't_i' is not of resource kind
///
/// body ∈ ProcedureBody ::=
///  | let x_1; ... let x_j; s // The locals declared in this procedure, and the code for that procedure
///
/// pdecl ∈ ProcedureDecl ::=
///   | (public?) p(x_1: 𝛕_1, ..., x_j: 𝛕_j): 𝛕-list { body } // declaration of a defined procedure
///                                                          // the procedure may be public, or internal to the module
///   | native (public?) p(x_1: 𝛕_1, ..., x_j: 𝛕_j): 𝛕-list; // declaration of a native procedure
///                                                         // the implementation is provided by the VM
///                                                         // the procedure may be public, or internal to the module
///
/// mdecl ∈ ModuleDecl ::=
///   | module m { idecl_1 ... idecl_i sdecl_1 ... sdecl_j pdecl_1 ... pdecl_k }
/// ```
///
/// ## Transaction Scripts
/// ```text
/// TransactionScript ::=
///   // declaration of the transaction scripts procedure
///   // the 'main' procedure must be 'public' and any parameters must have a ground type
///   | idecl_1 ... idecl_i public main(x_1: g_1, ..., x_j: g_j) { s }
/// ```
pub mod ast;

use codespan::{ByteIndex, CodeMap, Span};
use codespan_reporting::{emit, termcolor::Buffer, Diagnostic, Label, Severity};
use failure::*;
use lalrpop_util::ParseError;
use regex::Regex;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use types::account_address::AccountAddress;

// Since lalrpop can't handle comments without a custom lexer, we somewhat hackily remove all the
// comments from the input string before passing it off to lalrpop. We only support single line
// comments for now. Will later on add in other comment types.
fn strip_comments(string: &str) -> String {
    // Remove line comments
    let line_comments = Regex::new(r"//.*(\r\n|\n|\r)").unwrap();
    line_comments.replace_all(string, "$1").into_owned()
}

/// Given the raw input of a file, creates a `Program` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_program(program_str: &str) -> Result<ast::Program> {
    let stripped_string = &strip_comments(program_str);
    let parser = syntax::ProgramParser::new();
    match parser.parse(stripped_string) {
        Ok(program) => Ok(program),
        Err(e) => handle_error(e, program_str),
    }
}

/// Given the raw input of a file, creates a `Script` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_script(script_str: &str) -> Result<ast::Script> {
    let stripped_string = &strip_comments(script_str);
    let parser = syntax::ScriptParser::new();
    match parser.parse(stripped_string) {
        Ok(script) => Ok(script),
        Err(e) => handle_error(e, script_str),
    }
}

/// Given the raw input of a file, creates a single `ModuleDefinition` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_module(modules_str: &str) -> Result<ast::ModuleDefinition> {
    let stripped_string = &strip_comments(modules_str);
    let parser = syntax::ModuleParser::new();
    match parser.parse(stripped_string) {
        Ok(module) => Ok(module),
        Err(e) => handle_error(e, modules_str),
    }
}

/// Given the raw input of a file, creates a single `Cmd` struct
/// Fails with `Err(_)` if the text cannot be parsed
pub fn parse_cmd(cmd_str: &str, _sender_address: AccountAddress) -> Result<ast::Cmd> {
    let stripped_string = &strip_comments(cmd_str);
    let parser = syntax::CmdParser::new();
    match parser.parse(stripped_string) {
        Ok(cmd) => Ok(cmd),
        Err(e) => handle_error(e, cmd_str),
    }
}

fn handle_error<'input, T, Token>(
    e: lalrpop_util::ParseError<usize, Token, &'static str>,
    code_str: &'input str,
) -> Result<T>
where
    Token: std::fmt::Display,
{
    let mut s = DefaultHasher::new();
    code_str.hash(&mut s);
    let mut code = CodeMap::new();
    code.add_filemap(s.finish().to_string().into(), code_str.to_string());
    let msg = match &e {
        ParseError::InvalidToken { location } => {
            let error =
                Diagnostic::new(Severity::Error, "Invalid Token").with_label(Label::new_primary(
                    Span::new(ByteIndex(*location as u32), ByteIndex(*location as u32)),
                ));
            let mut buffer = Buffer::no_color();
            emit(&mut buffer, &code, &error).unwrap();
            std::str::from_utf8(buffer.as_slice()).unwrap().to_string()
        }
        ParseError::UnrecognizedToken {
            token: Some((l, tok, r)),
            expected,
        } => {
            let error = Diagnostic::new(Severity::Error, format!("Unrecognized Token: {}", tok))
                .with_label(
                    Label::new_primary(Span::new(ByteIndex(*l as u32), ByteIndex(*r as u32)))
                        .with_message(format!(
                            "Expected: {}",
                            expected
                                .iter()
                                .fold(String::new(), |acc, token| format!("{} {},", acc, token))
                        )),
                );
            let mut buffer = Buffer::no_color();
            emit(&mut buffer, &code, &error).unwrap();
            std::str::from_utf8(buffer.as_slice()).unwrap().to_string()
        }
        _ => format!("{}", e),
    };
    println!("{}", msg);
    bail!("ParserError: {}", e)
}
