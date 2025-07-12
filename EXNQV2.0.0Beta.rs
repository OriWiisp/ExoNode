#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;
use core::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------
// Constants & Types
// ---------------------------------

const FB_BASE: usize = 0x1000_0000;
const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

const MAX_VMS: usize = 256;
const MAX_NODES: usize = 128;
const MAX_CPUS_PER_NODE: usize = 64;
const MAX_GPUS_PER_NODE: usize = 8;
const MAX_STORAGE_PER_NODE: usize = 16;
const MAX_MSG_QUEUE_SIZE: usize = 512;

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;

type Color = u32;

#[derive(Clone, Copy, Debug)]
pub enum VmState {
    Running,
    Paused,
    Stopped,
    Suspended,
    Migrating,
    Hibernated,
    Crashed,
}

#[derive(Clone, Copy, Debug)]
pub enum Role {
    Admin = 0,
    Operator = 1,
    Guest = 2,
}

#[derive(Clone, Copy)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

// ---------------------------------
// Basic Utilities
// ---------------------------------

fn atoi(s: &str) -> usize {
    let mut res = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            break;
        }
        res = res * 10 + (b - b'0') as usize;
    }
    res
}

fn atoi64(s: &str) -> u64 {
    let mut res = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            break;
        }
        res = res * 10 + (b - b'0') as u64;
    }
    res
}

// ---------------------------------
// Panic Handler
// ---------------------------------

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ---------------------------------
// Global Variables
// ---------------------------------

static VERSION: &str = "ExoNode Quantum v1.0.0 MONOLITH";

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
    pub snapshot_enabled: bool,
    pub encrypted: bool,
    pub migration_target: Option<usize>,
    pub high_availability: bool,
    pub auto_restart: bool,
    pub priority_class: u8,
    pub backups_enabled: bool,
    pub restore_point: Option<u64>,
}

#[repr(C)]
pub struct CPU {
    pub id: usize,
    pub online: bool,
    pub usage_percent: u8,
    pub throttled: bool,
    pub frequency_mhz: u16,
}

#[repr(C)]
pub struct GPU {
    pub id: usize,
    pub online: bool,
    pub load_percent: u8,
    pub temperature_celsius: u8,
    pub fan_speed_rpm: u16,
    pub assigned_vm: Option<usize>,
    pub memory_mb: u32,
    pub pci_address: u32,
}

#[repr(C)]
pub struct StorageDevice {
    pub id: usize,
    pub online: bool,
    pub capacity_gb: u32,
    pub used_gb: u32,
    pub temperature_celsius: u8,
    pub io_bandwidth_mbps: u32,
    pub assigned_vm: Option<usize>,
    pub readonly: bool,
    pub encrypted: bool,
}

#[repr(C)]
pub struct Node {
    pub id: usize,
    pub online: bool,
    pub cpus: [CPU; MAX_CPUS_PER_NODE],
    pub gpus: [GPU; MAX_GPUS_PER_NODE],
    pub storage: [StorageDevice; MAX_STORAGE_PER_NODE],
    pub metadata: &'static str,
    pub thermal_celsius: u8,
    pub power_watts: u16,
    pub uptime_seconds: u64,
    pub io_bandwidth_mbps: u32,
    pub secure_boot_enabled: bool,
    pub fan_rpm: u16,
    pub rack_position: u8,
    pub network_latency_ms: u16,
    pub ipv6_enabled: bool,
}

#[repr(C)]
pub struct UserSession {
    pub username: &'static str,
    pub role: Role,
    pub session_id: u64,
    pub authenticated: bool,
}

// Message queue for VM-to-VM comms
#[repr(C)]
pub struct Message {
    pub sender_vm: usize,
    pub receiver_vm: usize,
    pub payload: &'static str,
    pub timestamp: u64,
}

// ---------------------------------
// Globals - Unsafe access guarded by functions
// ---------------------------------

static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];
static mut NODES: [Option<Node>; MAX_NODES] = [None; MAX_NODES];
static mut CURRENT_USER: Option<UserSession> = None;
static mut MESSAGE_QUEUE: [Option<Message>; MAX_MSG_QUEUE_SIZE] = [None; MAX_MSG_QUEUE_SIZE];
static mut MSG_QUEUE_HEAD: usize = 0;
static mut MSG_QUEUE_TAIL: usize = 0;
static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

// ---------------------------------
// Entry Point
// ---------------------------------

#[no_mangle]
pub extern "C" fn _start() -> ! {
    uart_init();
    framebuffer_init();
    pcie_init();
    discover_nodes();
    hypervisor_init();
    startup_screen();
    start_event_loop();

    loop {
        poll_uart();
        poll_mouse();
        poll_cluster();
        update_telemetry();
        process_vm_messages();
        scheduler_tick();
        cleanup_resources();
    }
}

// ---------------------------------
// Initialization Functions
// ---------------------------------

fn uart_init() {
    print_line("[UART] Initialized");
}

fn framebuffer_init() {
    clear_screen(0x101010);
    draw_dashboard();
    print_line("[Framebuffer] Initialized");
}

fn pcie_init() {
    print_line("[PCIe] Initialized (simulated)");
}

fn hypervisor_init() {
    print_line("[HV] Virtualization enabled");
}

fn startup_screen() {
    clear_screen(0x000000);
    draw_text(50, 50, "Welcome to ExoNode Quantum Hypervisor!", 0xFFFFFF);
    draw_text(50, 70, VERSION, 0x00FF00);
}

// ---------------------------------
// Hardware Discovery & Cluster
// ---------------------------------

fn discover_nodes() {
    unsafe {
        for i in 0..MAX_NODES {
            let mut cpus = [CPU { id: 0, online: false, usage_percent: 0, throttled: false, frequency_mhz: 3200 }; MAX_CPUS_PER_NODE];
            for j in 0..MAX_CPUS_PER_NODE {
                cpus[j] = CPU { id: j, online: true, usage_percent: 0, throttled: false, frequency_mhz: 3200 };
            }
            let mut gpus = [GPU {
                id: 0,
                online: false,
                load_percent: 0,
                temperature_celsius: 40,
                fan_speed_rpm: 1500,
                assigned_vm: None,
                memory_mb: 4096,
                pci_address: 0,
            }; MAX_GPUS_PER_NODE];
            for k in 0..MAX_GPUS_PER_NODE {
                gpus[k].id = k;
                gpus[k].online = true;
                gpus[k].pci_address = 0xDEADBEEF + (k as u32);
            }

            let mut storage = [StorageDevice {
                id: 0,
                online: false,
                capacity_gb: 1024,
                used_gb: 0,
                temperature_celsius: 30,
                io_bandwidth_mbps: 100,
                assigned_vm: None,
                readonly: false,
                encrypted: false,
            }; MAX_STORAGE_PER_NODE];
            for s in 0..MAX_STORAGE_PER_NODE {
                storage[s].id = s;
                storage[s].online = true;
                storage[s].used_gb = s as u32 * 10;
            }

            NODES[i] = Some(Node {
                id: i,
                online: true,
                cpus,
                gpus,
                storage,
                metadata: "EXOVega Node Rev 1.3",
                thermal_celsius: 42,
                power_watts: 150,
                uptime_seconds: 0,
                io_bandwidth_mbps: 0,
                secure_boot_enabled: true,
                fan_rpm: 3000,
                rack_position: (i % 42) as u8,
                network_latency_ms: 1 + (i as u16 % 10),
                ipv6_enabled: true,
            });
            print_node(i, "[ExoNode Quantum] Node online");
        }
    }
}

// ---------------------------------
// VM Lifecycle Management
// ---------------------------------

fn create_vm(id: usize, node_id: usize, cpu_mask: u64) {
    unsafe {
        if id >= MAX_VMS || node_id >= MAX_NODES {
            print_line("[VM] Invalid VM or Node ID");
            return;
        }
        if VMS[id].is_some() {
            print_line("[VM] VM ID already exists");
            return;
        }

        VMS[id] = Some(VM {
            id,
            active: true,
            network_ip: Some(0xC0A80001 + (id as u32)),
            node_id,
            cpu_mask,
            state: VmState::Running,
            uptime_ticks: 0,
            label: "GenericVM",
            memory_allocated_mb: 512,
            snapshot_enabled: false,
            encrypted: false,
            migration_target: None,
            high_availability: true,
            auto_restart: true,
            priority_class: 1,
            backups_enabled: true,
            restore_point: None,
        });

        print_vm(id, "Created with CPU mask");
    }
}

fn delete_vm(id: usize) {
    unsafe {
        if let Some(_) = VMS[id].take() {
            print_vm(id, "Deleted successfully");
        } else {
            print_line("[VM] VM does not exist");
        }
    }
}

fn list_vms() {
    unsafe {
        for i in 0..MAX_VMS {
            if let Some(vm) = &VMS[i] {
                print_line(&format!(
                    "VM {} ({} on Node {}): {:?} - IP: {} - CPUs: {:064b} - Mem: {}MB - Uptime: {} ticks - Snapshots: {} - Encrypted: {} - Migration Target: {:?} - HA: {} - AutoRestart: {} - Priority: {} - Backup: {}",
                    vm.id,
                    vm.label,
                    vm.node_id,
                    vm.state,
                    vm.network_ip.unwrap_or(0),
                    vm.cpu_mask,
                    vm.memory_allocated_mb,
                    vm.uptime_ticks,
                    if vm.snapshot_enabled { "Yes" } else { "No" },
                    if vm.encrypted { "Yes" } else { "No" },
                    vm.migration_target,
                    if vm.high_availability { "Yes" } else { "No" },
                    if vm.auto_restart { "Yes" } else { "No" },
                    vm.priority_class,
                    if vm.backups_enabled { "Yes" } else { "No" }
                ));
            }
        }
    }
}

// ---------------------------------
// Node & CPU Monitoring
// ---------------------------------

fn list_nodes() {
    unsafe {
        for i in 0..MAX_NODES {
            if let Some(node) = &NODES[i] {
                print_line(&format!(
                    "Node {}: {} | {} | {}C | {}W | Uptime: {}s | IO: {}Mbps | Fan: {} RPM | SecureBoot: {} | RackPos: {} | Latency: {}ms | IPv6: {}",
                    node.id,
                    if node.online { "Online" } else { "Offline" },
                    node.metadata,
                    node.thermal_celsius,
                    node.power_watts,
                    node.uptime_seconds,
                    node.io_bandwidth_mbps,
                    node.fan_rpm,
                    if node.secure_boot_enabled { "Yes" } else { "No" },
                    node.rack_position,
                    node.network_latency_ms,
                    if node.ipv6_enabled { "Yes" } else { "No" }
                ));
                for cpu in node.cpus.iter() {
                    print_line(&format!(
                        "  CPU {}: {} - {}% {} @ {}MHz",
                        cpu.id,
                        if cpu.online { "Online" } else { "Offline" },
                        cpu.usage_percent,
                        if cpu.throttled { "(Throttled)" } else { "" },
                        cpu.frequency_mhz
                    ));
                }
                for gpu in node.gpus.iter() {
                    print_line(&format!(
                        "  GPU {}: {} - {}% Load - {}C - {} RPM - Assigned VM: {:?}",
                        gpu.id,
                        if gpu.online { "Online" } else { "Offline" },
                        gpu.load_percent,
                        gpu.temperature_celsius,
                        gpu.fan_speed_rpm,
                        gpu.assigned_vm
                    ));
                }
                for storage in node.storage.iter() {
                    print_line(&format!(
                        "  Storage {}: {} - {}/{} GB - {}C - IO: {} Mbps - Assigned VM: {:?}",
                        storage.id,
                        if storage.online { "Online" } else { "Offline" },
                        storage.used_gb,
                        storage.capacity_gb,
                        storage.temperature_celsius,
                        storage.io_bandwidth_mbps,
                        storage.assigned_vm
                    ));
                }
            }
        }
    }
}

// ---------------------------------
// Telemetry and Resource Updates
// ---------------------------------

fn update_telemetry() {
    unsafe {
        for i in 0..MAX_VMS {
            if let Some(vm) = &mut VMS[i] {
                if vm.active {
                    vm.uptime_ticks = vm.uptime_ticks.wrapping_add(1);
                }
            }
        }
        for i in 0..MAX_NODES {
            if let Some(node) = &mut NODES[i] {
                node.uptime_seconds = node.uptime_seconds.wrapping_add(1);
                node.io_bandwidth_mbps = (node.io_bandwidth_mbps + 25) % 1000;
                node.fan_rpm = 2000 + ((node.uptime_seconds as u16) % 1000);
                node.network_latency_ms = 1 + ((node.network_latency_ms + 1) % 5);

                for cpu in node.cpus.iter_mut() {
                    if cpu.online {
                        cpu.usage_percent = (cpu.usage_percent + 13) % 100;
                        cpu.throttled = cpu.usage_percent > 90;
                        cpu.frequency_mhz = 2800 + (cpu.usage_percent as u16 % 600);
                    }
                }

                for gpu in node.gpus.iter_mut() {
                    gpu.load_percent = (gpu.load_percent + 7) % 100;
                    gpu.temperature_celsius = 40 + ((node.uptime_seconds % 10) as u8);
                    gpu.fan_speed_rpm = 1500 + ((node.uptime_seconds % 500) as u16);
                }

                for storage in node.storage.iter_mut() {
                    storage.used_gb = (storage.used_gb + 1) % storage.capacity_gb;
                    storage.temperature_celsius = 35 + ((node.uptime_seconds % 5) as u8);
                }

                thermal_throttle(i);
            }
        }
    }
}

fn thermal_throttle(node_id: usize) {
    unsafe {
        if let Some(node) = &mut NODES[node_id] {
            if node.thermal_celsius > 85 {
                print_line(&format!("Node {}: Thermal throttling engaged!", node_id));
                for cpu in node.cpus.iter_mut() {
                    cpu.throttled = true;
                    cpu.frequency_mhz = 1000; // throttle to 1GHz
                }
                for gpu in node.gpus.iter_mut() {
                    gpu.fan_speed_rpm = 5000;
                }
                // Additional throttling logic here
            }
        }
    }
}

// ---------------------------------
// VM Messaging System
// ---------------------------------

struct Message {
    sender_vm: usize,
    receiver_vm: usize,
    payload: &'static str,
    timestamp: u64,
}

fn send_vm_message(sender: usize, receiver: usize, payload: &'static str) {
    unsafe {
        let next_tail = (MSG_QUEUE_TAIL + 1) % MAX_MSG_QUEUE_SIZE;
        if next_tail == MSG_QUEUE_HEAD {
            log_event(AlertLevel::Warning, "Messaging", "Message queue full");
            return;
        }
        MESSAGE_QUEUE[MSG_QUEUE_TAIL] = Some(Message {
            sender_vm: sender,
            receiver_vm: receiver,
            payload,
            timestamp: get_system_ticks(),
        });
        MSG_QUEUE_TAIL = next_tail;
        log_event(AlertLevel::Info, "Messaging", "Message sent");
    }
}

fn receive_vm_message(vm_id: usize) -> Option<Message> {
    unsafe {
        if MSG_QUEUE_HEAD == MSG_QUEUE_TAIL {
            return None;
        }
        if let Some(msg) = &MESSAGE_QUEUE[MSG_QUEUE_HEAD] {
            if msg.receiver_vm == vm_id {
                let message = MESSAGE_QUEUE[MSG_QUEUE_HEAD].take();
                MSG_QUEUE_HEAD = (MSG_QUEUE_HEAD + 1) % MAX_MSG_QUEUE_SIZE;
                return message;
            }
        }
        None
    }
}

fn process_vm_messages() {
    // Process messages for all active VMs (simple demo)
    unsafe {
        for i in 0..MAX_VMS {
            if let Some(vm) = &VMS[i] {
                if vm.active {
                    if let Some(msg) = receive_vm_message(vm.id) {
                        print_line(&format!(
                            "VM {} received message from VM {}: {}",
                            vm.id, msg.sender_vm, msg.payload
                        ));
                    }
                }
            }
        }
    }
}

