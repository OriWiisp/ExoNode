# VEXOGA Operating System

**VEXOGA** is an experimental operating system designed to run on **RISC-V architecture**. It enables distributed computing by managing a cluster of **EXOVega processor cards** and their **virtual machines (VMs)**. It also features a **CLI** and a **GUI** for managing the cluster, VMs, and more.

VEXOGA is designed for high-performance tasks and virtualization in a cluster environment. It leverages PCIe communication for managing multiple nodes and VMs, and includes basic virtualization support.

---

## Features

- **Cluster Management**: Control and monitor the status of EXOVega processor nodes.
- **Virtual Machine Management**: Create, delete, and list virtual machines running on the system.
- **Graphical User Interface (GUI)**: A dashboard that displays the status of the cluster and VMs.
- **Command Line Interface (CLI)**: Control the system via a UART terminal.
- **VM Networking**: Simulate network communication between virtual machines.
- **Mouse and Keyboard Support**: Basic GUI interactivity with mouse clicks and keyboard input.

---

## Getting Started

### Hardware Requirements

- **RISC-V Processor**: A compatible RISC-V-based processor.
- **EXOVega Cards**: Cluster nodes running EXOVega processors connected via PCIe.
- **Controller Card**: A central card communicating with EXOVega processor nodes over PCIe.
- **Display**: For GUI visualization (HDMI, VGA, etc.).
- **Mouse & Keyboard**: For GUI interaction.

### Setting Up the Environment

1. **Install Rust**:
   - Make sure you have the latest version of Rust installed on your system. You can download it from the [official Rust website](https://www.rust-lang.org/tools/install).
   - To install, run the following command:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```

2. **Install `riscv32` Target**:
   - VEXOGA is designed to run on a 32-bit RISC-V architecture. You need to install the target:
     ```bash
     rustup target add riscv32imac-unknown-none-elf
     ```

3. **Set Up Emulator** (optional):
   - To run the system on an emulator (such as QEMU), you need to install the following:
     ```bash
     sudo apt install qemu-system-riscv
     ```

---

## System Architecture

VEXOGA operates on a **controller card** that communicates over **PCIe** with multiple **EXOVega cards**, each hosting several **RISC-V MCU processor cores**.

- **Controller Card**: This card manages the cluster and runs the VEXOGA OS.
- **EXOVega Processor Nodes**: Each node is a processing unit that may host several VMs.
- **Virtual Machines**: VMs run on EXOVega processors, with simple networking between them.

---

## Building and Running VEXOGA

### Building the Kernel

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/VEXOGA.git
   cd VEXOGA
