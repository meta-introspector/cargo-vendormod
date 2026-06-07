.PHONY: all build-ebpf monitor hits clean

all: build-ebpf monitor

# Build eBPF program for perf_event
build-ebpf:
	cd /mnt/data1/time-2026/06/03/aya && cargo build --release
	cd d8_2a_ebpf && cargo build --release --target bpfel-unknown-none

# Run eBPF monitor (requires root)
monitor:
	@echo "Starting eBPF monitor (requires root)..."
	@echo "Run in separate terminal: sudo ../../target/release/d8_2a_user"

# Build and test all projects while recording hits
hits:
	@echo "Building all projects to capture 0xD8 0x2A hits..."
	@# Layer 2: Direct Users
	cargo build --release -p serde_ipld_dagcbor-escaped 2>/dev/null || true
	@# Layer 3: Consumer Crates
	cargo build --release -p n0_dasl 2>/dev/null || true
	cargo build --release -p honggfuzz-rs 2>/dev/null || true
	@# Layer 4: IMPL Projects
	cd /home/mdupont/dasl/IMPL/jacquard && cargo build --release 2>/dev/null || true

# Quick test showing the register scan logic
test-scan:
	@echo "Testing 0xD8 0x2A detection logic..."
	@python3 -c "
data = bytes([0x01, 0x55, 0x12, 0x20, 0xF3, 0xC2, 0xC5, 0x90, 0xD8, 0x2A, 0x58, 0x24, 0x00])
for i in range(len(data)-1):
    if data[i] == 0xD8 and data[i+1] == 0x2A:
        print(f'Found 0xD8 0x2A at offset {i}')
"