// ---------------------------------
// Scheduler & Resource Manager
// ---------------------------------

fn scheduler_tick() {
    // Simple round-robin scheduler example
    unsafe {
        for node_opt in NODES.iter_mut() {
            if let Some(node) = node_opt {
                for cpu in node.cpus.iter_mut() {
                    if cpu.online {
                        // Artificially simulate cpu usage update
                        cpu.usage_percent = (cpu.usage_percent + 5) % 100;
                        cpu.throttled = cpu.usage_percent > 90;
                        cpu.frequency_mhz = 2500 + (cpu.usage_percent as u16 % 700);
                    }
                }
            }
        }
    }
}

// ---------------------------------
// CLI & User Management (stub)
// ---------------------------------

fn cli_init() {
    print_line("[CLI] Initialized");
}

fn parse_cli_command(cmd: &str) {
    // Stub parser
    print_line(&format!("Received CLI command: {}", cmd));
}

// ---------------------------------
// Framebuffer & GUI (Stubbed)
// ---------------------------------

fn clear_screen(color: Color) {
    // Stub framebuffer clear
}

fn draw_text(x: usize, y: usize, text: &str, color: Color) {
    // Stub text draw
}

fn draw_dashboard() {
    draw_text(10, 10, "ExoNode Quantum Dashboard", 0x00FF00);
}

// ---------------------------------
// Print helpers (Stubbed UART/Console)
// ---------------------------------

fn print_line(msg: &str) {
    // Stub: replace with actual UART or framebuffer output
    // For now, just a placeholder
}

fn print_node(id: usize, msg: &str) {
    print_line(&format!("Node {}: {}", id, msg));
}

fn print_vm(id: usize, msg: &str) {
    print_line(&format!("VM {}: {}", id, msg));
}

fn get_system_ticks() -> u64 {
    SYSTEM_TICKS.load(Ordering::Relaxed)
}

fn poll_uart() {}
fn poll_mouse() {}
fn poll_cluster() {}
fn cleanup_resources() {}
// ---------------------------------
// CLI Command Parsing & Handling
// ---------------------------------

#[derive(Debug)]
enum CliCommand {
    Help,
    ListVms,
    ListNodes,
    CreateVm { id: usize, node_id: usize, cpu_mask: u64 },
    DeleteVm { id: usize },
    StartVm { id: usize },
    StopVm { id: usize },
    SnapshotVm { id: usize },
    MigrateVm { id: usize, target_node: usize },
    SendMessage { sender: usize, receiver: usize, msg: &'static str },
    Unknown,
}

fn parse_cli_command(cmd: &str) -> CliCommand {
    let tokens: Vec<&str> = cmd.trim().split_whitespace().collect();
    if tokens.is_empty() {
        return CliCommand::Unknown;
    }
    match tokens[0].to_lowercase().as_str() {
        "help" => CliCommand::Help,
        "listvms" => CliCommand::ListVms,
        "listnodes" => CliCommand::ListNodes,
        "createvm" => {
            if tokens.len() < 4 {
                return CliCommand::Unknown;
            }
            let id = atoi(tokens[1]);
            let node_id = atoi(tokens[2]);
            let cpu_mask = atoi64(tokens[3]);
            CliCommand::CreateVm { id, node_id, cpu_mask }
        }
        "deletevm" => {
            if tokens.len() < 2 {
                return CliCommand::Unknown;
            }
            let id = atoi(tokens[1]);
            CliCommand::DeleteVm { id }
        }
        "startvm" => {
            if tokens.len() < 2 {
                return CliCommand::Unknown;
            }
            let id = atoi(tokens[1]);
            CliCommand::StartVm { id }
        }
        "stopvm" => {
            if tokens.len() < 2 {
                return CliCommand::Unknown;
            }
            let id = atoi(tokens[1]);
            CliCommand::StopVm { id }
        }
        "snapshotvm" => {
            if tokens.len() < 2 {
                return CliCommand::Unknown;
            }
            let id = atoi(tokens[1]);
            CliCommand::SnapshotVm { id }
        }
        "migratevm" => {
            if tokens.len() < 3 {
                return CliCommand::Unknown;
            }
            let id = atoi(tokens[1]);
            let target_node = atoi(tokens[2]);
            CliCommand::MigrateVm { id, target_node }
        }
        "sendmsg" => {
            if tokens.len() < 4 {
                return CliCommand::Unknown;
            }
            let sender = atoi(tokens[1]);
            let receiver = atoi(tokens[2]);
            let msg = tokens[3..].join(" ");
            let static_msg: &'static str = Box::leak(msg.into_boxed_str()); // leak for static lifetime
            CliCommand::SendMessage { sender, receiver, msg: static_msg }
        }
        _ => CliCommand::Unknown,
    }
}

fn handle_cli_command(cmd: &str) {
    let command = parse_cli_command(cmd);
    match command {
        CliCommand::Help => {
            print_line("Available commands:");
            print_line("  help                 - Show this help");
            print_line("  listvms              - List all VMs");
            print_line("  listnodes            - List all nodes");
            print_line("  createvm ID NODE CPU - Create VM with ID, node, CPU mask");
            print_line("  deletevm ID          - Delete VM by ID");
            print_line("  startvm ID           - Start VM by ID");
            print_line("  stopvm ID            - Stop VM by ID");
            print_line("  snapshotvm ID        - Snapshot VM");
            print_line("  migratevm ID NODE    - Migrate VM to node");
            print_line("  sendmsg S R MSG      - Send message from S to R");
        }
        CliCommand::ListVms => list_vms(),
        CliCommand::ListNodes => list_nodes(),
        CliCommand::CreateVm { id, node_id, cpu_mask } => create_vm(id, node_id, cpu_mask),
        CliCommand::DeleteVm { id } => delete_vm(id),
        CliCommand::StartVm { id } => start_vm(id),
        CliCommand::StopVm { id } => stop_vm(id),
        CliCommand::SnapshotVm { id } => snapshot_vm(id),
        CliCommand::MigrateVm { id, target_node } => migrate_vm(id, target_node),
        CliCommand::SendMessage { sender, receiver, msg } => send_vm_message(sender, receiver, msg),
        CliCommand::Unknown => print_line("[CLI] Unknown or invalid command"),
    }
}

fn start_vm(id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[id] {
            vm.state = VmState::Running;
            vm.active = true;
            print_vm(id, "VM started");
        } else {
            print_line("[VM] VM does not exist");
        }
    }
}

fn stop_vm(id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[id] {
            vm.state = VmState::Stopped;
            vm.active = false;
            print_vm(id, "VM stopped");
        } else {
            print_line("[VM] VM does not exist");
        }
    }
}

fn snapshot_vm(id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[id] {
            if !vm.snapshot_enabled {
                vm.snapshot_enabled = true;
                vm.restore_point = Some(get_system_ticks());
                print_vm(id, "Snapshot created");
            } else {
                print_vm(id, "Snapshot already enabled");
            }
        } else {
            print_line("[VM] VM does not exist");
        }
    }
}

fn migrate_vm(id: usize, target_node: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[id] {
            if target_node >= MAX_NODES {
                print_line("[VM] Invalid target node");
                return;
            }
            vm.migration_target = Some(target_node);
            vm.state = VmState::Migrating;
            print_vm(id, &format!("Migration started to node {}", target_node));
        } else {
            print_line("[VM] VM does not exist");
        }
    }
}

// ---------------------------------
// Security & Authentication
// ---------------------------------

fn authenticate(username: &'static str, password: &'static str) -> bool {
    // Stub: Always true for demo
    print_line(&format!("[AUTH] User '{}' authenticated", username));
    unsafe {
        CURRENT_USER = Some(UserSession {
            username,
            role: Role::Admin,
            session_id: get_system_ticks(),
            authenticated: true,
        });
    }
    true
}

fn check_permission(required_role: Role) -> bool {
    unsafe {
        if let Some(session) = &CURRENT_USER {
            return session.authenticated && (session.role as u8 <= required_role as u8);
        }
    }
    false
}

fn logout() {
    unsafe {
        if let Some(session) = &CURRENT_USER {
            print_line(&format!("[AUTH] User '{}' logged out", session.username));
        }
        CURRENT_USER = None;
    }
}

// ---------------------------------
// Network Virtualization & Config
// ---------------------------------

#[repr(C)]
pub struct VirtualNIC {
    pub vm_id: usize,
    pub mac_address: [u8; 6],
    pub ip_address: Option<u32>,
    pub netmask: Option<u32>,
    pub gateway: Option<u32>,
    pub online: bool,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub vlan_tag: Option<u16>,
    pub firewall_enabled: bool,
    pub firewall_rules: [FirewallRule; MAX_FIREWALL_RULES],
}

#[repr(C)]
pub struct FirewallRule {
    pub id: usize,
    pub enabled: bool,
    pub action_allow: bool,
    pub src_ip: Option<u32>,
    pub dest_ip: Option<u32>,
    pub protocol: Option<u8>, // TCP=6, UDP=17, ICMP=1
    pub src_port: Option<u16>,
    pub dest_port: Option<u16>,
}

const MAX_VNICS: usize = 512;
const MAX_FIREWALL_RULES: usize = 64;

static mut VIRTUAL_NICS: [Option<VirtualNIC>; MAX_VNICS] = [None; MAX_VNICS];

fn create_virtual_nic(vm_id: usize, mac: [u8; 6]) -> Result<(), &'static str> {
    unsafe {
        for i in 0..MAX_VNICS {
            if VIRTUAL_NICS[i].is_none() {
                VIRTUAL_NICS[i] = Some(VirtualNIC {
                    vm_id,
                    mac_address: mac,
                    ip_address: None,
                    netmask: None,
                    gateway: None,
                    online: true,
                    rx_bytes: 0,
                    tx_bytes: 0,
                    vlan_tag: None,
                    firewall_enabled: false,
                    firewall_rules: [FirewallRule {
                        id: 0,
                        enabled: false,
                        action_allow: true,
                        src_ip: None,
                        dest_ip: None,
                        protocol: None,
                        src_port: None,
                        dest_port: None,
                    }; MAX_FIREWALL_RULES],
                });
                print_vm(vm_id, &format!("Virtual NIC created with MAC {:02x?}", mac));
                return Ok(());
            }
        }
    }
    Err("No free VNIC slots")
}

fn delete_virtual_nic(vm_id: usize) {
    unsafe {
        for i in 0..MAX_VNICS {
            if let Some(nic) = &VIRTUAL_NICS[i] {
                if nic.vm_id == vm_id {
                    VIRTUAL_NICS[i] = None;
                    print_vm(vm_id, "Virtual NIC deleted");
                    return;
                }
            }
        }
    }
}

// ---------------------------------
// Storage Management
// ---------------------------------

fn attach_storage_to_vm(node_id: usize, storage_id: usize, vm_id: usize) {
    unsafe {
        if let Some(node) = &mut NODES[node_id] {
            if storage_id < MAX_STORAGE_PER_NODE {
                let storage = &mut node.storage[storage_id];
                if storage.assigned_vm.is_none() {
                    storage.assigned_vm = Some(vm_id);
                    print_vm(vm_id, &format!("Storage device {} attached", storage_id));
                } else {
                    print_vm(vm_id, "Storage device already assigned");
                }
            }
        }
    }
}

fn detach_storage_from_vm(node_id: usize, storage_id: usize) {
    unsafe {
        if let Some(node) = &mut NODES[node_id] {
            if storage_id < MAX_STORAGE_PER_NODE {
                let storage = &mut node.storage[storage_id];
                if storage.assigned_vm.is_some() {
                    print_line(&format!("Storage device {} detached from VM", storage_id));
                    storage.assigned_vm = None;
                }
            }
        }
    }
}

// ---------------------------------
// GPU Virtualization
// ---------------------------------

fn assign_gpu_to_vm(node_id: usize, gpu_id: usize, vm_id: usize) {
    unsafe {
        if let Some(node) = &mut NODES[node_id] {
            if gpu_id < MAX_GPUS_PER_NODE {
                let gpu = &mut node.gpus[gpu_id];
                if gpu.assigned_vm.is_none() && gpu.online {
                    gpu.assigned_vm = Some(vm_id);
                    print_vm(vm_id, &format!("GPU {} assigned", gpu_id));
                } else {
                    print_vm(vm_id, "GPU already assigned or offline");
                }
            }
        }
    }
}

fn release_gpu_from_vm(node_id: usize, gpu_id: usize) {
    unsafe {
        if let Some(node) = &mut NODES[node_id] {
            if gpu_id < MAX_GPUS_PER_NODE {
                let gpu = &mut node.gpus[gpu_id];
                gpu.assigned_vm = None;
                print_line(&format!("GPU {} released", gpu_id));
            }
        }
    }
}

// ---------------------------------
// Logging and Alerts
// ---------------------------------

fn log_event(level: AlertLevel, subsystem: &str, message: &str) {
    let prefix = match level {
        AlertLevel::Info => "[INFO]",
        AlertLevel::Warning => "[WARN]",
        AlertLevel::Error => "[ERROR]",
        AlertLevel::Critical => "[CRIT]",
    };
    print_line(&format!("{} [{}]: {}", prefix, subsystem, message));
}

// ---------------------------------
// Event Loop & Polling (expanded)
// ---------------------------------

fn poll_uart() {
    // Stub: listen for CLI input lines
    // Example usage:
    // let input = uart_read_line();
    // if let Some(line) = input {
    //     handle_cli_command(&line);
    // }
}

fn poll_mouse() {
    // Stub: Process mouse input for GUI
}

fn poll_cluster() {
    // Stub: Check cluster node statuses, failures, etc.
}

fn cleanup_resources() {
    // Stub: Garbage collect unused resources, expired snapshots, etc.
}

// ---------------------------------
// Advanced Telemetry & Alerting
// ---------------------------------

fn monitor_thermal_conditions() {
    unsafe {
        for i in 0..MAX_NODES {
            if let Some(node) = &NODES[i] {
                if node.thermal_celsius > 90 {
                    log_event(AlertLevel::Critical, "Thermal", &format!("Node {} overheating: {}C", node.id, node.thermal_celsius));
                    thermal_throttle(i);
                } else if node.thermal_celsius > 75 {
                    log_event(AlertLevel::Warning, "Thermal", &format!("Node {} high temp: {}C", node.id, node.thermal_celsius));
                }
            }
        }
    }
}

// ---------------------------------
// Network Firewall Rule Management
// ---------------------------------

fn add_firewall_rule(vm_id: usize, rule: FirewallRule) -> Result<(), &'static str> {
    unsafe {
        for nic_opt in VIRTUAL_NICS.iter_mut() {
            if let Some(nic) = nic_opt {
                if nic.vm_id == vm_id {
                    for i in 0..MAX_FIREWALL_RULES {
                        if !nic.firewall_rules[i].enabled {
                            nic.firewall_rules[i] = rule;
                            nic.firewall_rules[i].enabled = true;
                            print_vm(vm_id, "Firewall rule added");
                            return Ok(());
                        }
                    }
                    return Err("Firewall rule table full");
                }
            }
        }
    }
    Err("VM NIC not found")
}

fn remove_firewall_rule(vm_id: usize, rule_id: usize) -> Result<(), &'static str> {
    unsafe {
        for nic_opt in VIRTUAL_NICS.iter_mut() {
            if let Some(nic) = nic_opt {
                if nic.vm_id == vm_id {
                    for i in 0..MAX_FIREWALL_RULES {
                        if nic.firewall_rules[i].enabled && nic.firewall_rules[i].id == rule_id {
                            nic.firewall_rules[i].enabled = false;
                            print_vm(vm_id, "Firewall rule removed");
                            return Ok(());
                        }
                    }
                    return Err("Firewall rule not found");
                }
            }
        }
    }
    Err("VM NIC not found")
}
// ---------------------------------
// Storage Snapshot & Deduplication
// ---------------------------------

const MAX_SNAPSHOTS_PER_VM: usize = 16;
const MAX_STORAGE_PER_NODE: usize = 32;
const MAX_GPUS_PER_NODE: usize = 8;

