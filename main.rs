#![no_std]
#![no_main]

use core::panic::PanicInfo;

const FB_BASE: usize = 0x1000_0000; // Framebuffer base address
const W: usize = 1024;              // Width of the framebuffer
const H: usize = 768;               // Height of the framebuffer

const MAX_VMS: usize = 8;
const MAX_NODES: usize = 16;
const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;

type Color = u32;

#[repr(C)]
pub struct VM {
    id: usize,
    active: bool,
    network_ip: Option<u32>, // Simulate a network IP for each VM
}

#[repr(C)]
pub struct Node {
    id: usize,
    online: bool,
}

static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];
static mut NODES: [Option<Node>; MAX_NODES] = [None; MAX_NODES];
static mut MOUSE_X: usize = 0;
static mut MOUSE_Y: usize = 0;
static mut MOUSE_CLICKED: bool = false;

static mut CLI_BUFFER: [u8; 128] = [0; 128];
static mut CLI_INDEX: usize = 0;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    uart_init();          // Initialize UART for CLI
    framebuffer_init();   // Initialize framebuffer (GUI)
    pcie_init();          // Simulated PCIe initialization
    discover_nodes();     // Simulate cluster discovery (EXOVega cards)
    hypervisor_init();    // VM management system
    startup_screen();     // Show startup screen
    start_event_loop();   // Start the event loop for input handling

    loop {
        poll_uart(); // Handle user input over UART (CLI)
        poll_cluster(); // Handle cluster node communication
    }
}

/// === UART ===
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
                print_line(""); // Newline after command
                print("> ");
            } else if CLI_INDEX < 127 {
                uart_putc(c);
                CLI_BUFFER[CLI_INDEX] = c;
                CLI_INDEX += 1;
            }
        }
    }
}

fn handle_command(cmd: &str) {
    if cmd.starts_with("status") {
        print_line("[SYS] Cluster + VM Status:");
        list_nodes();
        list_vms();
    } else if cmd.starts_with("create-vm") {
        if let Some(id_str) = cmd.split_whitespace().nth(1) {
            if let Ok(id) = id_str.parse::<usize>() {
                create_vm(id);
            } else {
                print_line("Invalid VM ID.");
            }
        } else {
            print_line("Usage: create-vm <id>");
        }
    } else if cmd.starts_with("delete-vm") {
        if let Some(id_str) = cmd.split_whitespace().nth(1) {
            if let Ok(id) = id_str.parse::<usize>() {
                delete_vm(id);
            } else {
                print_line("Invalid VM ID.");
            }
        } else {
            print_line("Usage: delete-vm <id>");
        }
    } else if cmd == "list-vms" {
        list_vms();
    } else if cmd == "help" {
        print_line("Available commands:");
        print_line("  status        - Show cluster/VMs");
        print_line("  create-vm ID  - Start a VM");
        print_line("  delete-vm ID  - Stop and remove a VM");
        print_line("  list-vms      - Show all VMs");
        print_line("  help          - Show help");
    } else {
        print_line("Unknown command. Type 'help'.");
    }
}

/// === PCIe Initialization (Simulated) ===
fn pcie_init() {
    print_line("[PCIe] Initialized (simulated)");
}

fn discover_nodes() {
    unsafe {
        for i in 0..4 {
            NODES[i] = Some(Node { id: i, online: true });
            print_node(i, "EXOVega card detected");
        }
    }
}

/// === Virtualization Support ===
fn hypervisor_init() {
    print_line("[HV] Virtualization enabled");
}

fn create_vm(id: usize) {
    unsafe {
        if id >= MAX_VMS {
            print_line("[VM] ID too high");
            return;
        }

        if VMS[id].is_some() {
            print_line("[VM] Already exists");
            return;
        }

        VMS[id] = Some(VM {
            id,
            active: true,
            network_ip: Some(0xC0A80001), // Simulating a network IP (192.168.0.1)
        });

        print_vm(id, "Created successfully");
    }
}

fn delete_vm(id: usize) {
    unsafe {
        if let Some(vm) = VMS[id].take() {
            print_vm(id, "Stopped and removed successfully");
        } else {
            print_line("[VM] VM does not exist");
        }
    }
}

fn list_vms() {
    unsafe {
        for i in 0..MAX_VMS {
            if let Some(vm) = &VMS[i] {
                let msg = format!("VM {}: {} - IP: {}", vm.id, if vm.active { "Running" } else { "Stopped" }, vm.network_ip.unwrap_or(0));
                print_line(&msg);
            }
        }
    }
}

