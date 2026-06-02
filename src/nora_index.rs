//! Nora IPLD Index Module
//!
//! Stores extended crate metadata as IPLD-compatible DAG-CBOR in the shared memory server.
//! Each subsection (local, nix, pipelight, vendor, patches, consumers) is stored as
//! an independent CAR page, and a crate index maps crate names to their section CIDs.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_ipld_dagcbor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMirror {
    pub r#type: String,
    pub url: String,
    pub mirror_path: String,
    pub bare_path: String,
    pub worktree_path: String,
    pub clone_status: String,
    pub last_sync: String,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixBuild {
    pub r#type: String,
    pub flake_path: String,
    pub build_status: String,
    pub derivation_hash: String,
    pub last_build: String,
    pub systems: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelightInfo {
    pub r#type: String,
    pub pipeline_path: String,
    pub pipelines: Vec<Pipeline>,
    pub last_run: String,
    pub last_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendorsization {
    pub r#type: String,
    pub vendor_dir: String,
    pub status: String,
    pub file_count: u64,
    pub last_vendor: String,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchesInfo {
    pub r#type: String,
    pub patch_dir: String,
    pub patches: Vec<Patch>,
    pub cumulative_patch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub file: String,
    pub applied: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumersInfo {
    pub r#type: String,
    pub projects: Vec<ConsumerProject>,
    pub consumer_count: u64,
    pub dependency_depth: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerProject {
    pub name: String,
    pub path: String,
    pub workspace_member: bool,
    pub version_constraint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateIndexEntry {
    pub local_cid: String,
    pub nix_cid: String,
    pub pipelight_cid: String,
    pub vendor_cid: String,
    pub patches_cid: String,
    pub consumers_cid: String,
}

pub fn create_local_mirror(name: &str, version: &str, repo_url: Option<&str>) -> LocalMirror {
    let url = repo_url.unwrap_or(&format!("https://github.com/serde-rs/{}", name)).to_string();
    let mirror_path = format!("/home/mdupont/git/github.com/serde-rs/{}.git", name);
    let bare_path = mirror_path.clone();
    let worktree_path = format!("/mnt/data1/nix/vendor/rust/{}", name);

    LocalMirror {
        r#type: "local-mirror".to_string(),
        url,
        mirror_path,
        bare_path,
        worktree_path,
        clone_status: "exists".to_string(),
        last_sync: chrono::Utc::now().to_rfc3339(),
        branches: vec!["main".to_string(), format!("v{}", version)],
    }
}

pub fn create_nix_build(name: &str, flake_path: &str) -> NixBuild {
    NixBuild {
        r#type: "nix-build".to_string(),
        flake_path: flake_path.to_string(),
        build_status: "pending".to_string(),
        derivation_hash: String::new(),
        last_build: String::new(),
        systems: vec!["x86_64-linux".to_string(), "aarch64-linux".to_string()],
        outputs: vec!["default".to_string(), name.to_string()],
    }
}

pub fn create_pipelight_info(name: &str) -> PipelightInfo {
    PipelightInfo {
        r#type: "pipelight".to_string(),
        pipeline_path: format!("/mnt/data1/time-2024/05/28/depot/tvix/vendored/output/flakes/{}/pipelight.toml", name),
        pipelines: vec![
            Pipeline { name: format!("build-{}", name), status: "configured".to_string() },
            Pipeline { name: format!("publish-{}", name), status: "configured".to_string() },
            Pipeline { name: format!("ci-{}", name), status: "configured".to_string() },
        ],
        last_run: String::new(),
        last_result: String::new(),
    }
}

pub fn create_vendorsization(name: &str, vendor_dir: &str) -> Vendorsization {
    Vendorsization {
        r#type: "vendorsization".to_string(),
        vendor_dir: vendor_dir.to_string(),
        status: "pending".to_string(),
        file_count: 0,
        last_vendor: String::new(),
        config_path: format!("/mnt/data1/time-2024/05/28/depot/tvix/.cargo/config.toml"),
    }
}

pub fn create_patches_info() -> PatchesInfo {
    PatchesInfo {
        r#type: "patches".to_string(),
        patch_dir: "/mnt/data1/time-2024/05/28/depot/tvix/vendored/output/patches/".to_string(),
        patches: vec![],
        cumulative_patch: String::new(),
    }
}

pub fn create_consumers_info(project_name: &str, workspace_path: &str) -> ConsumersInfo {
    ConsumersInfo {
        r#type: "consumers".to_string(),
        projects: vec![ConsumerProject {
            name: project_name.to_string(),
            path: workspace_path.to_string(),
            workspace_member: true,
            version_constraint: "*".to_string(),
        }],
        consumer_count: 1,
        dependency_depth: 1,
    }
}

pub fn encode_as_dag_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(serde_ipld_dagcbor::to_vec(value)?)
}

pub fn wrap_as_car(content: &[u8]) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    let content_hash = Sha256::digest(content);
    let mut cid_bytes = Vec::with_capacity(36);
    cid_bytes.push(0x01);
    cid_bytes.push(0x55);
    cid_bytes.push(0x12);
    cid_bytes.push(0x20);
    cid_bytes.extend_from_slice(&content_hash);

    let header = build_car_header(&cid_bytes);
    let header_len = header.len();
    let mut car = Vec::new();
    write_varint(&mut car, header_len as u64);
    car.extend_from_slice(&header);
    car.push(0x01);
    car.push(0x55);
    car.push(0x12);
    car.push(0x20);
    car.extend_from_slice(&content_hash);
    car.extend_from_slice(content);
    Ok(car)
}

pub fn build_car_header(root_cid: &[u8]) -> Vec<u8> {
    let mut h = Vec::new();
    h.push(0xA2);
    h.push(0x67); h.extend_from_slice(b"version"); h.push(0x01);
    h.push(0x65); h.extend_from_slice(b"roots");
    h.push(0x81); h.push(0xD8); h.push(0x2A);
    h.push(0x58); h.push(root_cid.len() as u8); h.extend_from_slice(root_cid);
    h
}

pub fn write_varint(buf: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 { byte |= 0x80; }
        buf.push(byte);
        if value == 0 { break; }
    }
}

pub fn create_crate_index(
    crate_name: &str,
    local_cid: &[u8; 32],
    nix_cid: &[u8; 32],
    pipelight_cid: &[u8; 32],
    vendor_cid: &[u8; 32],
    patches_cid: &[u8; 32],
    consumers_cid: &[u8; 32],
) -> (CrateIndexEntry, Vec<u8>) {
    let entry = CrateIndexEntry {
        local_cid: hex::encode(local_cid),
        nix_cid: hex::encode(nix_cid),
        pipelight_cid: hex::encode(pipelight_cid),
        vendor_cid: hex::encode(vendor_cid),
        patches_cid: hex::encode(patches_cid),
        consumers_cid: hex::encode(consumers_cid),
    };

    let car_bytes = wrap_as_car(&encode_as_dag_cbor(&entry).unwrap_or_default()).unwrap_or_default();
    (entry, car_bytes)
}