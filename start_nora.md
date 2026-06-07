# Start Nora Cargo Registry

## Current Status
The nora registry binary needs to be built and started before running dasl-testing.

## Steps to Start Nora

### 1. Build Nora
```bash
cd /mnt/data1/time-2026/05-may/28/nora
cargo build --release -p nora-registry
```

### 2. Start Nora Server
```bash
./target/release/nora --config /mnt/data1/nora/config/nora.toml
```

This starts the server on `http://127.0.0.1:4000/cargo/index`.

### 3. Verify Nora is Running
```bash
curl http://127.0.0.1:4000/cargo/index
# Should return JSON or 404 for missing packages (but server is up)
```

### 4. Run DASL Testing with Nora
```bash
# In another terminal
cd /mnt/data1/time-2026/02-february/22/dasl/dasl-testing
cargo build --release -p test_n0_dasl
```

## Alternative: Bypass Nora Temporarily
If nora cannot be started, comment out the `.cargo/config.toml` source replacement:
```bash
cd /mnt/data1/time-2026/02-february/22/dasl/dasl-testing
mv .cargo/config.toml .cargo/config.toml.bak
# Run tests
cargo build --release -p test_n0_dasl
mv .cargo/config.toml.bak .cargo/config.toml
```