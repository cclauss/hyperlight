use super::context::Context;
use super::handle::Handle;
use super::hdl::Hdl;
use crate::hypervisor::kvm;
use crate::hypervisor::kvm_mem::{map_vm_memory_region_raw, unmap_vm_memory_region_raw};
use crate::hypervisor::kvm_regs::{CSRegs, Regs, SRegs};
use crate::{validate_context, validate_context_or_panic};
use anyhow::Result;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{Kvm, VcpuFd, VmFd};
use std::os::raw::c_void;

fn get_kvm(ctx: &Context, handle: Handle) -> Result<&Kvm> {
    Context::get(handle, &ctx.kvms, |b| matches!(b, Hdl::Kvm(_)))
}

fn get_vmfd(ctx: &Context, handle: Handle) -> Result<&VmFd> {
    Context::get(handle, &ctx.kvm_vmfds, |b| matches!(b, Hdl::KvmVmFd(_)))
}

fn get_vcpufd(ctx: &Context, handle: Handle) -> Result<&VcpuFd> {
    Context::get(handle, &ctx.kvm_vcpufds, |b| matches!(b, Hdl::KvmVcpuFd(_)))
}

fn get_user_mem_region_mut(
    ctx: &mut Context,
    handle: Handle,
) -> Result<&mut kvm_userspace_memory_region> {
    Context::get_mut(handle, &mut ctx.kvm_user_mem_regions, |b| {
        matches!(b, Hdl::KvmUserMemRegion(_))
    })
}

fn get_kvm_run_message(ctx: &Context, handle: Handle) -> Result<&kvm::KvmRunMessage> {
    Context::get(handle, &ctx.kvm_run_messages, |b| {
        matches!(b, Hdl::KvmRunMessage(_))
    })
}

fn get_sregisters_from_handle(ctx: &Context, handle: Handle) -> Result<&SRegs> {
    Context::get(handle, &(ctx.kvm_sregs), |h| {
        matches!(h, Hdl::KvmSRegisters(_))
    })
}

/// Returns a bool indicating if kvm is present on the machine
///
/// # Examples
///
/// ```
/// use hyperlight_host::capi::kvm::kvm_is_present;
///
/// assert_eq!(kvm::kvm_is_present(), true );
/// ```
#[no_mangle]
pub extern "C" fn kvm_is_present() -> bool {
    // At this point we dont have any way to report the error if one occurs.
    kvm::is_present().map(|_| true).unwrap_or(false)
}

