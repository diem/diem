// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan_reporting_new::diagnostic::Severity;

//**************************************************************************************************
// Main types
//**************************************************************************************************

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub(crate) struct DiagnosticInfo {
    pub(crate) severity: Severity,
    category: Category,
    code: u8,
    message: &'static str,
}

pub(crate) trait DiagnosticCode: Copy {
    const SEVERITY: Severity;
    const CATEGORY: Category;

    fn code_and_message(self) -> (u8, &'static str);

    fn into_info(self) -> DiagnosticInfo {
        let severity = Self::SEVERITY;
        let category = Self::CATEGORY;
        let (code, message) = self.code_and_message();
        DiagnosticInfo {
            severity,
            category,
            code,
            message,
        }
    }
}

//**************************************************************************************************
// Categories and Codes
//**************************************************************************************************

macro_rules! codes {
    ($(severity:$sev:ident, category:$cat:ident=$cat_num:literal, name:$name:ident, codes:[
        $($code:ident = $code_num:literal: $code_msg:literal),*

    ]),*) => {
        #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
        #[repr(u8)]
        pub enum Category {
            $($cat = $cat_num,)*
        }

        $(
            #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
            #[repr(u8)]
            pub enum $name {
                $($code = $code_num,)*
            }

            impl DiagnosticCode for $name {
                const SEVERITY: Severity = Severity::$sev;
                const CATEGORY: Category = {
                    // hacky check that $cat_num <= 99
                    let cat_is_leq_99 = $cat_num <= 99;
                    ["Diagnostic Category must be a u8 <= 99"][!cat_is_leq_99 as usize];
                    Category::$cat
                };

                fn code_and_message(self) -> (u8, &'static str) {
                    let code = self as u8;
                    debug_assert!(code > 0);
                    let message = match self {
                        $(Self::$code => $code_msg,)*
                    };
                    (code, message)
                }
            }
        )*

    };
}

codes!(
    severity: Error, category: NameResolution=2, name: NameResolutionError, codes: [
        UnboundModule = 1:  "unbound module"
    ]
);

//**************************************************************************************************
// impls
//**************************************************************************************************

impl DiagnosticInfo {
    pub fn render(self) -> (/* code */ String, /* message */ &'static str) {
        let Self {
            severity,
            category,
            code,
            message,
        } = self;
        let sev_prefix = match severity {
            Severity::Error => "E",
            Severity::Warning => "W",
            _ => unimplemented!(),
        };
        let cat_prefix: u8 = category as u8;
        debug_assert!(cat_prefix <= 99);
        let string_code = format!("{}{:02}{:03}", sev_prefix, cat_prefix, code);
        (string_code, message)
    }
}