#[repr(C)]
pub struct Snapshot {
    id: usize,
    timestamp_ticks: u64,
    size_mb: u32,
    parent_snapshot: Option<usize>,
    active: bool,
}

#[repr(C)]
pub struct StorageDevice {
    id: usize,
    size_mb: u32,
    assigned_vm: Option<usize>,
    read_iops: u32,
    write_iops: u32,
    snapshots: [Option<Snapshot>; MAX_SNAPSHOTS_PER_VM],
    dedup_enabled: bool,
    compression_enabled: bool,
}

#[repr(C)]
pub struct GPUDevice {
    id: usize,
    online: bool,
    assigned_vm: Option<usize>,
    memory_mb: u32,
    cores: u16,
    frequency_mhz: u16,
    driver_version: &'static str,
}

impl Node {
    storage: [StorageDevice; MAX_STORAGE_PER_NODE],
    gpus: [GPUDevice; MAX_GPUS_PER_NODE],
}

fn create_snapshot(vm_id: usize, snapshot_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if let Some(node) = &mut NODES[vm.node_id] {
                for storage in node.storage.iter_mut() {
                    if storage.assigned_vm == Some(vm_id) {
                        for i in 0..MAX_SNAPSHOTS_PER_VM {
                            if storage.snapshots[i].is_none() {
                                storage.snapshots[i] = Some(Snapshot {
                                    id: snapshot_id,
                                    timestamp_ticks: get_system_ticks(),
                                    size_mb: storage.size_mb / 4, // Example: snapshot uses 25% storage
                                    parent_snapshot: None,
                                    active: true,
                                });
                                print_vm(vm_id, &format!("Snapshot {} created on storage {}", snapshot_id, storage.id));
                                return;
                            }
                        }
                    }
                }
            }
            print_vm(vm_id, "No storage attached or snapshot limit reached");
        }
    }
}

fn delete_snapshot(vm_id: usize, snapshot_id: usize) {
    unsafe {
        if let Some(vm) = &VMS[vm_id] {
            if let Some(node) = &mut NODES[vm.node_id] {
                for storage in node.storage.iter_mut() {
                    if storage.assigned_vm == Some(vm_id) {
                        for i in 0..MAX_SNAPSHOTS_PER_VM {
                            if let Some(snapshot) = &storage.snapshots[i] {
                                if snapshot.id == snapshot_id {
                                    storage.snapshots[i] = None;
                                    print_vm(vm_id, &format!("Snapshot {} deleted on storage {}", snapshot_id, storage.id));
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            print_vm(vm_id, "Snapshot not found");
        }
    }
}

// ---------------------------------
// Network Virtualization Core
// ---------------------------------

#[repr(C)]
pub struct VirtualSwitch {
    id: usize,
    connected_nics: [Option<usize>; MAX_VNICS], // indices in VIRTUAL_NICS
    vlan_tags: [Option<u16>; MAX_VNICS],
    mac_table: [Option<[u8;6]>; MAX_VNICS], // Learned MAC addresses per port
}

const MAX_VIRTUAL_SWITCHES: usize = 16;
static mut VIRTUAL_SWITCHES: [Option<VirtualSwitch>; MAX_VIRTUAL_SWITCHES] = [None; MAX_VIRTUAL_SWITCHES];

fn create_virtual_switch(id: usize) -> Result<(), &'static str> {
    unsafe {
        if id >= MAX_VIRTUAL_SWITCHES {
            return Err("Virtual switch ID too high");
        }
        if VIRTUAL_SWITCHES[id].is_some() {
            return Err("Virtual switch already exists");
        }
        VIRTUAL_SWITCHES[id] = Some(VirtualSwitch {
            id,
            connected_nics: [None; MAX_VNICS],
            vlan_tags: [None; MAX_VNICS],
            mac_table: [None; MAX_VNICS],
        });
        print_line(&format!("[VSwitch] Created virtual switch {}", id));
        Ok(())
    }
}

fn connect_nic_to_switch(switch_id: usize, nic_index: usize, vlan_tag: Option<u16>) -> Result<(), &'static str> {
    unsafe {
        if switch_id >= MAX_VIRTUAL_SWITCHES {
            return Err("Invalid switch ID");
        }
        let vswitch = match &mut VIRTUAL_SWITCHES[switch_id] {
            Some(s) => s,
            None => return Err("Virtual switch not found"),
        };
        for i in 0..MAX_VNICS {
            if vswitch.connected_nics[i].is_none() {
                vswitch.connected_nics[i] = Some(nic_index);
                vswitch.vlan_tags[i] = vlan_tag;
                print_line(&format!("[VSwitch] NIC {} connected to switch {}", nic_index, switch_id));
                return Ok(());
            }
        }
        Err("Virtual switch port table full")
    }
}

fn disconnect_nic_from_switch(switch_id: usize, nic_index: usize) -> Result<(), &'static str> {
    unsafe {
        if switch_id >= MAX_VIRTUAL_SWITCHES {
            return Err("Invalid switch ID");
        }
        let vswitch = match &mut VIRTUAL_SWITCHES[switch_id] {
            Some(s) => s,
            None => return Err("Virtual switch not found"),
        };
        for i in 0..MAX_VNICS {
            if vswitch.connected_nics[i] == Some(nic_index) {
                vswitch.connected_nics[i] = None;
                vswitch.vlan_tags[i] = None;
                vswitch.mac_table[i] = None;
                print_line(&format!("[VSwitch] NIC {} disconnected from switch {}", nic_index, switch_id));
                return Ok(());
            }
        }
        Err("NIC not connected to switch")
    }
}

fn forward_packet(vswitch: &mut VirtualSwitch, src_nic: usize, dest_mac: [u8; 6], vlan_tag: Option<u16>, packet: &[u8]) {
    // Learn source MAC on src_nic port
    vswitch.mac_table[src_nic] = Some(dest_mac);

    // Forward packet based on MAC table and VLAN tags (simplified)
    for (port_idx, nic_opt) in vswitch.connected_nics.iter().enumerate() {
        if let Some(nic_index) = nic_opt {
            if *nic_index != src_nic {
                // VLAN tag filtering
                if vswitch.vlan_tags[port_idx] == vlan_tag {
                    // send packet out to nic_index (stubbed)
                    print_line(&format!("[VSwitch] Forwarding packet from NIC {} to NIC {}", src_nic, nic_index));
                }
            }
        }
    }
}

// ---------------------------------
// VM Migration Workflow
// ---------------------------------

fn migrate_vm_process(vm_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if vm.state != VmState::Migrating {
                return;
            }
            let target_node = match vm.migration_target {
                Some(n) => n,
                None => {
                    print_vm(vm_id, "Migration target not set");
                    vm.state = VmState::Running;
                    return;
                }
            };
            if target_node >= MAX_NODES {
                print_vm(vm_id, "Invalid migration target node");
                vm.state = VmState::Running;
                return;
            }

            // Simulate migration delay & steps
            print_vm(vm_id, &format!("Migrating VM {} to node {}", vm_id, target_node));

            // Step 1: Freeze VM
            vm.state = VmState::Suspended;
            // Step 2: Transfer memory and state (stub)
            // Step 3: Update node_id
            vm.node_id = target_node;
            // Step 4: Resume VM
            vm.state = VmState::Running;
            vm.migration_target = None;
            print_vm(vm_id, "Migration complete");
        }
    }
}

// ---------------------------------
// Advanced Scheduler with Priorities & Preemption
// ---------------------------------

#[repr(C)]
pub struct Scheduler {
    current_vm_id: Option<usize>,
    vm_queue: [Option<usize>; MAX_VMS],
    priorities: [u8; MAX_VMS],
    time_slice_ticks: u64,
    last_switch_tick: u64,
}

static mut SCHEDULER: Scheduler = Scheduler {
    current_vm_id: None,
    vm_queue: [None; MAX_VMS],
    priorities: [0; MAX_VMS],
    time_slice_ticks: 100,
    last_switch_tick: 0,
};

fn scheduler_add_vm(vm_id: usize, priority: u8) {
    unsafe {
        for slot in SCHEDULER.vm_queue.iter_mut() {
            if slot.is_none() {
                *slot = Some(vm_id);
                SCHEDULER.priorities[vm_id] = priority;
                print_vm(vm_id, &format!("Added to scheduler queue with priority {}", priority));
                return;
            }
        }
        print_vm(vm_id, "Scheduler queue full");
    }
}

fn scheduler_tick() {
    unsafe {
        let current_tick = get_system_ticks();
        if current_tick - SCHEDULER.last_switch_tick < SCHEDULER.time_slice_ticks {
            return; // time slice not expired
        }

        // Find next VM with highest priority
        let mut next_vm_id = None;
        let mut highest_priority = 0;
        for &vm_opt in SCHEDULER.vm_queue.iter() {
            if let Some(vm_id) = vm_opt {
                if let Some(vm) = &VMS[vm_id] {
                    if vm.active && vm.state == VmState::Running {
                        let p = SCHEDULER.priorities[vm_id];
                        if p > highest_priority {
                            highest_priority = p;
                            next_vm_id = Some(vm_id);
                        }
                    }
                }
            }
        }

        if let Some(next) = next_vm_id {
            SCHEDULER.current_vm_id = Some(next);
            SCHEDULER.last_switch_tick = current_tick;
            print_vm(next, "Scheduled to run");
            // Simulate VM running on CPU(s)
        }
    }
}

// ---------------------------------
// Framebuffer GUI Windows & Input
// ---------------------------------

#[repr(C)]
pub struct Window {
    id: usize,
    title: &'static str,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    visible: bool,
    focused: bool,
}

const MAX_WINDOWS: usize = 32;
static mut WINDOWS: [Option<Window>; MAX_WINDOWS] = [None; MAX_WINDOWS];

fn create_window(title: &'static str, x: usize, y: usize, width: usize, height: usize) -> usize {
    unsafe {
        for i in 0..MAX_WINDOWS {
            if WINDOWS[i].is_none() {
                WINDOWS[i] = Some(Window {
                    id: i,
                    title,
                    x,
                    y,
                    width,
                    height,
                    visible: true,
                    focused: false,
                });
                print_line(&format!("[GUI] Window '{}' created with ID {}", title, i));
                return i;
            }
        }
    }
    0 // failed
}

fn draw_window(window: &Window) {
    // Stub: draw window frame, title bar, contents
    print_line(&format!("[GUI] Drawing window '{}'", window.title));
}

fn process_mouse_click(x: usize, y: usize) {
    unsafe {
        for window_opt in WINDOWS.iter_mut() {
            if let Some(win) = window_opt {
                if win.visible && x >= win.x && x <= win.x + win.width && y >= win.y && y <= win.y + win.height {
                    win.focused = true;
                    print_line(&format!("[GUI] Window '{}' focused by click", win.title));
                    break;
                }
            }
        }
    }
}

// ---------------------------------
// Role-Based Access Control (RBAC)
// ---------------------------------

#[repr(u8)]
#[derive(PartialEq, PartialOrd, Copy, Clone)]
enum Role {
    Guest = 3,
    User = 2,
    Admin = 1,
    SuperAdmin = 0,
}

#[repr(C)]
pub struct UserSession {
    username: &'static str,
    role: Role,
    session_id: u64,
    authenticated: bool,
}

static mut CURRENT_USER: Option<UserSession> = None;

fn enforce_role(required: Role) -> bool {
    unsafe {
        if let Some(session) = &CURRENT_USER {
            return session.authenticated && (session.role <= required);
        }
    }
    false
}

// ---------------------------------
// VM Checkpoint Restore System
// ---------------------------------

fn restore_vm_checkpoint(vm_id: usize, snapshot_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if let Some(node) = &NODES[vm.node_id] {
                for storage in node.storage.iter() {
                    if storage.assigned_vm == Some(vm_id) {
                        for snapshot_opt in storage.snapshots.iter() {
                            if let Some(snapshot) = snapshot_opt {
                                if snapshot.id == snapshot_id {
                                    print_vm(vm_id, &format!("Restoring VM from snapshot {}", snapshot_id));
                                    // Stub: load snapshot data into VM memory
                                    vm.state = VmState::Running;
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            print_vm(vm_id, "Snapshot not found for restore");
        }
    }
}

// ---------------------------------
// Misc Utilities
// ---------------------------------

fn atoi(s: &str) -> usize {
    s.parse::<usize>().unwrap_or(0)
}

fn atoi64(s: &str) -> u64 {
    s.parse::<u64>().unwrap_or(0)
}

// ---------------------------------
// Panic Handler
// ---------------------------------

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
// ---------------------------------
// Virtual Router & DHCP Server
// ---------------------------------

const MAX_ROUTER_INTERFACES: usize = 8;
const MAX_DHCP_CLIENTS: usize = 64;

#[repr(C)]
pub struct RouterInterface {
    id: usize,
    node_id: usize,
    ip_addr: u32,
    subnet_mask: u32,
    mac_addr: [u8; 6],
    enabled: bool,
}

#[repr(C)]
pub struct DHCPClient {
    mac_addr: [u8; 6],
    ip_addr: u32,
    lease_expiry_tick: u64,
}

#[repr(C)]
pub struct VirtualRouter {
    id: usize,
    interfaces: [Option<RouterInterface>; MAX_ROUTER_INTERFACES],
    routing_table: [Option<RoutingEntry>; 64],
    dhcp_clients: [Option<DHCPClient>; MAX_DHCP_CLIENTS],
}

#[repr(C)]
pub struct RoutingEntry {
    destination: u32,
    subnet_mask: u32,
    gateway: u32,
    interface_id: usize,
    metric: u8,
    active: bool,
}

static mut VIRTUAL_ROUTERS: [Option<VirtualRouter>; 8] = [None; 8];

fn create_virtual_router(router_id: usize) {
    unsafe {
        if router_id >= VIRTUAL_ROUTERS.len() {
            print_line("[Router] Invalid router ID");
            return;
        }
        if VIRTUAL_ROUTERS[router_id].is_some() {
            print_line("[Router] Router already exists");
            return;
        }
        VIRTUAL_ROUTERS[router_id] = Some(VirtualRouter {
            id: router_id,
            interfaces: [None; MAX_ROUTER_INTERFACES],
            routing_table: [None; 64],
            dhcp_clients: [None; MAX_DHCP_CLIENTS],
        });
        print_line(&format!("[Router] Created virtual router {}", router_id));
    }
}

fn add_router_interface(router_id: usize, iface: RouterInterface) -> Result<(), &'static str> {
    unsafe {
        if let Some(router) = &mut VIRTUAL_ROUTERS[router_id] {
            for slot in router.interfaces.iter_mut() {
                if slot.is_none() {
                    *slot = Some(iface);
                    print_line(&format!("[Router] Interface {} added to router {}", iface.id, router_id));
                    return Ok(());
                }
            }
            Err("Router interfaces full")
        } else {
            Err("Router not found")
        }
    }
}

fn process_dhcp_request(router_id: usize, client_mac: [u8; 6]) -> Option<u32> {
    unsafe {
        let router = VIRTUAL_ROUTERS[router_id].as_mut()?;
        for client_opt in router.dhcp_clients.iter_mut() {
            if let Some(client) = client_opt {
                if client.mac_addr == client_mac {
                    // Renew lease
                    client.lease_expiry_tick = get_system_ticks() + 86400; // 1 day lease
                    print_line("[DHCP] Lease renewed");
                    return Some(client.ip_addr);
                }
            }
        }
        // Assign new IP
        for client_opt in router.dhcp_clients.iter_mut() {
            if client_opt.is_none() {
                let ip_base = 0xC0A80064; // 192.168.0.100 base
                let assigned_ip = ip_base + (router.dhcp_clients.iter().position(|c| c.is_none()).unwrap() as u32);
                *client_opt = Some(DHCPClient {
                    mac_addr: client_mac,
                    ip_addr: assigned_ip,
                    lease_expiry_tick: get_system_ticks() + 86400,
                });
                print_line(&format!("[DHCP] Assigned IP {:X} to MAC {:X?}", assigned_ip, client_mac));
                return Some(assigned_ip);
            }
        }
        None
    }
}

fn add_routing_entry(router_id: usize, entry: RoutingEntry) -> Result<(), &'static str> {
    unsafe {
        if let Some(router) = &mut VIRTUAL_ROUTERS[router_id] {
            for slot in router.routing_table.iter_mut() {
                if slot.is_none() {
                    *slot = Some(entry);
                    print_line(&format!("[Router] Added routing entry to router {}", router_id));
                    return Ok(());
                }
            }
            Err("Routing table full")
        } else {
            Err("Router not found")
        }
    }
}

fn route_packet(router_id: usize, dest_ip: u32) -> Option<usize> {
    unsafe {
        let router = VIRTUAL_ROUTERS[router_id].as_ref()?;
        let mut best_match: Option<&RoutingEntry> = None;
        for entry_opt in router.routing_table.iter() {
            if let Some(entry) = entry_opt {
                if entry.active && (dest_ip & entry.subnet_mask) == (entry.destination & entry.subnet_mask) {
                    if let Some(current_best) = &best_match {
                        if entry.metric < current_best.metric {
                            best_match = Some(entry);
                        }
                    } else {
                        best_match = Some(entry);
                    }
                }
            }
        }
        best_match.map(|e| e.interface_id)
    }
}

// ---------------------------------
// VM I/O Virtualization: Block and Network Devices
// ---------------------------------

#[repr(C)]
pub struct BlockDevice {
    id: usize,
    size_mb: u64,
    assigned_vm: Option<usize>,
    read_latency_ms: u8,
    write_latency_ms: u8,
    io_queues: u8,
    cache_enabled: bool,
}

#[repr(C)]
pub struct NetworkInterface {
    id: usize,
    mac_addr: [u8; 6],
    assigned_vm: Option<usize>,
    rx_packets: u64,
    tx_packets: u64,
    rx_errors: u32,
    tx_errors: u32,
    link_speed_mbps: u32,
    promiscuous_mode: bool,
}

const MAX_BLOCK_DEVICES: usize = 64;
const MAX_NETWORK_INTERFACES: usize = 128;

static mut BLOCK_DEVICES: [Option<BlockDevice>; MAX_BLOCK_DEVICES] = [None; MAX_BLOCK_DEVICES];
static mut NETWORK_INTERFACES: [Option<NetworkInterface>; MAX_NETWORK_INTERFACES] = [None; MAX_NETWORK_INTERFACES];

fn attach_block_device(vm_id: usize, device_id: usize) -> Result<(), &'static str> {
    unsafe {
        if vm_id >= MAX_VMS || device_id >= MAX_BLOCK_DEVICES {
            return Err("Invalid VM or device ID");
        }
        if let Some(device) = &mut BLOCK_DEVICES[device_id] {
            if device.assigned_vm.is_some() {
                return Err("Device already assigned");
            }
            device.assigned_vm = Some(vm_id);
            print_vm(vm_id, &format!("Block device {} attached", device_id));
            Ok(())
        } else {
            Err("Device not found")
        }
    }
}

fn detach_block_device(vm_id: usize, device_id: usize) -> Result<(), &'static str> {
    unsafe {
        if let Some(device) = &mut BLOCK_DEVICES[device_id] {
            if device.assigned_vm == Some(vm_id) {
                device.assigned_vm = None;
                print_vm(vm_id, &format!("Block device {} detached", device_id));
                return Ok(());
            }
            Err("Device not assigned to VM")
        } else {
            Err("Device not found")
        }
    }
}

fn attach_network_interface(vm_id: usize, nic_id: usize) -> Result<(), &'static str> {
    unsafe {
        if vm_id >= MAX_VMS || nic_id >= MAX_NETWORK_INTERFACES {
            return Err("Invalid VM or NIC ID");
        }
        if let Some(nic) = &mut NETWORK_INTERFACES[nic_id] {
            if nic.assigned_vm.is_some() {
                return Err("NIC already assigned");
            }
            nic.assigned_vm = Some(vm_id);
            print_vm(vm_id, &format!("NIC {} attached", nic_id));
            Ok(())
        } else {
            Err("NIC not found")
        }
    }
}