/// Open a Handle to KVM. Returns a handle to a KVM or a `Handle` to an error
/// if there was an issue.
///
/// The caller is responsible for closing the handle by passing it
/// to `handle_free` exactly once after they're done using it.
/// Doing so will not only free the memory that was allocated by
/// this function, it will also free all internal resources connected to
/// the associated VM, such as the underlying file descriptor.
///
/// No explicit close function (i.e. `kvm_close`) is needed or provided.
///
/// # Safety
///
/// You must free this handle by calling `handle_free` exactly once
/// after you're done using it.
///
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn kvm_open(ctx: *mut Context) -> Handle {
    match kvm::open() {
        Ok(k) => Context::register(k, &mut (*ctx).kvms, Hdl::Kvm),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Create a VM and return a Handle to it. Returns a handle to a VM or a `Handle` to an error
/// if there was an issue.
///
/// # Safety
///
/// You must free this handle by calling `handle_free` exactly once
/// after you're done using it.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
///
/// 2. `Handle` to `kvm` that has been:
/// - Created with `kvm_open`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn kvm_create_vm(ctx: *mut Context, kvm_handle: Handle) -> Handle {
    validate_context!(ctx);

    let kvm = match get_kvm(&*ctx, kvm_handle) {
        Ok(kvm) => kvm,
        Err(e) => return (*ctx).register_err(e),
    };
    match kvm::create_vm(kvm) {
        Ok(vm_fd) => Context::register(vm_fd, &mut (*ctx).kvm_vmfds, Hdl::KvmVmFd),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Create a KVM vCPU and return a Handle to it.
/// Returns a handle to a vCPU or a `Handle` to an error if there was an
/// issue.
///
/// # Safety
///
/// You must free this handle by calling `handle_free` exactly once
/// after you're done using it.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
///
/// 2. `Handle` to a `VmFd` that has been:
/// - Created with `create_vm`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn kvm_create_vcpu(ctx: *mut Context, vmfd_hdl: Handle) -> Handle {
    validate_context!(ctx);

    let vmfd = match get_vmfd(&*ctx, vmfd_hdl) {
        Ok(vmfd) => vmfd,
        Err(e) => return (*ctx).register_err(e),
    };
    match kvm::create_vcpu(vmfd) {
        Ok(res) => Context::register(res, &mut (*ctx).kvm_vcpufds, Hdl::KvmVcpuFd),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Map a memory region in the host to the VM and return a Handle to it. Returns a handle to a mshv_user_mem_region or a `Handle` to an error
/// if there was an issue.
///
/// # Safety
///
/// You must destory this handle by calling `kvm_unmap_vm_memory_region` exactly once
/// after you're done using it.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
///
/// 2. `Handle` to a `VmFd` that has been:
/// - Created with `kvm_create_vm`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
///
/// 3. The guest physical address of the memory region
///
/// 4. The load address of the memory region being mapped (this is the address of the memory in the host process)
///
/// 5. The size of the memory region being mapped (this is the size of the memory allocated at load_address)
#[no_mangle]
pub unsafe extern "C" fn kvm_map_vm_memory_region(
    ctx: *mut Context,
    vmfd_hdl: Handle,
    guest_phys_addr: u64,
    userspace_addr: *const c_void,
    mem_size: u64,
) -> Handle {
    validate_context!(ctx);

    let vmfd = match get_vmfd(&*ctx, vmfd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    match map_vm_memory_region_raw(vmfd, guest_phys_addr, userspace_addr, mem_size) {
        Ok(mem_region) => Context::register(
            mem_region,
            &mut (*ctx).kvm_user_mem_regions,
            Hdl::KvmUserMemRegion,
        ),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Unmap a memory region in the host to the VM and return a Handle to it. Returns an empty handle or a `Handle` to an error
/// if there was an issue.
///
/// # Safety
///
/// If the retruned handle is a Handle to an error then it should be freed by calling `handle_free` .The empty handle does not need to be freed but calling `handle_free` is will not cause an error.
/// The `mshv_user_mem_regions_handle` handle passed to this function should be freed after the call using `free_handle`.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
///
/// 2. `Handle` to a `VmFd` that has been:
/// - Created with `kvm_create_vm`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
///
/// 3. `Handle` to a `kvm_userspace_memory_region` that has been:
/// - Created with `kvm_map_vm_memory_region`
/// - Not unmapped and freed by calling this function
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn kvm_unmap_vm_memory_region(
    ctx: *mut Context,
    vmfd_hdl: Handle,
    user_mem_region_hdl: Handle,
) -> Handle {
    validate_context!(ctx);

    let vmfd = match get_vmfd(&*ctx, vmfd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    let mem_region = match get_user_mem_region_mut(&mut *ctx, user_mem_region_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    match unmap_vm_memory_region_raw(vmfd, &mut *mem_region) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Get registers from the vCPU. Returns a `Handle` holding a reference
/// to registers or a `Handle referencing an error if there was an issue.
/// Fetch the registers from a successful `Handle` with
/// `kvm_get_registers_from_handle`.
///
/// # Safety
///
/// If the handle is a Handle to an error then it should be freed by
/// calling `handle_free`.
/// The empty handle does not need to be freed but calling `handle_free`
/// will not cause an error.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
///
/// 2. `Handle` to a `VcpuFd` that has been:
/// - Created with `kvm_create_vcpu`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
///
/// 3. A valid `kvm_regs` instance
#[no_mangle]
pub unsafe extern "C" fn kvm_get_registers(ctx: *mut Context, vcpufd_hdl: Handle) -> Handle {
    validate_context!(ctx);

    let vcpufd = match get_vcpufd(&*ctx, vcpufd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    match kvm::get_registers(vcpufd) {
        Ok(regs) => Context::register(regs, &mut (*ctx).kvm_regs, Hdl::KvmRegisters),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Get registers from a handle created by `kvm_get_registers`.
///
/// Returns either a pointer to the registers or `NULL`.
///
/// # Safety
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
///
/// 2. `Handle` to a registers struct that has been created by
/// a call to `kvm_get_registers`
///
/// If this function returns a non-`NULL` pointer, the caller is responsible
/// for calling `free` on that pointer when they're done with the memory.
#[no_mangle]
pub unsafe extern "C" fn kvm_get_registers_from_handle(
    ctx: *const Context,
    regs_hdl: Handle,
) -> *mut Regs {
    validate_context_or_panic!(ctx);

    match Context::get(regs_hdl, &((*ctx).kvm_regs), |h| {
        matches!(h, Hdl::KvmRegisters(_))
    }) {
        Ok(r) => Box::into_raw(Box::new(*r)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get segment registers from the vCPU. Returns a `Handle` holding a reference
/// to registers or a `Handle referencing an error if there was an issue.
/// Fetch the registers from a successful `Handle` with
/// `kvm_get_registers_from_handle`.
///
/// # Safety
///
/// If the handle is a Handle to an error then it should be freed by
/// calling `handle_free`.
/// The empty handle does not need to be freed but calling `handle_free`
/// will not cause an error.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
///
/// 2. `Handle` to a `VcpuFd` that has been:
/// - Created with `kvm_create_vcpu`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API

#[no_mangle]
pub unsafe extern "C" fn kvm_get_sregisters(ctx: *mut Context, vcpufd_hdl: Handle) -> Handle {
    validate_context!(ctx);

    let vcpufd = match get_vcpufd(&*ctx, vcpufd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    match kvm::get_sregisters(vcpufd) {
        Ok(regs) => Context::register(regs, &mut (*ctx).kvm_sregs, Hdl::KvmSRegisters),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Get sregisters from a handle created by `kvm_get_sregisters`.
///
/// Returns either a pointer to the registers or `NULL`.
///
/// The pointer returned is to a CSRegs struct. This is identical to the
/// Sregs struct except that private kvm_sregs field is removed.
///
/// # Safety
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
///
/// 2. `Handle` to a sregisters struct that has been created by
/// a call to `kvm_get_sregisters`
///
/// If this function returns a non-`NULL` pointer, the caller is responsible
/// for calling `free` on that pointer when they're done with the memory.
#[no_mangle]
pub unsafe extern "C" fn kvm_get_sregisters_from_handle(
    ctx: *const Context,
    sregs_hdl: Handle,
) -> *mut CSRegs {
    validate_context_or_panic!(ctx);

    match get_sregisters_from_handle(&*ctx, sregs_hdl) {
        Ok(r) => Box::into_raw(Box::new(CSRegs::from(r))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Set Registers in the vCPU. Returns an empty handle or a `Handle` to
/// an error if there was an issue.
///
/// # Safety
///
/// If the handle is a Handle to an error then it should be freed by
/// calling `handle_free`.
/// The empty handle does not need to be freed but calling `handle_free`
/// will not cause an error.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
///
/// 2. `Handle` to a `VcpuFd` that has been:
/// - Created with `kvm_create_vcpu`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
///
/// 3. A valid `Regs` instance
#[no_mangle]
pub unsafe extern "C" fn kvm_set_registers(
    ctx: *mut Context,
    vcpufd_hdl: Handle,
    // TODO: consider passing this by reference or creating a new
    // Handle type for registers and passing a handle here.
    regs: Regs,
) -> Handle {
    validate_context!(ctx);

    let vcpu_fd = match get_vcpufd(&*ctx, vcpufd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    // TODO: create a RegisterArray similar to ByteArray here?
    match kvm::set_registers(vcpu_fd, &regs) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Set segment registers `sregs` on the vcpu stored in `ctx` referenced
/// by `vcpufd_hdl`.
///
/// # Safety
///
/// If the handle is a Handle to an error then it should be freed by
/// calling `handle_free`.
/// The empty handle does not need to be freed but calling `handle_free`
/// will not cause an error.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
///
/// 2. `Handle` to a `VcpuFd` that has been:
/// - Created with `kvm_create_vcpu`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
///
/// 3. `Handle` to a sregisters struct that has been:
/// - created by a call to `kvm_get_sregisters`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
///
/// 4. A valid `CSRegs` instance
#[no_mangle]
pub unsafe extern "C" fn kvm_set_sregisters(
    ctx: *mut Context,
    vcpufd_hdl: Handle,
    sregs_hdl: Handle,
    csregs: CSRegs,
) -> Handle {
    validate_context!(ctx);

    let vcpu_fd = match get_vcpufd(&*ctx, vcpufd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };

    let mut sregs = match get_sregisters_from_handle(&*ctx, sregs_hdl) {
        Ok(r) => *r,
        Err(e) => return (*ctx).register_err(e),
    };

    sregs.cs = csregs.cs;
    sregs.cr0 = csregs.cr0;
    sregs.cr3 = csregs.cr3;
    sregs.cr4 = csregs.cr4;
    sregs.efer = csregs.efer;

    match kvm::set_sregisters(vcpu_fd, &sregs) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Runs a vCPU. Returns an handle to an `kvm_run_message` or a
/// `Handle` to an error if there was an issue.
///
/// # Safety
///
/// The returned handle is a handle to an `kvm_run_message`.
/// The  corresponding `kvm_run_message`
/// should be retrieved using `kvm_get_run_result_from_handle`.
/// The handle should be freed by calling `handle_free` once the message
/// has been retrieved.
///
/// You must call this function with
///
/// 1. A `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
/// - Used to call `kvm_set_registers`
///
/// 2. `Handle` to a `VcpuFd` that has been:
/// - Created with `kvm_create_vcpu`
/// - Not yet freed with `handle_free`
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn kvm_run_vcpu(ctx: *mut Context, vcpufd_hdl: Handle) -> Handle {
    validate_context!(ctx);

    let vcpu_fd = match get_vcpufd(&*ctx, vcpufd_hdl) {
        Ok(r) => r,
        Err(e) => return (*ctx).register_err(e),
    };
    match kvm::run_vcpu(vcpu_fd) {
        Ok(run_result) => {
            Context::register(run_result, &mut (*ctx).kvm_run_messages, Hdl::KvmRunMessage)
        }
        Err(e) => (*ctx).register_err(e),
    }
}

/// Gets the `kvm_run_message` associated with the given handle.
///
/// # Safety
///
/// Both the returned `kvm_run_message` and the given `handle` should
/// be freed by the called when they're no longer in use. The former
/// should be freed with `free` and the latter with `handle_free`.
///
/// You must call this function with
///
/// 1. `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free`
/// - Not modified, except by calling functions in the Hyperlight C API
/// - Used to call `kvm_open`
/// - Used to call `kvm_create_vm`
/// - Used to call `kvm_create_vcpu`
/// - Used to call `kvm_set_registers`
/// - Used to call `kvm_run_vcpu`
///
/// 2. `Handle` to a `kvm_run_message` that has been:
/// - Created with `kvm_run_vcpu`
/// - Not yet used to call this function
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn kvm_get_run_result_from_handle(
    ctx: *mut Context,
    handle: Handle,
) -> *const kvm::KvmRunMessage {
    validate_context_or_panic!(ctx);

    let result = match get_kvm_run_message(&*ctx, handle) {
        Ok(res) => res,
        Err(_) => return std::ptr::null(),
    };
    // TODO: Investigate why calling (*ctx).remove(hdl, |_| true) hangs here.
    // This would be a better way to do things...
    Box::into_raw(Box::new(*result))
}

/// Frees a `kvm_run_message` previously returned by
/// `kvm_get_run_result_from_handle`.
///
/// see https://doc.rust-lang.org/std/boxed/index.html#memory-layout
/// for information on how the mechanics of this function work.
///
/// # Safety
///
/// You must call this function with
///
///
/// 1. A Pointer to a previously returned  `kvm_run_message` from
/// `kvm_get_run_result_from_handle`.
/// - Created with `kvm_get_run_result_from_handle`
/// - Not yet used to call this function
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub extern "C" fn kvm_free_run_result(_: Option<Box<kvm::KvmRunMessage>>) {}
