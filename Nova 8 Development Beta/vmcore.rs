#![no_std]

use core::panic::PanicInfo;

pub const MAX_VMS: usize = 1024;
pub const MAX_NETWORK_INTERFACES: usize = 8;

#[derive(Clone, Copy, Debug)]
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

#[repr(C)]
pub struct NetworkInterface {
    pub id: usize,
    pub mac_address: [u8; 6],
    pub ipv4: Option<u32>,
    pub ipv6: Option<[u8; 16]>,
    pub mtu: u16,
    pub link_speed_mbps: u32,
    pub is_up: bool,
    // ... add fields as needed
}

#[repr(C)]
pub struct VM {
    pub id: usize,
    pub active: bool,
    pub network_ip: Option<u32>,
    pub node_id: usize,
    pub cpu_mask: u64,
    pub state: VmState,
    pub uptime_ticks: u64,
    pub label: &'static str,
    pub memory_allocated_mb: u32,
    // ... add more fields as needed
    pub virtual_network_interfaces: [NetworkInterface; MAX_NETWORK_INTERFACES],
}

pub static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];

// Panic handler for VM Core
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Public API function placeholders (can be expanded in vm_manager.rs)
pub fn create_vm(id: usize, label: &'static str) -> Result<(), &'static str> {
    unsafe {
        if id >= MAX_VMS {
            return Err("VM ID out of range");
        }
        if VMS[id].is_some() {
            return Err("VM already exists");
        }
        VMS[id] = Some(VM {
            id,
            active: true,
            network_ip: None,
            node_id: 0,
            cpu_mask: 0,
            state: VmState::Initializing,
            uptime_ticks: 0,
            label,
            memory_allocated_mb: 0,
            virtual_network_interfaces: [NetworkInterface {
                id: 0,
                mac_address: [0; 6],
                ipv4: None,
                ipv6: None,
                mtu: 1500,
                link_speed_mbps: 1000,
                is_up: false,
            }; MAX_NETWORK_INTERFACES],
        });
        Ok(())
    }
}

pub fn delete_vm(id: usize) -> Result<(), &'static str> {
    unsafe {
        if id >= MAX_VMS {
            return Err("VM ID out of range");
        }
        if VMS[id].is_none() {
            return Err("VM does not exist");
        }
        VMS[id] = None;
        Ok(())
    }
}