fn detach_network_interface(vm_id: usize, nic_id: usize) -> Result<(), &'static str> {
    unsafe {
        if let Some(nic) = &mut NETWORK_INTERFACES[nic_id] {
            if nic.assigned_vm == Some(vm_id) {
                nic.assigned_vm = None;
                print_vm(vm_id, &format!("NIC {} detached", nic_id));
                return Ok(());
            }
            Err("NIC not assigned to VM")
        } else {
            Err("NIC not found")
        }
    }
}

// Simulated I/O processing for block and network devices
fn process_io() {
    unsafe {
        for device_opt in BLOCK_DEVICES.iter_mut() {
            if let Some(device) = device_opt {
                if let Some(vm_id) = device.assigned_vm {
                    // Simulate some IO activity increment
                    print_vm(vm_id, &format!("Processing I/O on block device {}", device.id));
                }
            }
        }
        for nic_opt in NETWORK_INTERFACES.iter_mut() {
            if let Some(nic) = nic_opt {
                if let Some(vm_id) = nic.assigned_vm {
                    nic.rx_packets += 10;
                    nic.tx_packets += 8;
                    print_vm(vm_id, &format!("Processing packets on NIC {}", nic.id));
                }
            }
        }
    }
}

// ---------------------------------
// System Tick & Utilities
// ---------------------------------

static mut SYSTEM_TICKS: u64 = 0;

fn get_system_ticks() -> u64 {
    unsafe { SYSTEM_TICKS }
}

fn increment_system_ticks() {
    unsafe {
        SYSTEM_TICKS += 1;
    }
}

// ---------------------------------
// Command Line Interface Enhancements
// ---------------------------------

fn handle_command(cmd: &str) {
    let args: Vec<&str> = cmd.trim().split_whitespace().collect();
    if args.is_empty() {
        return;
    }

    match args[0] {
        "list-vms" => list_vms(),
        "list-nodes" => list_nodes(),
        "create-vm" => {
            if args.len() >= 4 {
                let id = atoi(args[1]);
                let node_id = atoi(args[2]);
                let cpu_mask = u64::from_str_radix(args[3], 16).unwrap_or(0);
                create_vm(id, node_id, cpu_mask);
            } else {
                print_line("Usage: create-vm <id> <node_id> <cpu_mask_hex>");
            }
        }
        "delete-vm" => {
            if args.len() >= 2 {
                let id = atoi(args[1]);
                delete_vm(id);
            } else {
                print_line("Usage: delete-vm <id>");
            }
        }
        "attach-block" => {
            if args.len() >= 3 {
                let vm_id = atoi(args[1]);
                let dev_id = atoi(args[2]);
                match attach_block_device(vm_id, dev_id) {
                    Ok(_) => (),
                    Err(e) => print_line(e),
                }
            } else {
                print_line("Usage: attach-block <vm_id> <device_id>");
            }
        }
        "detach-block" => {
            if args.len() >= 3 {
                let vm_id = atoi(args[1]);
                let dev_id = atoi(args[2]);
                match detach_block_device(vm_id, dev_id) {
                    Ok(_) => (),
                    Err(e) => print_line(e),
                }
            } else {
                print_line("Usage: detach-block <vm_id> <device_id>");
            }
        }
        "attach-nic" => {
            if args.len() >= 3 {
                let vm_id = atoi(args[1]);
                let nic_id = atoi(args[2]);
                match attach_network_interface(vm_id, nic_id) {
                    Ok(_) => (),
                    Err(e) => print_line(e),
                }
            } else {
                print_line("Usage: attach-nic <vm_id> <nic_id>");
            }
        }
        "detach-nic" => {
            if args.len() >= 3 {
                let vm_id = atoi(args[1]);
                let nic_id = atoi(args[2]);
                match detach_network_interface(vm_id, nic_id) {
                    Ok(_) => (),
                    Err(e) => print_line(e),
                }
            } else {
                print_line("Usage: detach-nic <vm_id> <nic_id>");
            }
        }
        "help" => {
            print_line("Available commands:");
            print_line("  list-vms");
            print_line("  list-nodes");
            print_line("  create-vm <id> <node_id> <cpu_mask_hex>");
            print_line("  delete-vm <id>");
            print_line("  attach-block <vm_id> <device_id>");
            print_line("  detach-block <vm_id> <device_id>");
            print_line("  attach-nic <vm_id> <nic_id>");
            print_line("  detach-nic <vm_id> <nic_id>");
            print_line("  help");
        }
        _ => print_line("Unknown command. Type 'help'"),
    }
}

// ---------------------------------
// Main Loop Extension to Process Ticks and I/O
// ---------------------------------

#[no_mangle]
pub extern "C" fn _start() -> ! {
    uart_init();
    framebuffer_init();
    pcie_init();
    discover_nodes();
    discover_storage_devices();
    discover_network_interfaces();
    hypervisor_init();
    startup_screen();
    start_event_loop();

    loop {
        poll_uart();
        poll_mouse();
        poll_cluster();
        update_telemetry();
        process_io();
        scheduler_tick();
        migrate_vms_tick();
        increment_system_ticks();
    }
}

fn migrate_vms_tick() {
    unsafe {
        for i in 0..MAX_VMS {
            if let Some(vm) = &VMS[i] {
                if vm.state == VmState::Migrating {
                    migrate_vm_process(i);
                }
            }
        }
    }
}

fn discover_storage_devices() {
    unsafe {
        for i in 0..MAX_NODES {
            if let Some(node) = &mut NODES[i] {
                for j in 0..MAX_STORAGE_PER_NODE {
                    node.storage[j] = Some(StorageDevice {
                        id: j,
                        size_mb: 1024 * 10,
                        assigned_vm: None,
                        read_iops: 10000,
                        write_iops: 8000,
                        snapshots: [None; MAX_SNAPSHOTS_PER_VM],
                        dedup_enabled: true,
                        compression_enabled: true,
                    });
                }
            }
        }
        print_line("[Storage] Storage devices discovered");
    }
}

fn discover_network_interfaces() {
    unsafe {
        for i in 0..MAX_NETWORK_INTERFACES {
            NETWORK_INTERFACES[i] = Some(NetworkInterface {
                id: i,
                mac_addr: generate_mac_address(i),
                assigned_vm: None,
                rx_packets: 0,
                tx_packets: 0,
                rx_errors: 0,
                tx_errors: 0,
                link_speed_mbps: 1000,
                promiscuous_mode: false,
            });
        }
        print_line("[Network] Network interfaces discovered");
    }
}

fn generate_mac_address(index: usize) -> [u8; 6] {
    // Locally administered MAC, unicast
    let base: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut mac = base;
    mac[5] = index as u8;
    mac
}

// ---------------------------------
// Stubs for Initialization and Polling
// ---------------------------------

fn uart_init() {
    print_line("[UART] Initialized");
}

fn poll_uart() {
    if let Some(c) = uart_getc() {
        unsafe {
            if c == b'\r' || c == b'\n' {
                CLI_BUFFER[CLI_INDEX] = 0;
                handle_command(core::str::from_utf8_unchecked(&CLI_BUFFER[..CLI_INDEX]));
                CLI_INDEX = 0;
                print_line("");
                print("> ");
            } else if CLI_INDEX < 127 {
                uart_putc(c);
                CLI_BUFFER[CLI_INDEX] = c;
                CLI_INDEX += 1;
            }
        }
    }
}

fn uart_getc() -> Option<u8> {
    None
}

fn uart_putc(_c: u8) {}

fn poll_mouse() {}

fn poll_cluster() {}

fn framebuffer_init() {
    clear_screen(0x101010);
    draw_dashboard();
}

fn startup_screen() {
    print_line("[System] Starting ExoNode Quantum...");
}

fn start_event_loop() {}

fn print_line(msg: &str) {
    for c in msg.as_bytes() {
        uart_putc(*c);
    }
    uart_putc(b'\n');
}

fn print_node(id: usize, msg: &str) {
    print_line(&format!("Node {}: {}", id, msg));
}

fn print_vm(id: usize, msg: &str) {
    print_line(&format!("VM {}: {}", id, msg));
}

fn clear_screen(color: Color) {
    unsafe {
        let fb = FB_BASE as *mut Color;
        for i in 0..(W * H) {
            fb.add(i).write_volatile(color);
        }
    }
}

fn draw_dashboard() {
    draw_window("ExoNode Quantum Dashboard", 50, 50, 924, 668);
    draw_text("Cluster Nodes:", 60, 70, 0xFFFF00);
    unsafe {
        for (i, maybe) in NODES.iter().enumerate() {
            if let Some(node) = maybe {
                let y = 100 + i * 20;
                let label = if node.online { "Online" } else { "Offline" };
                draw_text(&format!("Node {}: {}", node.id, label), 60, y, 0x00FF00);
            }
        }
        draw_text("VMs:", 60, 300, 0xFF00FF);
        for (i, maybe) in VMS.iter().enumerate() {
            if let Some(vm) = maybe {
                let y = 330 + i * 20;
                let label = if vm.state == VmState::Running { "Running" } else { "Stopped" };
                draw_text(&format!("VM {}: {}", vm.id, label), 60, y, 0x00FF00);
            }
        }
    }
    draw_button("Create VM", 800, 100, 150, 40, 0x4CAF50);
    draw_button("Delete VM", 800, 150, 150, 40, 0xFF5722);
    draw_button("Refresh", 800, 200, 150, 40, 0x2196F3);
}

fn draw_window(title: &str, x: usize, y: usize, width: usize, height: usize) {
    print_line(&format!("[GUI] Drawing window '{}'", title));
}

fn draw_text(text: &str, x: usize, y: usize, color: Color) {}

fn draw_button(label: &str, x: usize, y: usize, width: usize, height: usize, color: Color) {}

type Color = u32;

const W: usize = 1024;
const H: usize = 768;

const FB_BASE: usize = 0x10000000;

// ---------------------------------
// End of This Chunk
// ---------------------------------
// ---------------------------------
// NUMA-Aware Memory Management
// ---------------------------------

const MAX_MEMORY_REGIONS: usize = 64;

#[repr(C)]
pub struct MemoryRegion {
    start_addr: usize,
    size_bytes: usize,
    node_id: usize,
    is_free: bool,
}

static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] = [MemoryRegion {
    start_addr: 0,
    size_bytes: 0,
    node_id: 0,
    is_free: true,
}; MAX_MEMORY_REGIONS];

fn init_memory_regions() {
    unsafe {
        // Example: divide 256GB total memory among nodes
        let total_mem = 256 * 1024 * 1024 * 1024usize; // 256GB
        let per_node_mem = total_mem / MAX_NODES;

        for i in 0..MAX_NODES {
            MEMORY_REGIONS[i] = MemoryRegion {
                start_addr: i * per_node_mem,
                size_bytes: per_node_mem,
                node_id: i,
                is_free: true,
            };
        }
        print_line("[Memory] Initialized NUMA memory regions");
    }
}

fn allocate_memory(node_id: usize, size_bytes: usize) -> Option<usize> {
    unsafe {
        for region in MEMORY_REGIONS.iter_mut() {
            if region.is_free && region.node_id == node_id && region.size_bytes >= size_bytes {
                region.is_free = false;
                print_line(&format!("[Memory] Allocated {} bytes on node {}", size_bytes, node_id));
                return Some(region.start_addr);
            }
        }
        None
    }
}

fn free_memory(addr: usize) {
    unsafe {
        for region in MEMORY_REGIONS.iter_mut() {
            if region.start_addr == addr {
                region.is_free = true;
                print_line(&format!("[Memory] Freed memory at address {:X}", addr));
                return;
            }
        }
    }
}

// ---------------------------------
// VM Snapshot and Live Migration
// ---------------------------------

const MAX_SNAPSHOTS_PER_VM: usize = 8;

#[repr(C)]
pub struct Snapshot {
    id: usize,
    timestamp: u64,
    vm_state_data: [u8; 4096], // Simplified snapshot data block
    valid: bool,
}

