#![no_std]
#![no_main]
const DEFAULT_GUEST_STACK_SIZE: i32 = 65536; // default stack size
const MAX_BUFFER_SIZE: usize = 1024;
// ^^^ arbitrary value for max buffer size
// to support allocations when we'd get a
// stack overflow. This can be removed once
// we have proper stack guards in place.

extern crate alloc;

use core::ffi::c_char;
use core::hint::black_box;

use alloc::{format, string::ToString, vec::Vec};
use hyperlight_flatbuffers::flatbuffer_wrappers::{
    function_call::FunctionCall,
    function_types::{ParameterType, ParameterValue, ReturnType},
    guest_error::ErrorCode,
    guest_function_definition::GuestFunctionDefinition,
};
use hyperlight_guest::alloca::_alloca;
use hyperlight_guest::memory::hlmalloc;
use hyperlight_guest::{entrypoint::abort_with_code, entrypoint::abort_with_code_and_message};
use hyperlight_guest::{
    error::{HyperlightGuestError, Result},
    flatbuffer_utils::{
        get_flatbuffer_result_from_int, get_flatbuffer_result_from_size_prefixed_buffer,
        get_flatbuffer_result_from_string, get_flatbuffer_result_from_void,
    },
    guest_functions::register_function,
    host_function_call::{call_host_function, get_host_value_return_as_int},
};
use log::{debug, error, info, trace, warn};

extern crate hyperlight_guest;

fn print_output(message: &str) -> Result<Vec<u8>> {
    call_host_function(
        "HostPrint",
        Some(Vec::from(&[ParameterValue::String(message.to_string())])),
        ReturnType::Int,
    )?;
    let result = get_host_value_return_as_int()?;
    Ok(get_flatbuffer_result_from_int(result))
}

fn simple_print_output(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to simple_print_output".to_string(),
        ))
    }
}

fn set_byte_array_to_zero(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::VecBytes(vec), ParameterValue::Int(length)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        unsafe {
            let mut ptr = vec.as_ptr() as *mut u8;
            for _ in 0..length {
                if !ptr.is_null() {
                    *ptr = 0;
                    ptr = ptr.add(1);
                }
            }
        }
        Ok(get_flatbuffer_result_from_void())
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to set_byte_array_to_zero".to_string(),
        ))
    }
}

fn print_two_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::String(arg1), ParameterValue::Int(arg2)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        let message = format!("Message: arg1:{} arg2:{}.", arg1, arg2);
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_two_args".to_string(),
        ))
    }
}

fn print_three_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::String(arg1), ParameterValue::Int(arg2), ParameterValue::Long(arg3)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
    ) {
        let message = format!("Message: arg1:{} arg2:{} arg3:{}.", arg1, arg2, arg3);
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_three_args".to_string(),
        ))
    }
}

fn print_four_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{}.",
            arg1, arg2, arg3, arg4
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_four_args".to_string(),
        ))
    }
}

fn print_five_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{}.",
            arg1, arg2, arg3, arg4, arg5
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_five_args".to_string(),
        ))
    }
}

fn print_six_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_six_args".to_string(),
        ))
    }
}

fn print_seven_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_seven_args".to_string(),
        ))
    }
}

fn print_eight_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::String(arg8),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_eight_args".to_string(),
        ))
    }
}

fn print_nine_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::String(arg8),
        ParameterValue::Long(arg9),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
        function_call.parameters.clone().unwrap()[8].clone(),
    ) {
        let message = format!(
            "Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{} arg9:{}.",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9
        );
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_nine_args".to_string(),
        ))
    }
}

fn print_ten_args(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (
        ParameterValue::String(arg1),
        ParameterValue::Int(arg2),
        ParameterValue::Long(arg3),
        ParameterValue::String(arg4),
        ParameterValue::String(arg5),
        ParameterValue::Bool(arg6),
        ParameterValue::Bool(arg7),
        ParameterValue::String(arg8),
        ParameterValue::Long(arg9),
        ParameterValue::Int(arg10),
    ) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
        function_call.parameters.clone().unwrap()[2].clone(),
        function_call.parameters.clone().unwrap()[3].clone(),
        function_call.parameters.clone().unwrap()[4].clone(),
        function_call.parameters.clone().unwrap()[5].clone(),
        function_call.parameters.clone().unwrap()[6].clone(),
        function_call.parameters.clone().unwrap()[7].clone(),
        function_call.parameters.clone().unwrap()[8].clone(),
        function_call.parameters.clone().unwrap()[9].clone(),
    ) {
        let message = format!("Message: arg1:{} arg2:{} arg3:{} arg4:{} arg5:{} arg6:{} arg7:{} arg8:{} arg9:{} arg10:{}.", arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10);
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to print_ten_args".to_string(),
        ))
    }
}

