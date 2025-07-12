#![no_std]

use core::panic::PanicInfo;

/// Maximum number of VMs supported
pub const MAX_VMS: usize = 1024;

/// Maximum number of virtual network interfaces per VM
pub const MAX_NETWORK_INTERFACES: usize = 8;

/// VM lifecycle states
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Representation of a virtual network interface assigned to a VM
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkInterface {
    pub id: usize,
    pub mac_address: [u8; 6],
    pub ipv4: Option<u32>,
    pub ipv6: Option<[u8; 16]>,
    pub mtu: u16,
    pub link_speed_mbps: u32,
    pub is_up: bool,
    // Further network interface features can be added here
}

/// Virtual Machine data structure
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

    /// Virtual NICs assigned to this VM
    pub virtual_network_interfaces: [NetworkInterface; MAX_NETWORK_INTERFACES],

    // Additional VM metadata fields can be added here
}

/// Global VM storage (unsafe mutable singleton)
/// Access must be synchronized in real implementations
pub static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];

/// Create a new VM with the given id and label.
/// Returns Err if VM already exists or id out of range.
pub fn create_vm(id: usize, label: &'static str) -> Result<(), &'static str> {
    unsafe {
        if id >= MAX_VMS {
            return Err("VM ID out of range");
        }
        if VMS[id].is_some() {
            return Err("VM already exists");
        }

        // Initialize virtual NICs with default values
        let default_nic = NetworkInterface {
            id: 0,
            mac_address: [0u8; 6],
            ipv4: None,
            ipv6: None,
            mtu: 1500,
            link_speed_mbps: 1000,
            is_up: false,
        };

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
            virtual_network_interfaces: [default_nic; MAX_NETWORK_INTERFACES],
        });
        Ok(())
    }
}

/// Delete an existing VM by ID.
/// Returns Err if VM does not exist or id out of range.
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

/// Get a reference to a VM by ID.
/// Unsafe because it returns a raw pointer to global state.
pub unsafe fn get_vm(id: usize) -> Option<&'static VM> {
    if id >= MAX_VMS {
        None
    } else {
        VMS[id].as_ref()
    }
}

/// Get a mutable reference to a VM by ID.
/// Unsafe because it returns a raw pointer to global state.
pub unsafe fn get_vm_mut(id: usize) -> Option<&'static mut VM> {
    if id >= MAX_VMS {
        None
    } else {
        VMS[id].as_mut()
    }
}

/// Panic handler for vmcore.
/// Infinite loop for no_std environment.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
