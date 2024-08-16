use alloc::string::ToString;
use alloc::vec::Vec;
use core::arch::global_asm;

use hyperlight_common::flatbuffer_wrappers::function_call::{FunctionCall, FunctionCallType};
use hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterValue, ReturnType, ReturnValue,
};
use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;

use crate::error::{HyperlightGuestError, Result};
use crate::flatbuffer_utils::get_flatbuffer_result_from_int;
use crate::host_error::check_for_host_error;
use crate::host_functions::validate_host_function_call;
use crate::shared_input_data::try_pop_shared_input_data_into;
use crate::shared_output_data::push_shared_output_data;
use crate::{OUTB_PTR, OUTB_PTR_WITH_CONTEXT, P_PEB, RUNNING_IN_HYPERLIGHT};

pub enum OutBAction {
    Log = 99,
    CallFunction = 101,
    Abort = 102,
}

pub fn get_host_value_return_as_void() -> Result<()> {
    let return_value = try_pop_shared_input_data_into::<ReturnValue>()
        .expect("Unable to deserialize a return value from host");
    if let ReturnValue::Void = return_value {
        Ok(())
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Host return value was not void as expected".to_string(),
        ))
    }
}

pub fn get_host_value_return_as_int() -> Result<i32> {
    let return_value = try_pop_shared_input_data_into::<ReturnValue>()
        .expect("Unable to deserialize return value from host");

    // check that return value is an int and return
    if let ReturnValue::Int(i) = return_value {
        Ok(i)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Host return value was not an int as expected".to_string(),
        ))
    }
}

pub fn get_host_value_return_as_uint() -> Result<u32> {
    let return_value = try_pop_shared_input_data_into::<ReturnValue>()
        .expect("Unable to deserialize return value from host");

    // check that return value is an int and return
    if let ReturnValue::UInt(ui) = return_value {
        Ok(ui)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Host return value was not a uint as expected".to_string(),
        ))
    }
}

pub fn get_host_value_return_as_long() -> Result<i64> {
    let return_value = try_pop_shared_input_data_into::<ReturnValue>()
        .expect("Unable to deserialize return value from host");

    // check that return value is an int and return
    if let ReturnValue::Long(l) = return_value {
        Ok(l)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Host return value was not a long as expected".to_string(),
        ))
    }
}

pub fn get_host_value_return_as_ulong() -> Result<u64> {
    let return_value = try_pop_shared_input_data_into::<ReturnValue>()
        .expect("Unable to deserialize return value from host");

    // check that return value is an int and return
    if let ReturnValue::ULong(ul) = return_value {
        Ok(ul)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Host return value was not a ulong as expected".to_string(),
        ))
    }
}

// TODO: Make this generic, return a Result<T, ErrorCode>

pub fn get_host_value_return_as_vecbytes() -> Result<Vec<u8>> {
    let return_value = try_pop_shared_input_data_into::<ReturnValue>()
        .expect("Unable to deserialize return value from host");

    // check that return value is an Vec<u8> and return
    if let ReturnValue::VecBytes(v) = return_value {
        Ok(v)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Host return value was not an VecBytes as expected".to_string(),
        ))
    }
}

// TODO: Make this generic, return a Result<T, ErrorCode> this should allow callers to call this function and get the result type they expect
// without having to do the conversion themselves

pub fn call_host_function(
    function_name: &str,
    parameters: Option<Vec<ParameterValue>>,
    return_type: ReturnType,
) -> Result<()> {
    let host_function_call = FunctionCall::new(
        function_name.to_string(),
        parameters,
        FunctionCallType::Host,
        return_type,
    );

    validate_host_function_call(&host_function_call)?;

    let host_function_call_buffer: Vec<u8> = host_function_call
        .try_into()
        .expect("Unable to serialize host function call");

    push_shared_output_data(host_function_call_buffer)?;

    outb(OutBAction::CallFunction as u16, 0);

    Ok(())
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        if RUNNING_IN_HYPERLIGHT {
            hloutb(port, value);
        } else if let Some(outb_func) = OUTB_PTR_WITH_CONTEXT {
            if let Some(peb_ptr) = P_PEB {
                outb_func((*peb_ptr).pOutbContext, port, value);
            }
        } else if let Some(outb_func) = OUTB_PTR {
            outb_func(port, value);
        }

        check_for_host_error();
    }
}

extern "win64" {
    fn hloutb(port: u16, value: u8);
}

pub fn print_output_as_guest_function(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        call_host_function(
            "HostPrint",
            Some(Vec::from(&[ParameterValue::String(message.to_string())])),
            ReturnType::Int,
        )?;
        let res_i = get_host_value_return_as_int()?;
        Ok(get_flatbuffer_result_from_int(res_i))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Wrong Parameters passed to print_output_as_guest_function".to_string(),
        ))
    }
}

// port: RCX(cx), value: RDX(dl)
global_asm!(
    ".global hloutb
        hloutb:
            xor rax, rax
            mov al, dl
            mov dx, cx
            out dx, al
            ret"
);