static mut VM_SNAPSHOTS: [[Option<Snapshot>; MAX_SNAPSHOTS_PER_VM]; MAX_VMS] =
    [[None; MAX_SNAPSHOTS_PER_VM]; MAX_VMS];

fn create_snapshot(vm_id: usize) -> Result<(), &'static str> {
    unsafe {
        if vm_id >= MAX_VMS {
            return Err("Invalid VM ID");
        }
        for slot in VM_SNAPSHOTS[vm_id].iter_mut() {
            if slot.is_none() {
                let snap = Snapshot {
                    id: vm_id, // Use vm_id as snapshot id for simplification
                    timestamp: get_system_ticks(),
                    vm_state_data: [0; 4096],
                    valid: true,
                };
                *slot = Some(snap);
                print_vm(vm_id, "Snapshot created");
                return Ok(());
            }
        }
        Err("Max snapshots reached")
    }
}

fn delete_snapshot(vm_id: usize, snapshot_id: usize) -> Result<(), &'static str> {
    unsafe {
        if vm_id >= MAX_VMS {
            return Err("Invalid VM ID");
        }
        for slot in VM_SNAPSHOTS[vm_id].iter_mut() {
            if let Some(snap) = slot {
                if snap.id == snapshot_id {
                    *slot = None;
                    print_vm(vm_id, &format!("Snapshot {} deleted", snapshot_id));
                    return Ok(());
                }
            }
        }
        Err("Snapshot not found")
    }
}

fn migrate_vm_process(vm_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if let Some(target_node) = vm.migration_target {
                print_vm(vm_id, &format!("Migrating VM to node {}", target_node));
                // Simulate migration delay and update node
                vm.node_id = target_node;
                vm.migration_target = None;
                vm.state = VmState::Running;
                print_vm(vm_id, "Migration complete");
            }
        }
    }
}

// ---------------------------------
// Cluster Node Synchronization Protocol
// ---------------------------------

const MAX_CLUSTER_MESSAGES: usize = 128;

#[repr(C)]
pub enum ClusterMessageType {
    NodeJoin,
    NodeLeave,
    VMStateUpdate,
    Heartbeat,
}

#[repr(C)]
pub struct ClusterMessage {
    msg_type: ClusterMessageType,
    source_node: usize,
    vm_id: Option<usize>,
    data: [u8; 64],
}

static mut CLUSTER_MESSAGE_QUEUE: [Option<ClusterMessage>; MAX_CLUSTER_MESSAGES] = [None; MAX_CLUSTER_MESSAGES];

fn enqueue_cluster_message(msg: ClusterMessage) -> Result<(), &'static str> {
    unsafe {
        for slot in CLUSTER_MESSAGE_QUEUE.iter_mut() {
            if slot.is_none() {
                *slot = Some(msg);
                return Ok(());
            }
        }
        Err("Cluster message queue full")
    }
}

fn process_cluster_messages() {
    unsafe {
        for i in 0..MAX_CLUSTER_MESSAGES {
            if let Some(msg) = &CLUSTER_MESSAGE_QUEUE[i] {
                match msg.msg_type {
                    ClusterMessageType::NodeJoin => {
                        print_line(&format!("[Cluster] Node {} joined", msg.source_node));
                    }
                    ClusterMessageType::NodeLeave => {
                        print_line(&format!("[Cluster] Node {} left", msg.source_node));
                    }
                    ClusterMessageType::VMStateUpdate => {
                        if let Some(vm_id) = msg.vm_id {
                            print_line(&format!("[Cluster] VM {} state updated from node {}", vm_id, msg.source_node));
                        }
                    }
                    ClusterMessageType::Heartbeat => {
                        print_line(&format!("[Cluster] Heartbeat from node {}", msg.source_node));
                    }
                }
                CLUSTER_MESSAGE_QUEUE[i] = None;
            }
        }
    }
}

// ---------------------------------
// Fair Priority Scheduler with Aging
// ---------------------------------

const MAX_SCHEDULE_ENTRIES: usize = 256;

#[repr(C)]
pub struct ScheduleEntry {
    vm_id: usize,
    priority: u8,
    last_run_tick: u64,
    run_count: u32,
}

static mut SCHEDULE_QUEUE: [Option<ScheduleEntry>; MAX_SCHEDULE_ENTRIES] = [None; MAX_SCHEDULE_ENTRIES];

fn schedule_vm(vm_id: usize, priority: u8) {
    unsafe {
        for slot in SCHEDULE_QUEUE.iter_mut() {
            if slot.is_none() {
                *slot = Some(ScheduleEntry {
                    vm_id,
                    priority,
                    last_run_tick: 0,
                    run_count: 0,
                });
                print_vm(vm_id, "Scheduled for execution");
                return;
            }
        }
        print_vm(vm_id, "Scheduler queue full, cannot schedule");
    }
}

fn scheduler_tick() {
    unsafe {
        let current_tick = get_system_ticks();
        // Find highest priority VM with aging applied
        let mut candidate_idx: Option<usize> = None;
        let mut highest_score = 0u64;

        for (i, entry_opt) in SCHEDULE_QUEUE.iter_mut().enumerate() {
            if let Some(entry) = entry_opt {
                let age = current_tick - entry.last_run_tick;
                // Score = priority * age to favor less recently run VMs
                let score = (entry.priority as u64) * age;
                if score > highest_score {
                    highest_score = score;
                    candidate_idx = Some(i);
                }
            }
        }

        if let Some(idx) = candidate_idx {
            if let Some(entry) = &mut SCHEDULE_QUEUE[idx] {
                entry.last_run_tick = current_tick;
                entry.run_count += 1;
                run_vm(entry.vm_id);
                print_vm(entry.vm_id, &format!("Scheduled run #{}", entry.run_count));
            }
        }
    }
}

fn run_vm(vm_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if vm.active && vm.state == VmState::Running {
                // Simulate VM workload execution
                print_vm(vm_id, "Executing workload");
            }
        }
    }
}

// ---------------------------------
// Framebuffer UI Event Handling
// ---------------------------------

#[repr(C)]
pub struct MouseEvent {
    x: usize,
    y: usize,
    left_button: bool,
    right_button: bool,
}

#[repr(C)]
pub struct KeyboardEvent {
    keycode: u8,
    pressed: bool,
}

static mut LAST_MOUSE_EVENT: Option<MouseEvent> = None;
static mut LAST_KEY_EVENT: Option<KeyboardEvent> = None;

fn poll_mouse() {
    unsafe {
        // Dummy mouse input simulation
        LAST_MOUSE_EVENT = Some(MouseEvent {
            x: (get_system_ticks() as usize) % W,
            y: (get_system_ticks() as usize * 2) % H,
            left_button: (get_system_ticks() % 2) == 0,
            right_button: false,
        });
        if let Some(ev) = &LAST_MOUSE_EVENT {
            handle_mouse_event(ev);
        }
    }
}

fn poll_keyboard() {
    unsafe {
        // Dummy keyboard input simulation
        let tick = get_system_ticks();
        if tick % 10 == 0 {
            LAST_KEY_EVENT = Some(KeyboardEvent {
                keycode: 13, // Enter key
                pressed: true,
            });
        } else {
            LAST_KEY_EVENT = None;
        }
        if let Some(ev) = &LAST_KEY_EVENT {
            handle_keyboard_event(ev);
        }
    }
}

fn handle_mouse_event(ev: &MouseEvent) {
    print_line(&format!(
        "[Input] Mouse at ({}, {}) Left:{} Right:{}",
        ev.x, ev.y, ev.left_button, ev.right_button
    ));
}

fn handle_keyboard_event(ev: &KeyboardEvent) {
    print_line(&format!(
        "[Input] Keycode {} Pressed:{}",
        ev.keycode, ev.pressed
    ));
}

// ---------------------------------
// Debug and Logging Enhancements
// ---------------------------------

static mut LOG_BUFFER: [u8; 4096] = [0; 4096];
static mut LOG_INDEX: usize = 0;

fn log_message(msg: &str) {
    unsafe {
        let bytes = msg.as_bytes();
        for &b in bytes {
            if LOG_INDEX < LOG_BUFFER.len() {
                LOG_BUFFER[LOG_INDEX] = b;
                LOG_INDEX += 1;
            } else {
                // Log buffer full, reset index
                LOG_INDEX = 0;
            }
        }
        // Append newline
        if LOG_INDEX < LOG_BUFFER.len() {
            LOG_BUFFER[LOG_INDEX] = b'\n';
            LOG_INDEX += 1;
        }
    }
}

fn dump_log() {
    unsafe {
        print_line("[Log Dump Start]");
        let log_str = core::str::from_utf8_unchecked(&LOG_BUFFER[..LOG_INDEX]);
        print_line(log_str);
        print_line("[Log Dump End]");
    }
}

// ---------------------------------
// Utility Functions
// ---------------------------------

fn atoi(s: &str) -> usize {
    let mut result = 0usize;
    for b in s.bytes() {
        if b >= b'0' && b <= b'9' {
            result = result * 10 + (b - b'0') as usize;
        } else {
            break;
        }
    }
    result
}

// ---------------------------------
// Integration into Main Loop
// ---------------------------------

#[no_mangle]
pub extern "C" fn _start() -> ! {
    uart_init();
    framebuffer_init();
    pcie_init();
    discover_nodes();
    discover_storage_devices();
    discover_network_interfaces();
    init_memory_regions();
    hypervisor_init();
    startup_screen();
    start_event_loop();

    loop {
        poll_uart();
        poll_mouse();
        poll_keyboard();
        poll_cluster();
        update_telemetry();
        process_io();
        scheduler_tick();
        migrate_vms_tick();
        process_cluster_messages();
        increment_system_ticks();
    }
}

// ---------------------------------
// End of this chunk
// ---------------------------------
// ---------------------------------
// Cluster Fault Tolerance & Leader Election
// ---------------------------------

#[repr(C)]
pub enum ClusterRole {
    Follower,
    Candidate,
    Leader,
}

#[repr(C)]
pub struct ClusterState {
    current_term: u64,
    voted_for: Option<usize>,
    role: ClusterRole,
    leader_id: Option<usize>,
    election_timeout_ticks: u64,
    last_heartbeat_ticks: u64,
}

static mut CLUSTER_STATE: ClusterState = ClusterState {
    current_term: 0,
    voted_for: None,
    role: ClusterRole::Follower,
    leader_id: None,
    election_timeout_ticks: 0,
    last_heartbeat_ticks: 0,
};

fn cluster_tick() {
    unsafe {
        let ticks = get_system_ticks();

        match CLUSTER_STATE.role {
            ClusterRole::Follower => {
                if ticks - CLUSTER_STATE.last_heartbeat_ticks > CLUSTER_STATE.election_timeout_ticks {
                    CLUSTER_STATE.role = ClusterRole::Candidate;
                    CLUSTER_STATE.current_term += 1;
                    CLUSTER_STATE.voted_for = Some(get_local_node_id());
                    CLUSTER_STATE.election_timeout_ticks = ticks + (50 + (ticks % 50)); // randomized timeout
                    print_line("[Cluster] Follower timeout, starting election");
                    send_vote_requests();
                }
            }
            ClusterRole::Candidate => {
                // In a real implementation, count votes and elect leader
                if ticks > CLUSTER_STATE.election_timeout_ticks {
                    print_line("[Cluster] Election timeout, reverting to follower");
                    CLUSTER_STATE.role = ClusterRole::Follower;
                }
            }
            ClusterRole::Leader => {
                if ticks - CLUSTER_STATE.last_heartbeat_ticks > 10 {
                    send_heartbeats();
                    CLUSTER_STATE.last_heartbeat_ticks = ticks;
                }
            }
        }
    }
}

fn get_local_node_id() -> usize {
    0 // Simplified, assume node 0 is local
}

fn send_vote_requests() {
    print_line("[Cluster] Sending vote requests");
}

fn send_heartbeats() {
    print_line("[Cluster] Sending heartbeats");
}

// ---------------------------------
// VM Disk Snapshot Diffing & Incremental Backup
// ---------------------------------

const MAX_DISK_SNAPSHOTS_PER_VM: usize = 16;

#[repr(C)]
pub struct DiskSnapshot {
    id: usize,
    vm_id: usize,
    timestamp: u64,
    base_snapshot_id: Option<usize>, // For diffs
    diff_data: [u8; 8192],
    valid: bool,
}

static mut DISK_SNAPSHOTS: [[Option<DiskSnapshot>; MAX_DISK_SNAPSHOTS_PER_VM]; MAX_VMS] =
    [[None; MAX_DISK_SNAPSHOTS_PER_VM]; MAX_VMS];

fn create_disk_snapshot(vm_id: usize, base_snapshot: Option<usize>) -> Result<(), &'static str> {
    unsafe {
        for slot in DISK_SNAPSHOTS[vm_id].iter_mut() {
            if slot.is_none() {
                let snap = DiskSnapshot {
                    id: vm_id,
                    vm_id,
                    timestamp: get_system_ticks(),
                    base_snapshot_id: base_snapshot,
                    diff_data: [0; 8192],
                    valid: true,
                };
                *slot = Some(snap);
                print_vm(vm_id, "Disk snapshot created");
                return Ok(());
            }
        }
        Err("Max disk snapshots reached")
    }
}

fn delete_disk_snapshot(vm_id: usize, snapshot_id: usize) -> Result<(), &'static str> {
    unsafe {
        for slot in DISK_SNAPSHOTS[vm_id].iter_mut() {
            if let Some(snap) = slot {
                if snap.id == snapshot_id {
                    *slot = None;
                    print_vm(vm_id, &format!("Disk snapshot {} deleted", snapshot_id));
                    return Ok(());
                }
            }
        }
        Err("Disk snapshot not found")
    }
}

// ---------------------------------
// Advanced VM Isolation with Secure Enclaves Simulation
// ---------------------------------

#[repr(C)]
pub struct SecureEnclave {
    vm_id: usize,
    enclave_id: usize,
    secure_memory_start: usize,
    secure_memory_size: usize,
    active: bool,
}

static mut ENCLAVES: [Option<SecureEnclave>; MAX_VMS] = [None; MAX_VMS];

fn create_enclave(vm_id: usize, mem_start: usize, mem_size: usize) -> Result<(), &'static str> {
    unsafe {
        if ENCLAVES[vm_id].is_some() {
            return Err("Enclave already exists for VM");
        }
        ENCLAVES[vm_id] = Some(SecureEnclave {
            vm_id,
            enclave_id: vm_id, // simplification
            secure_memory_start: mem_start,
            secure_memory_size: mem_size,
            active: true,
        });
        print_vm(vm_id, "Secure enclave created");
        Ok(())
    }
}

fn destroy_enclave(vm_id: usize) -> Result<(), &'static str> {
    unsafe {
        if ENCLAVES[vm_id].is_some() {
            ENCLAVES[vm_id] = None;
            print_vm(vm_id, "Secure enclave destroyed");
            Ok(())
        } else {
            Err("No enclave to destroy")
        }
    }
}

// ---------------------------------
// Dynamic Resource Scaling & QoS Enforcement
// ---------------------------------

fn adjust_vm_resources(vm_id: usize, cpu_mask: u64, memory_mb: u32) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            vm.cpu_mask = cpu_mask;
            vm.memory_allocated_mb = memory_mb;
            print_vm(vm_id, &format!("Resources adjusted: CPU mask {:064b}, Memory {}MB", cpu_mask, memory_mb));
        }
    }
}

fn enforce_qos() {
    unsafe {
        for vm_opt in VMS.iter_mut() {
            if let Some(vm) = vm_opt {
                // Simplified QoS: reduce CPU usage if VM priority low
                if vm.priority_class < 2 {
                    // Throttle VM CPU usage artificially
                    print_vm(vm.id, "QoS throttle applied");
                }
            }
        }
    }
}

// ---------------------------------
// Power Management & Thermal Throttling Controls
// ---------------------------------

