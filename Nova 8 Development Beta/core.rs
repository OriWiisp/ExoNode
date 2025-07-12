#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Constants
pub const FB_BASE: usize = 0x1000_0000;
pub const W: usize = 1920;
pub const H: usize = 1080;

pub const MAX_VMS: usize = 1024;
pub const MAX_NODES: usize = 512;
pub const MAX_CPUS_PER_NODE: usize = 128;
pub const MAX_GPU_PER_NODE: usize = 16;
pub const MAX_STORAGE_DEVICES: usize = 16;
pub const MAX_NETWORK_INTERFACES: usize = 8;
pub const MAX_SENSORS_PER_NODE: usize = 32;
pub const MAX_MODULES_PER_NODE: usize = 32;
pub const MAX_CONTAINERS: usize = 2048;

pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

pub static VERSION: &str = "Project Nova Pro v0.1.0-alpha";

// Enums
#[derive(Clone, Copy)]
pub enum VmState {
    Running,
    Paused,
    Stopped,
    Suspended,
    Migrating,
    Hibernated,
    Crashed,
    Recovering,
    Overprovisioned,
    Error,
    Initializing,
    AwaitingNetwork,
    Scaling,
    Draining,
    Quarantined,
    Debug,
    Updating,
    Snapshotting,
    Rebooting,
}

#[derive(Clone, Copy)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
    Restarting,
    Terminating,
}

// Types used throughout
pub type Color = u32;

// Data structures (structs)
#[repr(C)]
pub struct Container {
    pub id: usize,
    pub name: &'static str,
    pub image: &'static str,
    pub node_id: usize,
    pub memory_mb: u32,
    pub cpu_shares: u32,
    pub state: ContainerState,
    // ... (all other fields you had)
}

#[repr(C)]
pub struct NetworkInterface {
    pub id: usize,
    pub mac_address: [u8; 6],
    // ... rest of fields
}

#[repr(C)]
pub struct Sensor {
    pub id: usize,
    pub name: &'static str,
    // ...
}

#[repr(C)]
pub struct VM {
    pub id: usize,
    pub active: bool,
    pub network_ip: Option<u32>,
    // ... rest of fields
}

// Globals (unsafe mutables)
pub static mut CONTAINERS: [Option<Container>; MAX_CONTAINERS] = [None; MAX_CONTAINERS];

// Panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Modules to be added:
pub mod vm_manager;
pub mod container_manager;
pub mod network;
pub mod sensors;
pub mod framebuffer;
pub mod cli;
// ... etc