fn stack_allocate(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(length) = function_call.parameters.clone().unwrap()[0].clone() {
        let alloc_length = if length == 0 {
            DEFAULT_GUEST_STACK_SIZE + 1
        } else {
            length
        };

        _alloca(alloc_length as usize);

        Ok(get_flatbuffer_result_from_int(alloc_length))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to stack_allocate".to_string(),
        ))
    }
}

fn buffer_overrun(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(value) = function_call.parameters.clone().unwrap()[0].clone() {
        let c_str = value.as_str();

        let mut buffer: [u8; 17] = [0; 17];
        let length = c_str.len();

        let copy_length = length.min(buffer.len());
        buffer[..copy_length].copy_from_slice(&c_str.as_bytes()[..copy_length]);

        let result = (17i32).saturating_sub(length as i32);

        Ok(get_flatbuffer_result_from_int(result))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to buffer_overrun".to_string(),
        ))
    }
}

fn stack_overflow(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(i) = function_call.parameters.clone().unwrap()[0].clone() {
        loop_stack_overflow(i);
        Ok(get_flatbuffer_result_from_int(i))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to stack_overflow".to_string(),
        ))
    }
}
// This function will allocate i * (8KiB + 1B) on the stack
fn loop_stack_overflow(i: i32) {
    if i > 0 {
        let _nums = black_box([0u8; 0x2000 + 1]); // chkstk guaranteed to be called for > 8KiB
        loop_stack_overflow(i - 1);
    }
}

fn large_var(_: &FunctionCall) -> Result<Vec<u8>> {
    let _buffer = black_box([0u8; (DEFAULT_GUEST_STACK_SIZE + 1) as usize]);
    Ok(get_flatbuffer_result_from_int(DEFAULT_GUEST_STACK_SIZE + 1))
}

fn small_var(_: &FunctionCall) -> Result<Vec<u8>> {
    let _buffer = black_box([0u8; 1024]);
    Ok(get_flatbuffer_result_from_int(1024))
}

// TODO: This function could cause a stack overflow, update it once we have stack guards in place.
fn call_malloc(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(size) = function_call.parameters.clone().unwrap()[0].clone() {
        let alloc_length = if size < DEFAULT_GUEST_STACK_SIZE {
            // ^^^ arbitrary check to avoid stack overflow
            // because we don't have stack guards in place yet
            size
        } else {
            size.min(MAX_BUFFER_SIZE as i32)
        };
        let mut allocated_buffer = Vec::with_capacity(alloc_length as usize);
        allocated_buffer.resize(alloc_length as usize, 0);

        Ok(get_flatbuffer_result_from_int(size))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to call_malloc".to_string(),
        ))
    }
}

fn malloc_and_free(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(size) = function_call.parameters.clone().unwrap()[0].clone() {
        let alloc_length = if size < DEFAULT_GUEST_STACK_SIZE {
            size
        } else {
            size.min(MAX_BUFFER_SIZE as i32)
        };
        let mut allocated_buffer = Vec::with_capacity(alloc_length as usize);
        allocated_buffer.resize(alloc_length as usize, 0);
        drop(allocated_buffer);

        Ok(get_flatbuffer_result_from_int(size))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to malloc_and_free".to_string(),
        ))
    }
}

fn echo(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(value) = function_call.parameters.clone().unwrap()[0].clone() {
        Ok(get_flatbuffer_result_from_string(&value))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to echo".to_string(),
        ))
    }
}

fn get_size_prefixed_buffer(function_call: &FunctionCall) -> Result<Vec<u8>> {
    // This assumes that the first parameter is a buffer and the second is the length.
    // You may need to adjust this based on how your FunctionCall and ParameterValues are structured.
    if let (ParameterValue::VecBytes(data), ParameterValue::Int(length)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        unsafe {
            Ok(get_flatbuffer_result_from_size_prefixed_buffer(
                data.as_ptr(),
                length,
            ))
        }
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to get_size_prefixed_buffer".to_string(),
        ))
    }
}