fn monitor_thermal_conditions() {
    unsafe {
        for node_opt in NODES.iter_mut() {
            if let Some(node) = node_opt {
                if node.thermal_celsius > 80 {
                    print_line(&format!("[Thermal] Node {} overheating! Throttling CPUs", node.id));
                    for cpu in node.cpus.iter_mut() {
                        cpu.throttled = true;
                        cpu.frequency_mhz = 1200; // reduce frequency
                    }
                } else if node.thermal_celsius < 60 {
                    for cpu in node.cpus.iter_mut() {
                        if cpu.throttled {
                            cpu.throttled = false;
                            cpu.frequency_mhz = 3200; // restore frequency
                        }
                    }
                }
            }
        }
    }
}

fn power_save_mode(enable: bool) {
    if enable {
        print_line("[Power] Entering power save mode");
    } else {
        print_line("[Power] Exiting power save mode");
    }
}

// ---------------------------------
// Framebuffer UI: Windows, Draggable Panels, Event Queue
// ---------------------------------

struct UIEvent {
    event_type: UIEventType,
    x: usize,
    y: usize,
    button: Option<u8>,
}

enum UIEventType {
    MouseDown,
    MouseUp,
    MouseMove,
    KeyDown,
    KeyUp,
}

static mut UI_EVENT_QUEUE: [Option<UIEvent>; 128] = [None; 128];

fn enqueue_ui_event(ev: UIEvent) {
    unsafe {
        for slot in UI_EVENT_QUEUE.iter_mut() {
            if slot.is_none() {
                *slot = Some(ev);
                return;
            }
        }
        print_line("[UI] Event queue full");
    }
}

fn process_ui_events() {
    unsafe {
        for i in 0..UI_EVENT_QUEUE.len() {
            if let Some(ev) = UI_EVENT_QUEUE[i].take() {
                handle_ui_event(&ev);
            }
        }
    }
}

fn handle_ui_event(ev: &UIEvent) {
    match ev.event_type {
        UIEventType::MouseDown => {
            print_line(&format!("[UI] Mouse down at ({}, {})", ev.x, ev.y));
        }
        UIEventType::MouseUp => {
            print_line(&format!("[UI] Mouse up at ({}, {})", ev.x, ev.y));
        }
        UIEventType::MouseMove => {
            print_line(&format!("[UI] Mouse move at ({}, {})", ev.x, ev.y));
        }
        UIEventType::KeyDown => {
            print_line(&format!("[UI] Key down at code {}", ev.button.unwrap_or(0)));
        }
        UIEventType::KeyUp => {
            print_line(&format!("[UI] Key up at code {}", ev.button.unwrap_or(0)));
        }
    }
}

// ---------------------------------
// Network Stack Simulation: Virtual NICs & Packet Filtering
// ---------------------------------

#[repr(C)]
pub struct VirtualNIC {
    id: usize,
    vm_id: usize,
    mac_address: [u8; 6],
    ip_address: u32,
    enabled: bool,
    packets_sent: u64,
    packets_received: u64,
}

static mut VIRTUAL_NICS: [Option<VirtualNIC>; MAX_VMS] = [None; MAX_VMS];

fn create_virtual_nic(vm_id: usize, mac: [u8; 6], ip: u32) -> Result<(), &'static str> {
    unsafe {
        if VIRTUAL_NICS[vm_id].is_some() {
            return Err("NIC already exists for VM");
        }
        VIRTUAL_NICS[vm_id] = Some(VirtualNIC {
            id: vm_id,
            vm_id,
            mac_address: mac,
            ip_address: ip,
            enabled: true,
            packets_sent: 0,
            packets_received: 0,
        });
        print_vm(vm_id, "Virtual NIC created");
        Ok(())
    }
}

fn send_packet(vm_id: usize, data: &[u8]) -> Result<(), &'static str> {
    unsafe {
        if let Some(nic) = &mut VIRTUAL_NICS[vm_id] {
            if nic.enabled {
                nic.packets_sent += 1;
                print_vm(vm_id, &format!("Packet sent, length {}", data.len()));
                return Ok(());
            }
        }
        Err("NIC not available or disabled")
    }
}

fn receive_packet(vm_id: usize, data: &[u8]) -> Result<(), &'static str> {
    unsafe {
        if let Some(nic) = &mut VIRTUAL_NICS[vm_id] {
            if nic.enabled {
                nic.packets_received += 1;
                print_vm(vm_id, &format!("Packet received, length {}", data.len()));
                return Ok(());
            }
        }
        Err("NIC not available or disabled")
    }
}

// ---------------------------------
// Main Loop Integration (Partial)
// ---------------------------------

#[no_mangle]
pub extern "C" fn _start() -> ! {
    uart_init();
    framebuffer_init();
    pcie_init();
    discover_nodes();
    init_memory_regions();
    hypervisor_init();
    startup_screen();
    start_event_loop();

    loop {
        poll_uart();
        poll_mouse();
        poll_keyboard();
        poll_cluster();
        update_telemetry();
        process_ui_events();
        scheduler_tick();
        cluster_tick();
        enforce_qos();
        monitor_thermal_conditions();
        process_cluster_messages();
        increment_system_ticks();
    }
}

// ---------------------------------
// End of this chunk
// ---------------------------------
// === Container Runtime Support ===

const MAX_CONTAINERS: usize = 512;
const MAX_CONTAINER_LAYERS: usize = 32;
const MAX_IMAGES: usize = 128;

#[derive(Clone, Copy)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}

#[repr(C)]
pub struct Container {
    id: usize,
    vm_host_id: Option<usize>, // VM ID or None for bare-metal
    label: &'static str,
    cpu_shares: u32,
    memory_limit_mb: u32,
    state: ContainerState,
    network_namespace: usize,
    storage_layers: [Option<usize>; MAX_CONTAINER_LAYERS], // Image layers by ID
    storage_layer_count: usize,
    creation_ticks: u64,
    restart_policy: RestartPolicy,
    health_status: HealthStatus,
    ports: [u16; 16], // Exposed ports
    environment_vars: [(&'static str, &'static str); 16],
}

#[derive(Clone, Copy)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

#[derive(Clone, Copy)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

static mut CONTAINERS: [Option<Container>; MAX_CONTAINERS] = [None; MAX_CONTAINERS];

// === Container Image Management ===

#[repr(C)]
pub struct ImageLayer {
    id: usize,
    size_bytes: usize,
    compressed: bool,
    checksum: u64,
    // For simplicity, layer data storage abstracted
}

#[repr(C)]
pub struct ContainerImage {
    id: usize,
    name: &'static str,
    tag: &'static str,
    layers: [Option<ImageLayer>; MAX_CONTAINER_LAYERS],
    layer_count: usize,
    created_ticks: u64,
}

static mut IMAGES: [Option<ContainerImage>; MAX_IMAGES] = [None; MAX_IMAGES];

fn pull_image(name: &'static str, tag: &'static str) -> Result<usize, &'static str> {
    unsafe {
        for img in IMAGES.iter_mut() {
            if img.is_none() {
                // Simulate image pull by creating dummy layers
                let mut layers = [None; MAX_CONTAINER_LAYERS];
                layers[0] = Some(ImageLayer {
                    id: 0,
                    size_bytes: 1024 * 1024 * 100,
                    compressed: true,
                    checksum: 0xdeadbeef,
                });
                let image = ContainerImage {
                    id: 0,
                    name,
                    tag,
                    layers,
                    layer_count: 1,
                    created_ticks: get_system_ticks(),
                };
                *img = Some(image);
                print_line(&format!("[Container] Pulled image {}:{}", name, tag));
                return Ok(0);
            }
        }
        Err("Image storage full")
    }
}

fn create_container_from_image(vm_host_id: Option<usize>, image_id: usize, label: &'static str) -> Result<usize, &'static str> {
    unsafe {
        if image_id >= MAX_IMAGES || IMAGES[image_id].is_none() {
            return Err("Image not found");
        }
        for slot in CONTAINERS.iter_mut() {
            if slot.is_none() {
                let image = IMAGES[image_id].unwrap();
                let mut layers = [None; MAX_CONTAINER_LAYERS];
                for i in 0..image.layer_count {
                    layers[i] = image.layers[i];
                }
                let container = Container {
                    id: 0,
                    vm_host_id,
                    label,
                    cpu_shares: 1024,
                    memory_limit_mb: 256,
                    state: ContainerState::Created,
                    network_namespace: generate_network_namespace(),
                    storage_layers: layers,
                    storage_layer_count: image.layer_count,
                    creation_ticks: get_system_ticks(),
                    restart_policy: RestartPolicy::OnFailure,
                    health_status: HealthStatus::Unknown,
                    ports: [0;16],
                    environment_vars: [("PATH", "/usr/bin") ; 16],
                };
                *slot = Some(container);
                print_line(&format!("[Container] Created container {} from image {}", label, image.name));
                return Ok(0);
            }
        }
        Err("Container limit reached")
    }
}

// === Container Networking & Namespaces ===

fn generate_network_namespace() -> usize {
    // Simplified: use a random or incrementing namespace id
    static mut LAST_NET_NS: usize = 0;
    unsafe {
        LAST_NET_NS += 1;
        LAST_NET_NS
    }
}

fn assign_container_ip(ns: usize) -> u32 {
    // Dummy IP allocation: 10.0.NS.X
    0x0A000000 + ((ns as u32) << 8) + 1
}

// === Orchestration Primitives ===

fn deploy_service(image_name: &'static str, tag: &'static str, replicas: usize, label_prefix: &'static str) {
    let image_id = match pull_image(image_name, tag) {
        Ok(id) => id,
        Err(e) => {
            print_line(&format!("[Orchestration] Failed to pull image: {}", e));
            return;
        }
    };
    for i in 0..replicas {
        let label = format!("{}-{}", label_prefix, i);
        match create_container_from_image(None, image_id, Box::leak(label.into_boxed_str())) {
            Ok(cid) => print_line(&format!("[Orchestration] Deployed container ID {}", cid)),
            Err(e) => print_line(&format!("[Orchestration] Failed to create container: {}", e)),
        }
    }
}

fn scale_service(label_prefix: &'static str, desired_replicas: usize) {
    let mut current = 0;
    unsafe {
        for c in CONTAINERS.iter() {
            if let Some(container) = c {
                if container.label.starts_with(label_prefix) && container.state == ContainerState::Running {
                    current += 1;
                }
            }
        }
        if desired_replicas > current {
            let to_create = desired_replicas - current;
            print_line(&format!("[Orchestration] Scaling up {} replicas", to_create));
            // Creation code omitted for brevity
        } else if current > desired_replicas {
            let to_stop = current - desired_replicas;
            print_line(&format!("[Orchestration] Scaling down {} replicas", to_stop));
            // Stop code omitted for brevity
        } else {
            print_line("[Orchestration] Desired replicas equals current replicas; no action");
        }
    }
}

// === Persistent & Ephemeral Storage Backends ===

#[repr(C)]
pub struct StorageVolume {
    id: usize,
    size_mb: usize,
    mounted_path: &'static str,
    persistent: bool,
}

const MAX_VOLUMES: usize = 256;
static mut VOLUMES: [Option<StorageVolume>; MAX_VOLUMES] = [None; MAX_VOLUMES];

fn create_volume(size_mb: usize, path: &'static str, persistent: bool) -> Result<usize, &'static str> {
    unsafe {
        for slot in VOLUMES.iter_mut() {
            if slot.is_none() {
                let volume = StorageVolume {
                    id: 0,
                    size_mb,
                    mounted_path: path,
                    persistent,
                };
                *slot = Some(volume);
                print_line(&format!("[Storage] Created volume {} MB at {}", size_mb, path));
                return Ok(0);
            }
        }
        Err("No storage volume slots available")
    }
}

// === Distributed Filesystem Integration (Stub) ===

fn mount_distributed_fs(mount_point: &'static str) -> bool {
    print_line(&format!("[DistributedFS] Mounted at {}", mount_point));
    true
}

fn unmount_distributed_fs(mount_point: &'static str) -> bool {
    print_line(&format!("[DistributedFS] Unmounted from {}", mount_point));
    true
}

// === Basic Service Mesh & RPC Simulation ===

#[repr(C)]
pub struct RPCRequest {
    from_vm_id: usize,
    to_vm_id: usize,
    payload: [u8; 256],
    length: usize,
}

fn send_rpc(request: &RPCRequest) -> Result<(), &'static str> {
    print_line(&format!("[RPC] VM {} -> VM {}: {} bytes", request.from_vm_id, request.to_vm_id, request.length));
    Ok(())
}

fn receive_rpc(request: &RPCRequest) {
    print_line(&format!("[RPC] Received RPC at VM {}", request.to_vm_id));
}

// === Metrics Collection & Export ===

#[repr(C)]
pub struct Metric {
    name: &'static str,
    value: u64,
}

static mut METRICS: [Option<Metric>; 128] = [None; 128];

fn update_metric(name: &'static str, val: u64) {
    unsafe {
        for slot in METRICS.iter_mut() {
            if let Some(m) = slot {
                if m.name == name {
                    *slot = Some(Metric { name, value: val });
                    return;
                }
            }
        }
        for slot in METRICS.iter_mut() {
            if slot.is_none() {
                *slot = Some(Metric { name, value: val });
                return;
            }
        }
    }
}

fn export_metrics() {
    unsafe {
        for m in METRICS.iter() {
            if let Some(metric) = m {
                print_line(&format!("[Metric] {} = {}", metric.name, metric.value));
            }
        }
    }
}

// === Helpers & Utilities ===

fn get_system_ticks() -> u64 {
    static mut TICKS: u64 = 0;
    unsafe {
        TICKS += 1;
        TICKS
    }
}

fn print_line(_msg: &str) {
    // Simulated UART print, omitted for brevity
}

// === Container Runtime Support ===

const MAX_CONTAINERS: usize = 512;
const MAX_CONTAINER_LAYERS: usize = 32;
const MAX_IMAGES: usize = 128;

#[derive(Clone, Copy)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}

#[repr(C)]
pub struct Container {
    id: usize,
    vm_host_id: Option<usize>, // VM ID or None for bare-metal
    label: &'static str,
    cpu_shares: u32,
    memory_limit_mb: u32,
    state: ContainerState,
    network_namespace: usize,
    storage_layers: [Option<usize>; MAX_CONTAINER_LAYERS], // Image layers by ID
    storage_layer_count: usize,
    creation_ticks: u64,
    restart_policy: RestartPolicy,
    health_status: HealthStatus,
    ports: [u16; 16], // Exposed ports
    environment_vars: [(&'static str, &'static str); 16],
}

#[derive(Clone, Copy)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

#[derive(Clone, Copy)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

static mut CONTAINERS: [Option<Container>; MAX_CONTAINERS] = [None; MAX_CONTAINERS];

// === Container Image Management ===

#[repr(C)]
pub struct ImageLayer {
    id: usize,
    size_bytes: usize,
    compressed: bool,
    checksum: u64,
    // For simplicity, layer data storage abstracted
}

#[repr(C)]
pub struct ContainerImage {
    id: usize,
    name: &'static str,
    tag: &'static str,
    layers: [Option<ImageLayer>; MAX_CONTAINER_LAYERS],
    layer_count: usize,
    created_ticks: u64,
}

static mut IMAGES: [Option<ContainerImage>; MAX_IMAGES] = [None; MAX_IMAGES];

fn pull_image(name: &'static str, tag: &'static str) -> Result<usize, &'static str> {
    unsafe {
        for img in IMAGES.iter_mut() {
            if img.is_none() {
                // Simulate image pull by creating dummy layers
                let mut layers = [None; MAX_CONTAINER_LAYERS];
                layers[0] = Some(ImageLayer {
                    id: 0,
                    size_bytes: 1024 * 1024 * 100,
                    compressed: true,
                    checksum: 0xdeadbeef,
                });
                let image = ContainerImage {
                    id: 0,
                    name,
                    tag,
                    layers,
                    layer_count: 1,
                    created_ticks: get_system_ticks(),
                };
                *img = Some(image);
                print_line(&format!("[Container] Pulled image {}:{}", name, tag));
                return Ok(0);
            }
        }
        Err("Image storage full")
    }
}