fn list_nodes() {
    unsafe {
        for i in 0..MAX_NODES {
            if let Some(n) = &NODES[i] {
                let msg = format!("Node {}: {}", n.id, if n.online { "Online" } else { "Offline" });
                print_line(&msg);
            }
        }
    }
}

/// === GUI (Framebuffer) ===
fn framebuffer_init() {
    clear_screen(0x101010); // Initialize background color
    draw_dashboard(); // Draw the initial dashboard
}

fn clear_screen(color: Color) {
    unsafe {
        let fb = FB_BASE as *mut Color;
        for i in 0..(W * H) {
            fb.add(i).write_volatile(color);
        }
    }
}

fn put_pixel(x: usize, y: usize, color: Color) {
    if x < W && y < H {
        unsafe {
            let offset = y * W + x;
            let fb = FB_BASE as *mut Color;
            fb.add(offset).write_volatile(color);
        }
    }
}

fn fill_rect(x: usize, y: usize, w: usize, h: usize, color: Color) {
    for dy in 0..h {
        for dx in 0..w {
            put_pixel(x + dx, y + dy, color);
        }
    }
}

fn draw_window(title: &str, x: usize, y: usize, w: usize, h: usize) {
    fill_rect(x, y, w, h, 0x2A2A2A); // Window background
    fill_rect(x, y, w, 24, 0x3A3A3A); // Title bar
    draw_text(title, x + 8, y + 4, 0xFFFFFF); // Title text
}

fn draw_button(label: &str, x: usize, y: usize, w: usize, h: usize, color: Color) {
    fill_rect(x, y, w, h, color); // Button background
    draw_text(label, x + 8, y + 12, 0xFFFFFF); // Button text
}

fn draw_text(s: &str, x: usize, y: usize, color: Color) {
    for (i, c) in s.chars().enumerate() {
        let x_offset = x + i * FONT_WIDTH;
        if x_offset + FONT_WIDTH > W { break; }
        draw_char(c, x_offset, y, color);
    }
}

fn draw_char(c: char, x: usize, y: usize, color: Color) {
    // Simple ASCII-based font, for demonstration purposes
    let pattern = match c {
        ' ' => [0u8; FONT_HEIGHT],
        'A' => [0b11110000, 0b10001000, 0b11111100, 0b10001000, 0b10001000, 0b10001000, 0b00000000],
        // Add more characters as needed...
        _ => [0u8; FONT_HEIGHT],
    };

    for row in 0..FONT_HEIGHT {
        for col in 0..FONT_WIDTH {
            if (pattern[row] >> (FONT_WIDTH - 1 - col)) & 1 == 1 {
                put_pixel(x + col, y + row, color);
            }
        }
    }
}

fn draw_dashboard() {
    draw_window("VEXOGA Dashboard", 50, 50, 924, 668);
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
                let label = if vm.active { "Running" } else { "Stopped" };
                draw_text(&format!("VM {}: {}", vm.id, label), 60, y, 0x00FF00);
            }
        }
    }

    draw_button("Create VM", 800, 100, 150, 40, 0x4CAF50); // Green button
    draw_button("Delete VM", 800, 150, 150, 40, 0xFF5722); // Red button
    draw_button("Refresh",   800, 200, 150, 40, 0x2196F3); // Blue button
}

/// === Mouse Input (Simulated) ===
fn poll_mouse() {
    // Simulating mouse movements and click detection
    unsafe {
        if MOUSE_CLICKED {
            // Detect if a button was clicked based on mouse position
            if MOUSE_X > 800 && MOUSE_X < 950 {
                if MOUSE_Y > 100 && MOUSE_Y < 140 {
                    create_vm(1); // Create VM button click
                } else if MOUSE_Y > 150 && MOUSE_Y < 190 {
                    delete_vm(1); // Delete VM button click
                } else if MOUSE_Y > 200 && MOUSE_Y < 240 {
                    print_line("Refreshing..."); // Refresh button click
                }
            }
            MOUSE_CLICKED = false;
        }
    }
}

/// === UART SIMULATION ===
fn uart_getc() -> Option<u8> {
    // Simulate getting a character via UART
    None
}

fn uart_putc(c: u8) {
    // Simulate UART output
}

fn print_line(msg: &str) {
    // Simulate printing to UART
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

/// Panic Handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
