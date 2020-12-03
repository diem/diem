// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! log {
    // Entry, Log Level + stuff
    ($level:expr, $($args:tt)+) => {{
        let metadata = $crate::Metadata::new(
            $level,
            module_path!().split("::").next().unwrap(),
            module_path!(),
            file!(),
            line!(),
            concat!(file!(), ':', line!()),
        );

        if metadata.enabled() {
            $crate::Event::dispatch(
                &metadata,
                $crate::fmt_args!($($args)+),
                $crate::schema!($($args)+),
            );
        }
    }};
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::log!($crate::Level::Trace, $($arg)+)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::log!($crate::Level::Debug, $($arg)+)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::log!($crate::Level::Info, $($arg)+)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::log!($crate::Level::Warn, $($arg)+)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::log!($crate::Level::Error, $($arg)+)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! schema {
    //
    // base case
    //
    (@ { $(,)* $($val:expr),* $(,)* } $(,)*) => {
        &[ $($val),* ]
    };

    //
    // recursive cases
    //

    // format args
    (@ { $(,)* $($out:expr),* }, $template:literal, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),* }
        )
    };
    (@ { $(,)* $($out:expr),* }, $template:literal) => {
        $crate::schema!(
            @ { $($out),* }
        )
    };

    // Identifier Keys
    (@ { $(,)* $($out:expr),* }, $($k:ident).+ = $val:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($crate::__log_stringify!($($k).+), $crate::Value::from_serde(&$val)) },
            $($args)*
        )
    };

    (@ { $(,)* $($out:expr),* }, $($k:ident).+ = $val:expr) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($crate::__log_stringify!($($k).+), $crate::Value::from_serde(&$val)) },
        )
    };

    // Identifier Keys debug
    (@ { $(,)* $($out:expr),* }, $($k:ident).+ = ?$val:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($crate::__log_stringify!($($k).+), $crate::Value::from_debug(&$val)) },
            $($args)*
        )
    };

    (@ { $(,)* $($out:expr),* }, $($k:ident).+ = ?$val:expr) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($crate::__log_stringify!($($k).+), $crate::Value::from_debug($val)) },
        )
    };

    // Identifier Keys display
    (@ { $(,)* $($out:expr),* }, $($k:ident).+ = %$val:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($crate::__log_stringify!($($k).+), $crate::Value::from_display(&$val)) },
            $($args)*
        )
    };

    (@ { $(,)* $($out:expr),* }, $($k:ident).+ = %$val:expr) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($crate::__log_stringify!($($k).+), $crate::Value::from_display(&$val)) },
        )
    };

    // Literal Keys
    (@ { $(,)* $($out:expr),* }, $k:literal = $val:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($k, $crate::Value::from_serde(&$val)) },
            $($args)*
        )
    };

    (@ { $(,)* $($out:expr),* }, $k:literal = $val:expr) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($k, $crate::Value::from_serde(&$val)) },
        )
    };

    // Literal Keys debug
    (@ { $(,)* $($out:expr),* }, $k:literal = ?$val:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($k, $crate::Value::from_debug(&$val)) },
            $($args)*
        )
    };

    (@ { $(,)* $($out:expr),* }, $k:literal = ?$val:expr) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($k, $crate::Value::from_debug(&$val)) },
        )
    };

    // Literal Keys display
    (@ { $(,)* $($out:expr),* }, $k:literal = %$val:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($k, $crate::Value::from_display(&$val)) },
            $($args)*
        )
    };

    (@ { $(,)* $($out:expr),* }, $k:literal = %$val:expr) => {
        $crate::schema!(
            @ { $($out),*, &$crate::KeyValue::new($k, $crate::Value::from_display(&$val)) },
        )
    };

    // Lone Schemas
    (@ { $(,)* $($out:expr),* }, $schema:expr, $($args:tt)*) => {
        $crate::schema!(
            @ { $($out),*, &$schema },
            $($args)*
        )
    };
    (@ { $(,)* $($out:expr),* }, $schema:expr) => {
        $crate::schema!(
            @ { $($out),*, &$schema },
        )
    };

    //
    // entry
    //
    ($($args:tt)*) => {
        $crate::schema!(@ { }, $($args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! fmt_args {
    //
    // base case
    //
    () => {
        None
    };

    // format args
    ($template:literal, $($args:tt)*) => {
        Some(::std::format_args!($template, $($args)*))
    };
    ($template:literal) => {
        Some(::std::format_args!($template))
    };

    // Identifier Keys
    ($($k:ident).+ = $val:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($($k:ident).+ = $val:expr) => {
        $crate::fmt_args!()
    };
    // Identifier Keys with Debug
    ($($k:ident).+ = ?$val:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($($k:ident).+ = ?$val:expr) => {
        $crate::fmt_args!()
    };
    // Identifier Keys with Display
    ($($k:ident).+ = %$val:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($($k:ident).+ = %$val:expr) => {
        $crate::fmt_args!()
    };

    // Literal Keys
    ($k:literal = $val:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($k:literal = $val:expr) => {
        $crate::fmt_args!()
    };
    // Literal Keys with Debug
    ($k:literal = ?$val:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($k:literal = ?$val:expr) => {
        $crate::fmt_args!()
    };
    // Literal Keys with Display
    ($k:literal = %$val:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($k:literal = %$val:expr) => {
        $crate::fmt_args!()
    };

    // Lone Schemas
    ($schema:expr, $($args:tt)*) => {
        $crate::fmt_args!(
            $($args)*
        )
    };
    ($schema:expr) => {
        $crate::fmt_args!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_stringify {
    ($s:expr) => {
        stringify!($s)
    };
}