fn create_container_from_image(vm_host_id: Option<usize>, image_id: usize, label: &'static str) -> Result<usize, &'static str> {
    unsafe {
        if image_id >= MAX_IMAGES || IMAGES[image_id].is_none() {
            return Err("Image not found");
        }
        for slot in CONTAINERS.iter_mut() {
            if slot.is_none() {
                let image = IMAGES[image_id].unwrap();
                let mut layers = [None; MAX_CONTAINER_LAYERS];
                for i in 0..image.layer_count {
                    layers[i] = image.layers[i];
                }
                let container = Container {
                    id: 0,
                    vm_host_id,
                    label,
                    cpu_shares: 1024,
                    memory_limit_mb: 256,
                    state: ContainerState::Created,
                    network_namespace: generate_network_namespace(),
                    storage_layers: layers,
                    storage_layer_count: image.layer_count,
                    creation_ticks: get_system_ticks(),
                    restart_policy: RestartPolicy::OnFailure,
                    health_status: HealthStatus::Unknown,
                    ports: [0;16],
                    environment_vars: [("PATH", "/usr/bin") ; 16],
                };
                *slot = Some(container);
                print_line(&format!("[Container] Created container {} from image {}", label, image.name));
                return Ok(0);
            }
        }
        Err("Container limit reached")
    }
}

// === Container Networking & Namespaces ===

fn generate_network_namespace() -> usize {
    // Simplified: use a random or incrementing namespace id
    static mut LAST_NET_NS: usize = 0;
    unsafe {
        LAST_NET_NS += 1;
        LAST_NET_NS
    }
}

fn assign_container_ip(ns: usize) -> u32 {
    // Dummy IP allocation: 10.0.NS.X
    0x0A000000 + ((ns as u32) << 8) + 1
}

// === Orchestration Primitives ===

fn deploy_service(image_name: &'static str, tag: &'static str, replicas: usize, label_prefix: &'static str) {
    let image_id = match pull_image(image_name, tag) {
        Ok(id) => id,
        Err(e) => {
            print_line(&format!("[Orchestration] Failed to pull image: {}", e));
            return;
        }
    };
    for i in 0..replicas {
        let label = format!("{}-{}", label_prefix, i);
        match create_container_from_image(None, image_id, Box::leak(label.into_boxed_str())) {
            Ok(cid) => print_line(&format!("[Orchestration] Deployed container ID {}", cid)),
            Err(e) => print_line(&format!("[Orchestration] Failed to create container: {}", e)),
        }
    }
}

fn scale_service(label_prefix: &'static str, desired_replicas: usize) {
    let mut current = 0;
    unsafe {
        for c in CONTAINERS.iter() {
            if let Some(container) = c {
                if container.label.starts_with(label_prefix) && container.state == ContainerState::Running {
                    current += 1;
                }
            }
        }
        if desired_replicas > current {
            let to_create = desired_replicas - current;
            print_line(&format!("[Orchestration] Scaling up {} replicas", to_create));
            // Creation code omitted for brevity
        } else if current > desired_replicas {
            let to_stop = current - desired_replicas;
            print_line(&format!("[Orchestration] Scaling down {} replicas", to_stop));
            // Stop code omitted for brevity
        } else {
            print_line("[Orchestration] Desired replicas equals current replicas; no action");
        }
    }
}

// === Persistent & Ephemeral Storage Backends ===

#[repr(C)]
pub struct StorageVolume {
    id: usize,
    size_mb: usize,
    mounted_path: &'static str,
    persistent: bool,
}

const MAX_VOLUMES: usize = 256;
static mut VOLUMES: [Option<StorageVolume>; MAX_VOLUMES] = [None; MAX_VOLUMES];

fn create_volume(size_mb: usize, path: &'static str, persistent: bool) -> Result<usize, &'static str> {
    unsafe {
        for slot in VOLUMES.iter_mut() {
            if slot.is_none() {
                let volume = StorageVolume {
                    id: 0,
                    size_mb,
                    mounted_path: path,
                    persistent,
                };
                *slot = Some(volume);
                print_line(&format!("[Storage] Created volume {} MB at {}", size_mb, path));
                return Ok(0);
            }
        }
        Err("No storage volume slots available")
    }
}

// === Distributed Filesystem Integration (Stub) ===

fn mount_distributed_fs(mount_point: &'static str) -> bool {
    print_line(&format!("[DistributedFS] Mounted at {}", mount_point));
    true
}

fn unmount_distributed_fs(mount_point: &'static str) -> bool {
    print_line(&format!("[DistributedFS] Unmounted from {}", mount_point));
    true
}

// === Basic Service Mesh & RPC Simulation ===

#[repr(C)]
pub struct RPCRequest {
    from_vm_id: usize,
    to_vm_id: usize,
    payload: [u8; 256],
    length: usize,
}

fn send_rpc(request: &RPCRequest) -> Result<(), &'static str> {
    print_line(&format!("[RPC] VM {} -> VM {}: {} bytes", request.from_vm_id, request.to_vm_id, request.length));
    Ok(())
}

fn receive_rpc(request: &RPCRequest) {
    print_line(&format!("[RPC] Received RPC at VM {}", request.to_vm_id));
}

// === Metrics Collection & Export ===

#[repr(C)]
pub struct Metric {
    name: &'static str,
    value: u64,
}

static mut METRICS: [Option<Metric>; 128] = [None; 128];

fn update_metric(name: &'static str, val: u64) {
    unsafe {
        for slot in METRICS.iter_mut() {
            if let Some(m) = slot {
                if m.name == name {
                    *slot = Some(Metric { name, value: val });
                    return;
                }
            }
        }
        for slot in METRICS.iter_mut() {
            if slot.is_none() {
                *slot = Some(Metric { name, value: val });
                return;
            }
        }
    }
}

fn export_metrics() {
    unsafe {
        for m in METRICS.iter() {
            if let Some(metric) = m {
                print_line(&format!("[Metric] {} = {}", metric.name, metric.value));
            }
        }
    }
}

// === Helpers & Utilities ===

fn get_system_ticks() -> u64 {
    static mut TICKS: u64 = 0;
    unsafe {
        TICKS += 1;
        TICKS
    }
}

fn print_line(_msg: &str) {
    // Simulated UART print, omitted for brevity
}
// === Container Orchestration Internals ===

const MAX_SCHEDULED_TASKS: usize = 256;

#[repr(C)]
pub struct ScheduledTask {
    container_id: usize,
    next_run_tick: u64,
    interval_ticks: u64,
    retries: u8,
    max_retries: u8,
    active: bool,
}

static mut SCHEDULED_TASKS: [Option<ScheduledTask>; MAX_SCHEDULED_TASKS] = [None; MAX_SCHEDULED_TASKS];

fn schedule_health_check(container_id: usize, interval_ticks: u64, max_retries: u8) {
    unsafe {
        for slot in SCHEDULED_TASKS.iter_mut() {
            if slot.is_none() {
                *slot = Some(ScheduledTask {
                    container_id,
                    next_run_tick: get_system_ticks() + interval_ticks,
                    interval_ticks,
                    retries: 0,
                    max_retries,
                    active: true,
                });
                print_line(&format!("[Scheduler] Health check scheduled for container {}", container_id));
                return;
            }
        }
        print_line("[Scheduler] No free slots for scheduling");
    }
}

fn run_scheduled_tasks() {
    let now = get_system_ticks();
    unsafe {
        for task_opt in SCHEDULED_TASKS.iter_mut() {
            if let Some(task) = task_opt {
                if task.active && now >= task.next_run_tick {
                    let mut should_retry = false;
                    match perform_health_check(task.container_id) {
                        Ok(healthy) if healthy => {
                            task.retries = 0; // Reset retries on success
                            print_line(&format!("[Scheduler] Container {} healthy", task.container_id));
                        }
                        Ok(false) => {
                            print_line(&format!("[Scheduler] Container {} unhealthy", task.container_id));
                            should_retry = true;
                        }
                        Err(e) => {
                            print_line(&format!("[Scheduler] Health check error for container {}: {}", task.container_id, e));
                            should_retry = true;
                        }
                    }
                    if should_retry {
                        task.retries += 1;
                        if task.retries > task.max_retries {
                            print_line(&format!("[Scheduler] Max retries exceeded for container {}, restarting", task.container_id));
                            restart_container(task.container_id);
                            task.retries = 0;
                        }
                    }
                    task.next_run_tick = now + task.interval_ticks;
                }
            }
        }
    }
}

fn perform_health_check(container_id: usize) -> Result<bool, &'static str> {
    unsafe {
        if container_id >= MAX_CONTAINERS {
            return Err("Invalid container ID");
        }
        if let Some(container) = &CONTAINERS[container_id] {
            match container.state {
                ContainerState::Running => Ok(true), // Simplified, always healthy
                _ => Ok(false),
            }
        } else {
            Err("Container not found")
        }
    }
}

fn restart_container(container_id: usize) {
    unsafe {
        if let Some(container) = &mut CONTAINERS[container_id] {
            print_line(&format!("[Container] Restarting container {}", container_id));
            container.state = ContainerState::Stopped;
            // Wait simulated, then restart
            container.state = ContainerState::Running;
            container.health_status = HealthStatus::Healthy;
        }
    }
}

// === Security Isolation Concepts (Namespace + Resource Control) ===

#[repr(C)]
pub struct Namespace {
    id: usize,
    processes: [Option<usize>; 64], // container or vm IDs
    net_namespace_id: usize,
    ipc_namespace_id: usize,
    mount_namespace_id: usize,
    pid_namespace_id: usize,
}

static mut NAMESPACES: [Option<Namespace>; 64] = [None; 64];

fn create_namespace() -> usize {
    unsafe {
        for (i, slot) in NAMESPACES.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(Namespace {
                    id: i,
                    processes: [None; 64],
                    net_namespace_id: i,
                    ipc_namespace_id: i,
                    mount_namespace_id: i,
                    pid_namespace_id: i,
                });
                print_line(&format!("[Namespace] Created namespace {}", i));
                return i;
            }
        }
        print_line("[Namespace] Max namespaces reached");
        0 // Fallback to default namespace
    }
}

fn assign_process_to_namespace(process_id: usize, ns_id: usize) -> bool {
    unsafe {
        if let Some(ns) = &mut NAMESPACES[ns_id] {
            for slot in ns.processes.iter_mut() {
                if slot.is_none() {
                    *slot = Some(process_id);
                    print_line(&format!("[Namespace] Assigned process {} to namespace {}", process_id, ns_id));
                    return true;
                }
            }
        }
        false
    }
}

// === Distributed Consensus (Raft-like Stub) ===

#[repr(C)]
pub struct ConsensusNode {
    id: usize,
    term: u64,
    voted_for: Option<usize>,
    log_index: u64,
    commit_index: u64,
    state: ConsensusState,
    peers: [usize; 8],
    peer_count: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConsensusState {
    Follower,
    Candidate,
    Leader,
}

static mut CONSENSUS_NODES: [Option<ConsensusNode>; 16] = [None; 16];

fn init_consensus_node(id: usize, peers: &[usize]) {
    unsafe {
        CONSENSUS_NODES[id] = Some(ConsensusNode {
            id,
            term: 0,
            voted_for: None,
            log_index: 0,
            commit_index: 0,
            state: ConsensusState::Follower,
            peers: [0; 8],
            peer_count: peers.len().min(8),
        });
        if let Some(node) = &mut CONSENSUS_NODES[id] {
            for (i, &peer) in peers.iter().enumerate().take(node.peer_count) {
                node.peers[i] = peer;
            }
        }
        print_line(&format!("[Consensus] Node {} initialized with {} peers", id, peers.len()));
    }
}

fn run_consensus_election(id: usize) {
    unsafe {
        if let Some(node) = &mut CONSENSUS_NODES[id] {
            node.term += 1;
            node.voted_for = Some(id);
            node.state = ConsensusState::Candidate;
            print_line(&format!("[Consensus] Node {} starting election term {}", id, node.term));
            // Voting and log replication logic omitted for brevity
        }
    }
}

// === Filesystem Drivers (Simplified) ===

fn mount_ext4(mount_point: &'static str, device_path: &'static str) -> bool {
    print_line(&format!("[FS] Mounted ext4 {} on {}", device_path, mount_point));
    true
}

fn unmount_fs(mount_point: &'static str) -> bool {
    print_line(&format!("[FS] Unmounted {}", mount_point));
    true
}

// === CLI Extensions ===

fn handle_command(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "status" => {
            print_line("[CLI] Cluster status:");
            list_nodes();
            list_vms();
            list_containers();
        }
        "create-vm" => {
            if parts.len() >= 3 {
                let id = parts[1].parse::<usize>().unwrap_or(0);
                let node_id = parts[2].parse::<usize>().unwrap_or(0);
                create_vm(id, node_id, 0xFFFF_FFFF);
            } else {
                print_line("Usage: create-vm <id> <node_id>");
            }
        }
        "create-container" => {
            if parts.len() >= 4 {
                let label = parts[1];
                let image = parts[2];
                let tag = parts[3];
                let _ = pull_image(image, tag);
                let _ = create_container_from_image(None, 0, label);
            } else {
                print_line("Usage: create-container <label> <image> <tag>");
            }
        }
        "deploy-service" => {
            if parts.len() >= 4 {
                let image = parts[1];
                let tag = parts[2];
                let replicas = parts[3].parse::<usize>().unwrap_or(1);
                let label = if parts.len() > 4 { parts[4] } else { "svc" };
                deploy_service(image, tag, replicas, label);
            } else {
                print_line("Usage: deploy-service <image> <tag> <replicas> [label_prefix]");
            }
        }
        "scale-service" => {
            if parts.len() >= 3 {
                let label = parts[1];
                let replicas = parts[2].parse::<usize>().unwrap_or(1);
                scale_service(label, replicas);
            } else {
                print_line("Usage: scale-service <label_prefix> <replicas>");
            }
        }
        "help" => {
            print_line("Commands:");
            print_line("  status");
            print_line("  create-vm <id> <node_id>");
            print_line("  create-container <label> <image> <tag>");
            print_line("  deploy-service <image> <tag> <replicas> [label_prefix]");
            print_line("  scale-service <label_prefix> <replicas>");
        }
        _ => {
            print_line("Unknown command");
        }
    }
}

fn list_containers() {
    unsafe {
        for c in CONTAINERS.iter() {
            if let Some(container) = c {
                print_line(&format!(
                    "Container {}: {} - State: {:?} - VM Host: {:?} - CPU Shares: {} - Mem Limit: {}MB - Health: {:?}",
                    container.id,
                    container.label,
                    container.state,
                    container.vm_host_id,
                    container.cpu_shares,
                    container.memory_limit_mb,
                    container.health_status
                ));
            }
        }
    }
}

// === Extended Metrics, Logging & Alerts ===

