// SPDX-License-Identifier: MPL-2.0
// A Kernel in About 100 Lines of Safe Rust for LoongArch64
// adapted from https://asterinas.github.io/book/ostd/a-100-line-kernel.html

#![no_std]
#![deny(unsafe_code)]

extern crate alloc;

use align_ext::AlignExt;
use core::str;

use alloc::sync::Arc;
use alloc::vec;

use ostd::arch::qemu::{QemuExitCode, exit_qemu};
use ostd::cpu::context::UserContext;
use ostd::mm::{
    CachePolicy, FallibleVmRead, FrameAllocOptions, PAGE_SIZE, PageFlags, PageProperty, Vaddr,
    VmIo, VmSpace, VmWriter,
};
use ostd::prelude::*;
use ostd::task::{Task, TaskOptions};
use ostd::user::{ReturnReason, UserContextApi, UserMode};

#[ostd::main]
fn kernel_main() {
    println!("Hello world from guest kernel!");
    let program_binary = include_bytes!("../../hello");
    let vm_space = Arc::new(create_vm_space(program_binary));
    vm_space.activate();
    let user_task = create_user_task(vm_space);
    user_task.run();
}

fn create_vm_space(program: &[u8]) -> VmSpace {
    let nbytes = program.len().align_up(PAGE_SIZE);
    let user_pages = {
        let segment = FrameAllocOptions::new()
            .alloc_segment(nbytes / PAGE_SIZE)
            .unwrap();
        // Physical memory pages can be only accessed
        // via the `UFrame` or `USegment` abstraction.
        segment.write_bytes(0, program).unwrap();
        segment
    };

    // The page table of the user space can be
    // created and manipulated safely through
    // the `VmSpace` abstraction.
    let vm_space = VmSpace::new();
    const MAP_ADDR: Vaddr = 0x120000000; // The map addr for statically-linked executable
    let mut cursor = vm_space.cursor_mut(&(MAP_ADDR..MAP_ADDR + nbytes)).unwrap();
    let map_prop = PageProperty::new(PageFlags::RWX, CachePolicy::Writeback);
    for frame in user_pages {
        cursor.map(frame.into(), map_prop);
    }
    drop(cursor);
    vm_space
}

fn create_user_task(vm_space: Arc<VmSpace>) -> Arc<Task> {
    fn user_task() {
        let current = Task::current().unwrap();
        // Switching between user-kernel space is
        // performed via the UserMode abstraction.
        let mut user_mode = {
            let user_ctx = create_user_context();
            UserMode::new(user_ctx)
        };

        loop {
            // The execute method returns when system
            // calls or CPU exceptions occur or some
            // events specified by the kernel occur.
            let return_reason = user_mode.execute(|| false);

            // The CPU registers of the user space
            // can be accessed and manipulated via
            // the `UserContext` abstraction.
            let user_context = user_mode.context_mut();
            if ReturnReason::UserSyscall == return_reason {
                let vm_space = current.data().downcast_ref::<Arc<VmSpace>>().unwrap();
                handle_syscall(user_context, &vm_space);
            }
        }
    }

    // Kernel tasks are managed by the Framework,
    // while scheduling algorithms for them can be
    // determined by the users of the Framework.
    Arc::new(TaskOptions::new(user_task).data(vm_space).build().unwrap())
}

fn create_user_context() -> UserContext {
    // The user-space CPU states can be initialized
    // to arbitrary values via the `UserContext`
    // abstraction.
    let mut user_ctx = UserContext::default();
    const ENTRY_POINT: Vaddr = 0x120000078; // The entry point for statically-linked executable
    user_ctx.set_instruction_pointer(ENTRY_POINT);
    user_ctx
}

fn handle_syscall(user_context: &mut UserContext, vm_space: &VmSpace) {
    const SYS_WRITE: usize = 64;
    const SYS_EXIT: usize = 93;

    let syscall_num = user_context.a7();

    match syscall_num {
        SYS_WRITE => {
            // Access the user-space CPU registers safely.
            let (_, buf_addr, buf_len) = (user_context.a0(), user_context.a1(), user_context.a2());

            let buf = {
                let mut buf = vec![0u8; buf_len];
                // Copy data from the user space without
                // unsafe pointer dereferencing.
                let mut reader = vm_space.reader(buf_addr, buf_len).unwrap();
                reader
                    .read_fallible(&mut VmWriter::from(&mut buf as &mut [u8]))
                    .unwrap();
                buf
            };
            // Use the console for output safely.
            println!("{}", str::from_utf8(&buf).unwrap());
            // Manipulate the user-space CPU registers safely.
            user_context.set_a0(buf_len);
        }
        SYS_EXIT => exit_qemu(QemuExitCode::Success),
        _ => unimplemented!(),
    }
}
