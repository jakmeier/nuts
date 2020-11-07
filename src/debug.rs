#![allow(unused_macros)]
/* log_print, to println! or web console log (nothing in release mode) */

#[cfg(debug_assertions)]
#[cfg(all(feature = "web-debug", target_arch = "wasm32"))]
macro_rules! log_print {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[cfg(debug_assertions)]
#[cfg(not(all(feature = "web-debug", target_arch = "wasm32")))]
macro_rules! log_print {
    ( $( $t:tt )* ) => {
        println!( $( $t )* );
    }
}

#[cfg(not(debug_assertions))]
macro_rules! log_print {
    ( $( $t:tt )* ) => {};
}

/* debug_print, to web console debug or (for now) println (could be extended to be configurable) */

#[cfg(debug_assertions)]
#[cfg(all(feature = "web-debug", target_arch = "wasm32"))]
macro_rules! debug_print {
    ( $( $t:tt )* ) => {
        web_sys::console::debug_1(&format!( $( $t )* ).into());
    }
}

#[cfg(debug_assertions)]
#[cfg(not(all(feature = "web-debug", target_arch = "wasm32")))]
macro_rules! debug_print {
    ( $( $t:tt )* ) => {
        println!( $( $t )* );
    }
}

#[cfg(not(debug_assertions))]
macro_rules! debug_print {
    ( $( $t:tt )* ) => {};
}

#[derive(Clone, Copy)]
pub(crate) struct DebugTypeName(
    #[cfg(debug_assertions)] pub(crate) &'static str,
    #[cfg(not(debug_assertions))] (),
);

impl DebugTypeName {
    pub fn new<MSG: std::any::Any>() -> Self {
        Self(
            #[cfg(debug_assertions)]
            std::any::type_name::<MSG>(),
            #[cfg(not(debug_assertions))]
            (),
        )
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for DebugTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