fn log_event(level: LogLevel, message: &str) {
    // Simulated log output with severity
    print_line(&format!("[{:?}] {}", level, message));
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

fn alert_on_metric(name: &'static str, threshold: u64) {
    unsafe {
        for m in METRICS.iter() {
            if let Some(metric) = m {
                if metric.name == name && metric.value > threshold {
                    log_event(LogLevel::Warn, &format!("Metric {} exceeded threshold {}", name, threshold));
                }
            }
        }
    }
}

// === Main loop enhancements ===

#[no_mangle]
pub extern "C" fn _start() -> ! {
    uart_init();
    framebuffer_init();
    pcie_init();
    discover_nodes();
    hypervisor_init();
    startup_screen();
    start_event_loop();

    // Initialize consensus nodes for cluster coordination
    init_consensus_node(0, &[1,2,3]);
    init_consensus_node(1, &[0,2,3]);
    init_consensus_node(2, &[0,1,3]);
    init_consensus_node(3, &[0,1,2]);

    loop {
        poll_uart();
        poll_mouse();
        poll_cluster();
        run_scheduled_tasks();
        update_telemetry();
        export_metrics();
        // Add more periodic cluster and orchestration maintenance tasks here
    }
}
// === Container Networking ===

#[repr(C)]
pub struct NetworkInterface {
    id: usize,
    mac_address: [u8; 6],
    ip_address_v4: Option<u32>,
    ip_address_v6: Option<[u8; 16]>,
    net_namespace: usize,
    link_speed_mbps: u32,
    state: NetIfState,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NetIfState {
    Down,
    Up,
    Testing,
}

static mut NETWORK_INTERFACES: [Option<NetworkInterface>; 512] = [None; 512];

fn create_network_interface(ns_id: usize, ip_v4: Option<u32>, ip_v6: Option<[u8; 16]>) -> usize {
    unsafe {
        for (i, slot) in NETWORK_INTERFACES.iter_mut().enumerate() {
            if slot.is_none() {
                let mac = generate_mac_address(i as u8);
                *slot = Some(NetworkInterface {
                    id: i,
                    mac_address: mac,
                    ip_address_v4: ip_v4,
                    ip_address_v6: ip_v6,
                    net_namespace: ns_id,
                    link_speed_mbps: 1000,
                    state: NetIfState::Up,
                });
                print_line(&format!("[Net] Created interface {} in namespace {}", i, ns_id));
                return i;
            }
        }
        print_line("[Net] No free network interface slots");
        0
    }
}

fn generate_mac_address(seed: u8) -> [u8; 6] {
    // Locally administered MAC with fixed OUI
    [0x02, 0x00, 0x00, 0x00, 0x00, seed]
}

fn bring_interface_up(if_id: usize) {
    unsafe {
        if let Some(iface) = &mut NETWORK_INTERFACES[if_id] {
            iface.state = NetIfState::Up;
            print_line(&format!("[Net] Interface {} brought up", if_id));
        }
    }
}

fn bring_interface_down(if_id: usize) {
    unsafe {
        if let Some(iface) = &mut NETWORK_INTERFACES[if_id] {
            iface.state = NetIfState::Down;
            print_line(&format!("[Net] Interface {} brought down", if_id));
        }
    }
}

// === Storage Layer Enhancements ===

const MAX_BLOCK_DEVICES: usize = 64;

#[repr(C)]
pub struct BlockDevice {
    id: usize,
    device_path: &'static str,
    size_gb: u32,
    read_iops: u32,
    write_iops: u32,
    mounted: bool,
    mount_point: Option<&'static str>,
    fs_type: FsType,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FsType {
    Ext4,
    Xfs,
    Btrfs,
    DistributedFs,
    Unknown,
}

static mut BLOCK_DEVICES: [Option<BlockDevice>; MAX_BLOCK_DEVICES] = [None; MAX_BLOCK_DEVICES];

fn register_block_device(device_path: &'static str, size_gb: u32, fs_type: FsType) -> usize {
    unsafe {
        for (i, slot) in BLOCK_DEVICES.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(BlockDevice {
                    id: i,
                    device_path,
                    size_gb,
                    read_iops: 0,
                    write_iops: 0,
                    mounted: false,
                    mount_point: None,
                    fs_type,
                });
                print_line(&format!("[Storage] Registered device {} size {}GB", device_path, size_gb));
                return i;
            }
        }
        print_line("[Storage] No free block device slots");
        0
    }
}

fn mount_block_device(device_id: usize, mount_point: &'static str) -> bool {
    unsafe {
        if let Some(dev) = &mut BLOCK_DEVICES[device_id] {
            if dev.mounted {
                print_line(&format!("[Storage] Device {} already mounted", dev.device_path));
                return false;
            }
            dev.mounted = true;
            dev.mount_point = Some(mount_point);
            print_line(&format!("[Storage] Mounted device {} at {}", dev.device_path, mount_point));
            true
        } else {
            print_line("[Storage] Invalid device ID");
            false
        }
    }
}

fn unmount_block_device(device_id: usize) -> bool {
    unsafe {
        if let Some(dev) = &mut BLOCK_DEVICES[device_id] {
            if !dev.mounted {
                print_line(&format!("[Storage] Device {} not mounted", dev.device_path));
                return false;
            }
            print_line(&format!("[Storage] Unmounted device {} from {}", dev.device_path, dev.mount_point.unwrap_or("unknown")));
            dev.mounted = false;
            dev.mount_point = None;
            true
        } else {
            print_line("[Storage] Invalid device ID");
            false
        }
    }
}

// === Extended Container Runtime Features ===

#[derive(Clone, Copy, PartialEq)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
    Restarting,
    Exited,
}

#[derive(Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Unhealthy,
}

#[repr(C)]
pub struct Container {
    id: usize,
    label: &'static str,
    image_name: &'static str,
    image_tag: &'static str,
    state: ContainerState,
    vm_host_id: Option<usize>,
    cpu_shares: u16,
    memory_limit_mb: u32,
    health_status: HealthStatus,
    restart_policy: RestartPolicy,
    network_interface_id: Option<usize>,
    namespace_id: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RestartPolicy {
    No,
    OnFailure(u8), // max retries
    Always,
}

const MAX_CONTAINERS: usize = 1024;
static mut CONTAINERS: [Option<Container>; MAX_CONTAINERS] = [None; MAX_CONTAINERS];

fn create_container_from_image(vm_host_id: Option<usize>, namespace_id: usize, label: &'static str) -> usize {
    unsafe {
        for (i, slot) in CONTAINERS.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(Container {
                    id: i,
                    label,
                    image_name: "example-image",
                    image_tag: "latest",
                    state: ContainerState::Created,
                    vm_host_id,
                    cpu_shares: 1024,
                    memory_limit_mb: 512,
                    health_status: HealthStatus::Unknown,
                    restart_policy: RestartPolicy::OnFailure(5),
                    network_interface_id: None,
                    namespace_id,
                });
                print_line(&format!("[Container] Created container {} '{}'", i, label));
                return i;
            }
        }
        print_line("[Container] No free container slots");
        0
    }
}

fn start_container(container_id: usize) {
    unsafe {
        if let Some(container) = &mut CONTAINERS[container_id] {
            if container.state == ContainerState::Running {
                print_line(&format!("[Container] Container {} already running", container_id));
                return;
            }
            container.state = ContainerState::Running;
            container.health_status = HealthStatus::Healthy;
            print_line(&format!("[Container] Started container {}", container_id));
        } else {
            print_line("[Container] Invalid container ID");
        }
    }
}

fn stop_container(container_id: usize) {
    unsafe {
        if let Some(container) = &mut CONTAINERS[container_id] {
            if container.state == ContainerState::Stopped {
                print_line(&format!("[Container] Container {} already stopped", container_id));
                return;
            }
            container.state = ContainerState::Stopped;
            container.health_status = HealthStatus::Unknown;
            print_line(&format!("[Container] Stopped container {}", container_id));
        } else {
            print_line("[Container] Invalid container ID");
        }
    }
}

// === CLI Command Extensions for Storage and Networking ===

fn handle_command(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "list-devices" => {
            unsafe {
                for dev_opt in BLOCK_DEVICES.iter() {
                    if let Some(dev) = dev_opt {
                        print_line(&format!("Device {}: Path={} Size={}GB Mounted={} MountPoint={:?} FS={:?}",
                            dev.id, dev.device_path, dev.size_gb, dev.mounted, dev.mount_point, dev.fs_type));
                    }
                }
            }
        }
        "mount-device" => {
            if parts.len() == 3 {
                let id = parts[1].parse::<usize>().unwrap_or(usize::MAX);
                let mount_point = parts[2];
                if id != usize::MAX && mount_block_device(id, mount_point) {
                    print_line(&format!("Mounted device {} at {}", id, mount_point));
                } else {
                    print_line("Failed to mount device");
                }
            } else {
                print_line("Usage: mount-device <device_id> <mount_point>");
            }
        }
        "unmount-device" => {
            if parts.len() == 2 {
                let id = parts[1].parse::<usize>().unwrap_or(usize::MAX);
                if id != usize::MAX && unmount_block_device(id) {
                    print_line(&format!("Unmounted device {}", id));
                } else {
                    print_line("Failed to unmount device");
                }
            } else {
                print_line("Usage: unmount-device <device_id>");
            }
        }
        "list-ifaces" => {
            unsafe {
                for iface_opt in NETWORK_INTERFACES.iter() {
                    if let Some(iface) = iface_opt {
                        print_line(&format!("Iface {}: MAC={:02X?} IPv4={:?} NS={} State={:?}",
                            iface.id, iface.mac_address, iface.ip_address_v4, iface.net_namespace, iface.state));
                    }
                }
            }
        }
        "iface-up" => {
            if parts.len() == 2 {
                let id = parts[1].parse::<usize>().unwrap_or(usize::MAX);
                if id != usize::MAX {
                    bring_interface_up(id);
                }
            } else {
                print_line("Usage: iface-up <iface_id>");
            }
        }
        "iface-down" => {
            if parts.len() == 2 {
                let id = parts[1].parse::<usize>().unwrap_or(usize::MAX);
                if id != usize::MAX {
                    bring_interface_down(id);
                }
            } else {
                print_line("Usage: iface-down <iface_id>");
            }
        }
        _ => {
            // fallback to previous handle_command
            handle_command_basic(cmd);
        }
    }
}

fn handle_command_basic(cmd: &str) {
    // (previous command handling code here)
}

// === Cluster Management Extensions ===

fn rebalance_cluster_load() {
    unsafe {
        print_line("[Cluster] Starting load rebalance...");
        for node_opt in NODES.iter_mut() {
            if let Some(node) = node_opt {
                let total_cpu_usage: u32 = node.cpus.iter().map(|c| c.usage_percent as u32).sum();
                if total_cpu_usage > (MAX_CPUS_PER_NODE as u32 * 80) {
                    print_line(&format!("Node {} overloaded, migrating VMs...", node.id));
                    migrate_vm_to_less_loaded_node(node.id);
                }
            }
        }
    }
}

fn migrate_vm_to_less_loaded_node(from_node: usize) {
    unsafe {
        let mut target_node = None;
        let mut min_load = u32::MAX;
        for node_opt in NODES.iter() {
            if let Some(node) = node_opt {
                if node.id != from_node {
                    let load: u32 = node.cpus.iter().map(|c| c.usage_percent as u32).sum();
                    if load < min_load {
                        min_load = load;
                        target_node = Some(node.id);
                    }
                }
            }
        }
        if let Some(target) = target_node {
            for vm_opt in VMS.iter_mut() {
                if let Some(vm) = vm_opt {
                    if vm.node_id == from_node {
                        vm.migration_target = Some(target);
                        vm.state = VmState::Migrating;
                        print_line(&format!("Migrating VM {} from node {} to node {}", vm.id, from_node, target));
                        break;
                    }
                }
            }
        }
    }
}

fn perform_vm_migrations() {
    unsafe {
        for vm_opt in VMS.iter_mut() {
            if let Some(vm) = vm_opt {
                if let Some(target) = vm.migration_target {
                    // simulate migration complete
                    vm.node_id = target;
                    vm.migration_target = None;
                    vm.state = VmState::Running;
                    print_line(&format!("Migration of VM {} to node {} complete", vm.id, target));
                }
            }
        }
    }
}

// === Extended Telemetry and Metrics Export ===

#[repr(C)]
pub struct Metric {
    name: &'static str,
    value: u64,
}

static mut METRICS: [Option<Metric>; 128] = [None; 128];

fn export_metrics() {
    unsafe {
        METRICS[0] = Some(Metric { name: "total_nodes", value: NODES.iter().filter(|n| n.is_some()).count() as u64 });
        METRICS[1] = Some(Metric { name: "total_vms", value: VMS.iter().filter(|v| v.is_some()).count() as u64 });
        METRICS[2] = Some(Metric { name: "total_containers", value: CONTAINERS.iter().filter(|c| c.is_some()).count() as u64 });
        // Example alert: total containers > 900 triggers warning
        alert_on_metric("total_containers", 900);
    }
}
// === Security & Access Control ===

#[repr(C)]
pub struct AccessControlList {
    resource_id: usize,
    allowed_user_ids: [usize; 16],
    allowed_vm_ids: [usize; 16],
    allowed_container_ids: [usize; 32],
}

static mut ACLS: [Option<AccessControlList>; 64] = [None; 64];

fn check_access(user_id: usize, resource_id: usize, vm_id: Option<usize>, container_id: Option<usize>) -> bool {
    unsafe {
        for acl_opt in ACLS.iter() {
            if let Some(acl) = acl_opt {
                if acl.resource_id == resource_id {
                    if acl.allowed_user_ids.contains(&user_id) &&
                       (vm_id.is_none() || acl.allowed_vm_ids.contains(&vm_id.unwrap())) &&
                       (container_id.is_none() || acl.allowed_container_ids.contains(&container_id.unwrap()))
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// === Fault Tolerance and Recovery ===

fn checkpoint_vm(vm_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if vm.state == VmState::Running {
                vm.snapshot_enabled = true;
                vm.restore_point = Some(vm.uptime_ticks);
                print_line(&format!("[FaultTolerance] Checkpoint created for VM {}", vm_id));
            }
        }
    }
}

fn restore_vm_from_checkpoint(vm_id: usize) {
    unsafe {
        if let Some(vm) = &mut VMS[vm_id] {
            if let Some(rp) = vm.restore_point {
                vm.uptime_ticks = rp;
                vm.state = VmState::Running;
                print_line(&format!("[FaultTolerance] VM {} restored to checkpoint at tick {}", vm_id, rp));
            } else {
                print_line("[FaultTolerance] No checkpoint found");
            }
        }
    }
}

// === Distributed Logging ===

static mut LOG_BUFFER: [u8; 4096] = [0; 4096];
static mut LOG_INDEX: usize = 0;

fn log_message(msg: &str) {
    unsafe {
        let bytes = msg.as_bytes();
        if LOG_INDEX + bytes.len() < LOG_BUFFER.len() {
            for &b in bytes {
                LOG_BUFFER[LOG_INDEX] = b;
                LOG_INDEX += 1;
            }
            LOG_BUFFER[LOG_INDEX] = b'\n';
            LOG_INDEX += 1;
        } else {
            // Rotate logs (simple overwrite for example)
            LOG_INDEX = 0;
        }
    }
}

// === AI-Driven Optimization (Stub) ===

fn ai_optimize_cluster() {
    // Placeholder for future AI workload optimization
    print_line("[AI] Running optimization algorithms...");
    // Example: redistribute workloads based on predicted resource needs
}

// === Main Event Loop Enhancements ===

fn start_event_loop() {
    loop {
        poll_uart();
        poll_mouse();
        poll_cluster();
        update_telemetry();
        perform_vm_migrations();
        rebalance_cluster_load();
        export_metrics();
        ai_optimize_cluster();
        // Add sleep/delay in real implementation
    }
}

// === Final CLI help update ===

fn handle_command_basic(cmd: &str) {
    match cmd {
        "help" => {
            print_line("Available commands:");
            print_line("  status            - Show cluster and VM status");
            print_line("  list-vms          - List all VMs");
            print_line("  create-vm <id> <node> <cpu_mask> - Create a VM");
            print_line("  delete-vm <id>    - Delete a VM");
            print_line("  list-nodes        - List all nodes");
            print_line("  list-devices      - List storage devices");
            print_line("  mount-device <id> <mount_point> - Mount a storage device");
            print_line("  unmount-device <id> - Unmount a storage device");
            print_line("  list-ifaces       - List network interfaces");
            print_line("  iface-up <id>     - Bring interface up");
            print_line("  iface-down <id>   - Bring interface down");
            print_line("  create-container <label> - Create a container");
            print_line("  start-container <id> - Start a container");
            print_line("  stop-container <id> - Stop a container");
            print_line("  checkpoint-vm <id> - Create VM checkpoint");
            print_line("  restore-vm <id>   - Restore VM from checkpoint");
            print_line("  help              - Show this help message");
        }
        _ => print_line("Unknown command. Type 'help' for commands."),
    }
}

// === Panic Handler ===

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// === END OF FILE ===