fn spin(_: &FunctionCall) -> Result<Vec<u8>> {
    loop {
        // Keep the CPU 100% busy forever
    }

    #[allow(unreachable_code)]
    Ok(get_flatbuffer_result_from_void())
}

fn test_abort(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(code) = function_call.parameters.clone().unwrap()[0].clone() {
        abort_with_code(code);
    }
    Ok(get_flatbuffer_result_from_void())
}

fn test_abort_with_code_and_message(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let (ParameterValue::Int(code), ParameterValue::String(message)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        abort_with_code_and_message(code, message.as_ptr() as *const c_char);
    }
    Ok(get_flatbuffer_result_from_void())
}

fn test_guest_panic(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        panic!{"{}", message};
    }
    Ok(get_flatbuffer_result_from_void())
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn test_rust_malloc(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::Int(code) = function_call.parameters.clone().unwrap()[0].clone() {
        let ptr = hlmalloc(code as usize);
        Ok(get_flatbuffer_result_from_int(ptr as i32))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to test_rust_malloc".to_string(),
        ))
    }
}

fn log_message(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if function_call.parameters.clone().unwrap().len() != 2 {
        return Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionIncorrecNoOfParameters,
            format!(
                "Invalid number of parameters passed to log_message, expected 2 got {}",
                function_call.parameters.clone().unwrap().len()
            ),
        ));
    }

    if let (ParameterValue::String(message), ParameterValue::Int(level)) = (
        function_call.parameters.clone().unwrap()[0].clone(),
        function_call.parameters.clone().unwrap()[1].clone(),
    ) {
        match level {
            0 => trace!("{}", &message),
            1 => debug!("{}", &message),
            2 => info!("{}", &message),
            3 => warn!("{}", &message),
            4 => error!("{}", &message),
            5 => error!("{}", &message),
            _ => info!("{}", &message),
        };
        Ok(get_flatbuffer_result_from_void())
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to log_message".to_string(),
        ))
    }
}

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let simple_print_output_def = GuestFunctionDefinition::new(
        "PrintOutput".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        simple_print_output as i64,
    );
    register_function(simple_print_output_def);

    let print_using_printf_def = GuestFunctionDefinition::new(
        "PrintUsingPrintf".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        simple_print_output as i64, // alias to simple_print_output for now
    );
    register_function(print_using_printf_def);

    let stack_allocate_def = GuestFunctionDefinition::new(
        "StackAllocate".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        stack_allocate as i64,
    );
    register_function(stack_allocate_def);

    let stack_overflow_def = GuestFunctionDefinition::new(
        "StackOverflow".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        stack_overflow as i64,
    );
    register_function(stack_overflow_def);

    let buffer_overrun_def = GuestFunctionDefinition::new(
        "BufferOverrun".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        buffer_overrun as i64,
    );
    register_function(buffer_overrun_def);

    let large_var_def = GuestFunctionDefinition::new(
        "LargeVar".to_string(),
        Vec::new(),
        ReturnType::Int,
        large_var as i64,
    );
    register_function(large_var_def);

    let small_var_def = GuestFunctionDefinition::new(
        "SmallVar".to_string(),
        Vec::new(),
        ReturnType::Int,
        small_var as i64,
    );
    register_function(small_var_def);

    let call_malloc_def = GuestFunctionDefinition::new(
        "CallMalloc".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        call_malloc as i64,
    );
    register_function(call_malloc_def);

    let malloc_and_free_def = GuestFunctionDefinition::new(
        "MallocAndFree".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        malloc_and_free as i64,
    );
    register_function(malloc_and_free_def);

    let print_two_args_def = GuestFunctionDefinition::new(
        "PrintTwoArgs".to_string(),
        Vec::from(&[ParameterType::String, ParameterType::Int]),
        ReturnType::Int,
        print_two_args as i64,
    );
    register_function(print_two_args_def);

    let print_three_args_def = GuestFunctionDefinition::new(
        "PrintThreeArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
        ]),
        ReturnType::Int,
        print_three_args as i64,
    );
    register_function(print_three_args_def);

    let print_four_args_def = GuestFunctionDefinition::new(
        "PrintFourArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
        ]),
        ReturnType::Int,
        print_four_args as i64,
    );
    register_function(print_four_args_def);

    let print_five_args_def = GuestFunctionDefinition::new(
        "PrintFiveArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
        ]),
        ReturnType::Int,
        print_five_args as i64,
    );
    register_function(print_five_args_def);

    let print_six_args_def = GuestFunctionDefinition::new(
        "PrintSixArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
        ]),
        ReturnType::Int,
        print_six_args as i64,
    );
    register_function(print_six_args_def);

    let print_seven_args_def = GuestFunctionDefinition::new(
        "PrintSevenArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
        ]),
        ReturnType::Int,
        print_seven_args as i64,
    );
    register_function(print_seven_args_def);

    let print_eight_args_def = GuestFunctionDefinition::new(
        "PrintEightArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::String,
        ]),
        ReturnType::Int,
        print_eight_args as i64,
    );
    register_function(print_eight_args_def);

    let print_nine_args_def = GuestFunctionDefinition::new(
        "PrintNineArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::String,
            ParameterType::Long,
        ]),
        ReturnType::Int,
        print_nine_args as i64,
    );
    register_function(print_nine_args_def);

    let print_ten_args_def = GuestFunctionDefinition::new(
        "PrintTenArgs".to_string(),
        Vec::from(&[
            ParameterType::String,
            ParameterType::Int,
            ParameterType::Long,
            ParameterType::String,
            ParameterType::String,
            ParameterType::Bool,
            ParameterType::Bool,
            ParameterType::String,
            ParameterType::Long,
            ParameterType::Int,
        ]),
        ReturnType::Int,
        print_ten_args as i64,
    );
    register_function(print_ten_args_def);

    let set_byte_array_to_zero_def = GuestFunctionDefinition::new(
        "SetByteArrayToZero".to_string(),
        Vec::from(&[ParameterType::VecBytes, ParameterType::Int]),
        ReturnType::Int,
        set_byte_array_to_zero as i64,
    );
    register_function(set_byte_array_to_zero_def);

    let echo_def = GuestFunctionDefinition::new(
        "Echo".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        echo as i64,
    );
    register_function(echo_def);

    let get_size_prefixed_buffer_def = GuestFunctionDefinition::new(
        "GetSizePrefixedBuffer".to_string(),
        Vec::from(&[ParameterType::VecBytes, ParameterType::Int]),
        ReturnType::Int,
        get_size_prefixed_buffer as i64,
    );
    register_function(get_size_prefixed_buffer_def);

    let spin_def =
        GuestFunctionDefinition::new("Spin".to_string(), Vec::new(), ReturnType::Int, spin as i64);
    register_function(spin_def);

    let abort_def = GuestFunctionDefinition::new(
        "test_abort".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Void,
        test_abort as i64,
    );
    register_function(abort_def);

    let abort_with_code_message_def = GuestFunctionDefinition::new(
        "abort_with_code_and_message".to_string(),
        Vec::from(&[ParameterType::Int, ParameterType::String]),
        ReturnType::Void,
        test_abort_with_code_and_message as i64,
    );
    register_function(abort_with_code_message_def);

    let guest_panic_def = GuestFunctionDefinition::new(
        "guest_panic".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Void,
        test_guest_panic as i64,
    );
    register_function(guest_panic_def);

    let rust_malloc_def = GuestFunctionDefinition::new(
        "test_rust_malloc".to_string(),
        Vec::from(&[ParameterType::Int]),
        ReturnType::Int,
        test_rust_malloc as i64,
    );
    register_function(rust_malloc_def);

    let log_message_def = GuestFunctionDefinition::new(
        "LogMessage".to_string(),
        Vec::from(&[ParameterType::String, ParameterType::Int]),
        ReturnType::Void,
        log_message as i64,
    );
    register_function(log_message_def);
}

#[no_mangle]
pub fn guest_dispatch_function(function_call: &FunctionCall) -> Result<Vec<u8>> {
    Err(HyperlightGuestError::new(
        ErrorCode::GuestFunctionNotFound,
        function_call.function_name.clone(),
    ))
}
