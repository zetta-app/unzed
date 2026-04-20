// Telemetry removed for privacy. This crate is a no-op shim so that
// telemetry::event!() call sites compile without modification.

#[macro_export]
macro_rules! event {
    ($name:expr) => {{
        let _ = $name;
    }};
    ($name:expr, $($key:ident $(= $value:expr)?),+ $(,)?) => {{
        let _ = $name;
        $($crate::suppress_unused!($key $(= $value)?);)+
    }};
}

#[macro_export]
macro_rules! suppress_unused {
    ($key:ident) => {
        let _ = &$key;
    };
    ($key:ident = $value:expr) => {
        let _ = &$value;
    };
}

// Keep this around as a no-op so old code referencing it still compiles.
#[macro_export]
macro_rules! serialize_property {
    ($key:ident) => {
        $key
    };
    ($key:ident = $value:expr) => {
        $value
    };
}
