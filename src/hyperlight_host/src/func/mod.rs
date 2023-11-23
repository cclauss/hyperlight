use crate::Result;
/// Context structures used to allow the user to call one or more guest
/// functions on the same Hyperlight sandbox instance, all from within the
/// same state and mutual exclusion context.
pub mod call_ctx;
/// Definitions for common functions to be exposed in the guest
pub mod exports;
/// Functionality to dispatch a call from the host to the guest
mod guest_dispatch;
/// Functionality to check for errors after a guest call
mod guest_err;
/// Definitions and functionality to enable guest-to-host function calling,
/// also called "host functions"
///
/// This module includes functionality to do the following
///
/// - Define several prototypes for what a host function must look like,
/// including the number of arguments (arity) they can have, supported argument
/// types, and supported return types
/// - Registering host functions to be callable by the guest
/// - Dynamically dispatching a call from the guest to the appropriate
/// host function
pub mod host_functions;
/// Definitions and functionality for supported parameter types
pub(crate) mod param_type;
/// Definitions and functionality for supported return types
pub mod ret_type;

use hyperlight_flatbuffers::flatbuffer_wrappers::function_types::{ParameterValue, ReturnValue};
pub use param_type::SupportedParameterType;
pub use ret_type::SupportedReturnType;
use std::sync::{Arc, Mutex};
use tracing::instrument;
use tracing::Span;

type HLFunc<'a> =
    Arc<Mutex<Box<dyn FnMut(Vec<ParameterValue>) -> Result<ReturnValue> + 'a + Send>>>;

/// Generic HyperlightFunction
#[derive(Clone)]
pub struct HyperlightFunction<'a>(HLFunc<'a>);

impl<'a> HyperlightFunction<'a> {
    #[instrument(skip_all, parent = Span::current(), level= "Trace")]
    pub(crate) fn new<F>(f: F) -> Self
    where
        F: FnMut(Vec<ParameterValue>) -> Result<ReturnValue> + 'a + Send,
    {
        Self(Arc::new(Mutex::new(Box::new(f))))
    }

    #[instrument(err(Debug), skip_all, parent = Span::current(), level= "Trace")]
    pub(crate) fn call(&self, args: Vec<ParameterValue>) -> Result<ReturnValue> {
        let mut f = self.0.lock().unwrap();
        f(args)
    }
}

/// Re-export for `get_stack_boundary` function
pub use exports::get_stack_boundary;
/// Re-export for `HostFunction0` trait
pub use host_functions::HostFunction0;
/// Re-export for `HostFunction1` trait
pub use host_functions::HostFunction1;
