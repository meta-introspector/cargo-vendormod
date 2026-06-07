use aya::{
    include_bytes_aligned,
    maps::perf::PerfEventArray,
    programs::PerfEvent,
    Bpf,
};
use anyhow::Result;
use std::{mem, time::Duration};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Hit {
    pid: u32,
    tid: u32,
    ip: u64,
    reg_val: u64,
    reg_name: [u8; 8],
}

fn main() -> Result<()> {
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../d8_2a_ebpf/target/bpfel-unknown-none/release/d8_2a_ebpf"
    ))?;

    let program: &mut PerfEvent = bpf.program_mut("on_sample")?.try_into()?;
    program.load()?;
    program.attach(aya::programs::PerfEventScope::AllCpu)?;

    let mut hits: PerfEventArray<Hit> = bpf.map_mut("HITS")?.try_into()?;
    let mut buf = aya::maps::perf::PerfBuffer::new(hits, hit_handler, lost_handler)?;

    println!("Monitoring for 0xD8 0x2A in registers...");
    loop {
        buf.poll(Duration::from_millis(100))?;
    }
}

fn hit_handler(_cpu: i32, data: &[u8]) {
    if data.len() < mem::size_of::<Hit>() {
        return;
    }
    let mut h = Hit {
        pid: 0, tid: 0, ip: 0, reg_val: 0, reg_name: [0; 8],
    };
    unsafe {
        core::ptr::copy_nonoverlapping(
            data.as_ptr(), &mut h as *mut Hit as *mut u8,
            mem::size_of::<Hit>(),
        );
    }

    let name = String::from_utf8_lossy(
        &h.reg_name.iter().copied().take_while(|b| *b != 0).collect::<Vec<_>>(),
    );
    println!(
        "hit: pid={} tid={} ip=0x{:x} reg={} val=0x{:x}",
        h.pid, h.tid, h.ip, name, h.reg_val
    );
}

fn lost_handler(cpu: i32, count: u64) {
    eprintln!("lost {} events on cpu {}", count, cpu);
}