// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{CodeOffset, FunctionDefinitionIndex, TableIndex},
    IndexKind,
};
use move_core_types::{
    language_storage::ModuleId,
    vm_status::{self, StatusCode, StatusType, VMStatus},
};
use std::fmt;

pub type VMResult<T> = ::std::result::Result<T, VMError>;
pub type BinaryLoaderResult<T> = ::std::result::Result<T, PartialVMError>;
pub type PartialVMResult<T> = ::std::result::Result<T, PartialVMError>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Location {
    Undefined,
    Script,
    Module(ModuleId),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VMError {
    major_status: StatusCode,
    sub_status: Option<u64>,
    message: Option<String>,
    location: Location,
    indices: Vec<(IndexKind, TableIndex)>,
    offsets: Vec<(FunctionDefinitionIndex, CodeOffset)>,
}

impl VMError {
    pub fn into_vm_status(self) -> VMStatus {
        let VMError {
            major_status,
            sub_status,
            location,
            mut offsets,
            ..
        } = self;
        match (major_status, sub_status, location) {
            (StatusCode::EXECUTED, sub_status, _) => {
                debug_assert!(sub_status.is_none());
                VMStatus::Executed
            }
            (StatusCode::ABORTED, Some(code), Location::Script) => {
                VMStatus::MoveAbort(vm_status::AbortLocation::Script, code)
            }
            (StatusCode::ABORTED, Some(code), Location::Module(id)) => {
                VMStatus::MoveAbort(vm_status::AbortLocation::Module(id), code)
            }

            (StatusCode::ABORTED, sub_status, location) => {
                debug_assert!(
                    false,
                    "Expected a code and module/script location with ABORTED, but got {:?} and {}",
                    sub_status, location
                );
                VMStatus::Error(StatusCode::ABORTED)
            }

            // TODO Errors for OUT_OF_GAS do not always have index set
            (major_status, sub_status, location)
                if major_status.status_type() == StatusType::Execution =>
            {
                debug_assert!(
                    offsets.len() == 1,
                    "Unexpected offsets. major_status: {:?}\
                    sub_status: {:?}\
                    location: {:?}\
                    offsets: {:#?}",
                    major_status,
                    sub_status,
                    location,
                    offsets
                );
                let abort_location = match location {
                    Location::Script => vm_status::AbortLocation::Script,
                    Location::Module(id) => vm_status::AbortLocation::Module(id),
                    Location::Undefined => {
                        return VMStatus::Error(major_status);
                    }
                };
                let (function, code_offset) = match offsets.pop() {
                    None => {
                        return VMStatus::Error(major_status);
                    }
                    Some((fdef_idx, code_offset)) => (fdef_idx.0, code_offset),
                };
                VMStatus::ExecutionFailure {
                    status_code: major_status,
                    location: abort_location,
                    function,
                    code_offset,
                }
            }

            (major_status, _, _) => VMStatus::Error(major_status),
        }
    }

    pub fn major_status(&self) -> StatusCode {
        self.major_status
    }

    pub fn sub_status(&self) -> Option<u64> {
        self.sub_status
    }

    pub fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn indices(&self) -> &Vec<(IndexKind, TableIndex)> {
        &self.indices
    }

    pub fn offsets(&self) -> &Vec<(FunctionDefinitionIndex, CodeOffset)> {
        &self.offsets
    }

    pub fn status_type(&self) -> StatusType {
        self.major_status.status_type()
    }

    pub fn all_data(
        self,
    ) -> (
        StatusCode,
        Option<u64>,
        Option<String>,
        Location,
        Vec<(IndexKind, TableIndex)>,
        Vec<(FunctionDefinitionIndex, CodeOffset)>,
    ) {
        let VMError {
            major_status,
            sub_status,
            message,
            location,
            indices,
            offsets,
        } = self;
        (
            major_status,
            sub_status,
            message,
            location,
            indices,
            offsets,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PartialVMError {
    major_status: StatusCode,
    sub_status: Option<u64>,
    message: Option<String>,
    indices: Vec<(IndexKind, TableIndex)>,
    offsets: Vec<(FunctionDefinitionIndex, CodeOffset)>,
}

impl PartialVMError {
    pub fn all_data(
        self,
    ) -> (
        StatusCode,
        Option<u64>,
        Option<String>,
        Vec<(IndexKind, TableIndex)>,
        Vec<(FunctionDefinitionIndex, CodeOffset)>,
    ) {
        let PartialVMError {
            major_status,
            sub_status,
            message,
            indices,
            offsets,
        } = self;
        (major_status, sub_status, message, indices, offsets)
    }

    pub fn finish(self, location: Location) -> VMError {
        let PartialVMError {
            major_status,
            sub_status,
            message,
            indices,
            offsets,
        } = self;
        VMError {
            major_status,
            sub_status,
            message,
            location,
            indices,
            offsets,
        }
    }

    pub fn new(major_status: StatusCode) -> Self {
        Self {
            major_status,
            sub_status: None,
            message: None,
            indices: vec![],
            offsets: vec![],
        }
    }

    pub fn major_status(&self) -> StatusCode {
        self.major_status
    }

    pub fn with_sub_status(self, sub_status: u64) -> Self {
        debug_assert!(self.sub_status.is_none());
        Self {
            sub_status: Some(sub_status),
            ..self
        }
    }

    pub fn with_message(self, message: String) -> Self {
        debug_assert!(self.message.is_none());
        Self {
            message: Some(message),
            ..self
        }
    }

    pub fn at_index(self, kind: IndexKind, index: TableIndex) -> Self {
        let mut indices = self.indices;
        indices.push((kind, index));
        Self { indices, ..self }
    }

    pub fn at_indices(self, additional_indices: Vec<(IndexKind, TableIndex)>) -> Self {
        let mut indices = self.indices;
        indices.extend(additional_indices);
        Self { indices, ..self }
    }

    pub fn at_code_offset(self, function: FunctionDefinitionIndex, offset: CodeOffset) -> Self {
        let mut offsets = self.offsets;
        offsets.push((function, offset));
        Self { offsets, ..self }
    }

    pub fn at_code_offsets(
        self,
        additional_offsets: Vec<(FunctionDefinitionIndex, CodeOffset)>,
    ) -> Self {
        let mut offsets = self.offsets;
        offsets.extend(additional_offsets);
        Self { offsets, ..self }
    }

    /// Append the message `message` to the message field of the VM status, and insert a seperator
    /// if the original message is non-empty.
    pub fn append_message_with_separator(
        self,
        separator: char,
        additional_message: String,
    ) -> Self {
        let message = match self.message {
            Some(mut msg) => {
                if !msg.is_empty() {
                    msg.push(separator);
                }
                msg.push_str(&additional_message);
                msg
            }
            None => additional_message,
        };
        Self {
            message: Some(message),
            ..self
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Undefined => write!(f, "UNDEFINED"),
            Location::Script => write!(f, "Script"),
            Location::Module(id) => write!(f, "Module {:?}", id),
        }
    }
}

impl fmt::Display for PartialVMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut status = format!("PartialVMError with status {:#?}", self.major_status);

        if let Some(sub_status) = self.sub_status {
            status = format!("{} with sub status {}", status, sub_status);
        }

        if let Some(msg) = &self.message {
            status = format!("{} and message {}", status, msg);
        }

        for (kind, index) in &self.indices {
            status = format!("{} at index {} for {}", status, index, kind);
        }
        for (fdef, code_offset) in &self.offsets {
            status = format!(
                "{} at code offset {} in function definition {}",
                status, code_offset, fdef
            );
        }

        write!(f, "{}", status)
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut status = format!("VMError with status {:#?}", self.major_status);

        if let Some(sub_status) = self.sub_status {
            status = format!("{} with sub status {}", status, sub_status);
        }

        status = format!("{} at location {}", status, self.location);

        if let Some(msg) = &self.message {
            status = format!("{} and message {}", status, msg);
        }

        for (kind, index) in &self.indices {
            status = format!("{} at index {} for {}", status, index, kind);
        }
        for (fdef, code_offset) in &self.offsets {
            status = format!(
                "{} at code offset {} in function definition {}",
                status, code_offset, fdef
            );
        }

        write!(f, "{}", status)
    }
}

////////////////////////////////////////////////////////////////////////////
/// Conversion functions from internal VM statuses into external VM statuses
////////////////////////////////////////////////////////////////////////////
impl From<VMError> for VMStatus {
    fn from(vm_error: VMError) -> VMStatus {
        vm_error.into_vm_status()
    }
}

pub fn vm_status_of_result<T>(result: VMResult<T>) -> VMStatus {
    match result {
        Ok(_) => VMStatus::Executed,
        Err(err) => err.into_vm_status(),
    }
}

pub fn offset_out_of_bounds(
    status: StatusCode,
    kind: IndexKind,
    target_offset: usize,
    target_pool_len: usize,
    cur_function: FunctionDefinitionIndex,
    cur_bytecode_offset: CodeOffset,
) -> PartialVMError {
    let msg = format!(
        "Index {} out of bounds for {} at bytecode offset {} in function {} while indexing {}",
        target_offset, target_pool_len, cur_bytecode_offset, cur_function, kind
    );
    PartialVMError::new(status)
        .with_message(msg)
        .at_code_offset(cur_function, cur_bytecode_offset)
}

pub fn bounds_error(
    status: StatusCode,
    kind: IndexKind,
    idx: TableIndex,
    len: usize,
) -> PartialVMError {
    let msg = format!(
        "Index {} out of bounds for {} while indexing {}",
        idx, len, kind
    );
    PartialVMError::new(status)
        .at_index(kind, idx)
        .with_message(msg)
}

pub fn verification_error(status: StatusCode, kind: IndexKind, idx: TableIndex) -> PartialVMError {
    PartialVMError::new(status).at_index(kind, idx)
}

impl std::error::Error for PartialVMError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
