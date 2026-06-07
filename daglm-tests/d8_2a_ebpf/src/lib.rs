#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::pt_regs,
    macros::{map, perf_event},
    maps::PerfEventArray,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Hit {
    pub pid: u32,
    pub tid: u32,
    pub ip: u64,
    pub reg_val: u64,
    pub reg_name: [u8; 8],
}

#[map(name = "HITS")]
static mut HITS: PerfEventArray<Hit> = PerfEventArray::new(1024);

fn has_d8_2a(v: u64) -> bool {
    let mut x = v;
    let mut i = 0;
    while i < 7 {
        let b0 = (x & 0xFF) as u8;
        let b1 = ((x >> 8) & 0xFF) as u8;
        if b0 == 0xD8 && b1 == 0x2A {
            return true;
        }
        x >>= 8;
        i += 1;
    }
    false
}

#[perf_event]
pub fn on_sample(ctx: perf_event::PerfEventContext) -> i32 {
    let perf_data = unsafe { &*(ctx.ctx) };
    let regs = &perf_data.regs;

    let pid_tgid = aya_ebpf::helpers::bpf_get_current_pid_tgid();
    let pid = (pid_tgid >> 32) as u32;
    let tid = (pid_tgid & 0xFFFF_FFFF) as u32;
    let ip = regs.rip as u64;

    let vals = [
        regs.rax as u64, regs.rbx as u64, regs.rcx as u64, regs.rdx as u64,
        regs.rsi as u64, regs.rdi as u64, regs.rsp as u64, regs.rbp as u64,
        regs.r8 as u64, regs.r9 as u64, regs.r10 as u64, regs.r11 as u64,
        regs.r12 as u64, regs.r13 as u64, regs.r14 as u64, regs.r15 as u64,
    ];

    let names: [[u8; 8]; 16] = [
        *b"rax\0\0\0\0\0", *b"rbx\0\0\0\0\0", *b"rcx\0\0\0\0\0", *b"rdx\0\0\0\0\0",
        *b"rsi\0\0\0\0\0", *b"rdi\0\0\0\0\0", *b"rsp\0\0\0\0\0", *b"rbp\0\0\0\0\0",
        *b"r8\0\0\0\0\0\0", *b"r9\0\0\0\0\0\0", *b"r10\0\0\0\0\0", *b"r11\0\0\0\0\0",
        *b"r12\0\0\0\0\0", *b"r13\0\0\0\0\0", *b"r14\0\0\0\0\0", *b"r15\0\0\0\0\0",
    ];

    unsafe {
        for i in 0..16 {
            let val = vals[i as usize];
            if has_d8_2a(val) {
                let hit = Hit {
                    pid, tid, ip, reg_val: val, reg_name: names[i as usize],
                };
                let _ = HITS.output(&ctx, &hit, 0);
            }
        }
    }

    0
}