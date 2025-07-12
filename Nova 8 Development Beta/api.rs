use warp::Filter;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::convert::Infallible;

use crate::vmcore::{self, VM, VmState, MAX_VMS};

/// Thread-safe wrapper around the VM storage
type SharedVms = Arc<Mutex<[Option<VM>; MAX_VMS]>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VmInfo {
    id: usize,
    label: String,
    state: String,
    active: bool,
}

#[derive(Deserialize)]
struct VmCreateRequest {
    id: usize,
    label: String,
}

#[derive(Deserialize)]
struct VmIdRequest {
    id: usize,
}

/// Converts a VM to VmInfo for serialization
fn vm_to_info(vm: &VM) -> VmInfo {
    VmInfo {
        id: vm.id,
        label: vm.label.to_string(),
        state: format!("{:?}", vm.state),
        active: vm.active,
    }
}

/// Handler to list all active VMs
async fn list_vms(vms: SharedVms) -> Result<impl warp::Reply, Infallible> {
    let vms_guard = vms.lock().unwrap();
    let vm_list: Vec<VmInfo> = vms_guard.iter()
        .filter_map(|vm_opt| vm_opt.as_ref())
        .map(vm_to_info)
        .collect();
    Ok(warp::reply::json(&vm_list))
}

/// Handler to create a VM
async fn create_vm(vms: SharedVms, req: VmCreateRequest) -> Result<impl warp::Reply, Infallible> {
    let mut vms_guard = vms.lock().unwrap();

    if req.id >= MAX_VMS {
        return Ok(warp::reply::with_status(
            "VM ID out of range",
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if vms_guard[req.id].is_some() {
        return Ok(warp::reply::with_status(
            "VM already exists",
            warp::http::StatusCode::CONFLICT,
        ));
    }

    match vmcore::create_vm(req.id, Box::leak(req.label.into_boxed_str())) {
        Ok(_) => {
            // Sync shared storage with vmcore::VMS after creation (unsafe)
            unsafe {
                vms_guard[req.id] = vmcore::VMS[req.id];
            }
            Ok(warp::reply::with_status(
                "VM created",
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(e) => Ok(warp::reply::with_status(
            e,
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Handler to delete a VM
async fn delete_vm(vms: SharedVms, req: VmIdRequest) -> Result<impl warp::Reply, Infallible> {
    let mut vms_guard = vms.lock().unwrap();

    if req.id >= MAX_VMS || vms_guard[req.id].is_none() {
        return Ok(warp::reply::with_status(
            "VM does not exist",
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    match vmcore::delete_vm(req.id) {
        Ok(_) => {
            // Sync shared storage with vmcore::VMS after deletion (unsafe)
            unsafe {
                vms_guard[req.id] = None;
            }
            Ok(warp::reply::with_status(
                "VM deleted",
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => Ok(warp::reply::with_status(
            e,
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Handler to start a VM
async fn start_vm(vms: SharedVms, req: VmIdRequest) -> Result<impl warp::Reply, Infallible> {
    let mut vms_guard = vms.lock().unwrap();

    if req.id >= MAX_VMS {
        return Ok(warp::reply::with_status(
            "VM ID out of range",
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if let Some(vm) = &mut vms_guard[req.id] {
        if vm.state == VmState::Running {
            return Ok(warp::reply::with_status(
                "VM already running",
                warp::http::StatusCode::CONFLICT,
            ));
        }
        // Start VM logic — here just set the state and active flag
        vm.state = VmState::Running;
        vm.active = true;

        // Sync back to global unsafe VMS storage
        unsafe {
            vmcore::VMS[req.id] = Some(*vm);
        }

        Ok(warp::reply::with_status(
            "VM started",
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            "VM not found",
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

/// Handler to stop a VM
async fn stop_vm(vms: SharedVms, req: VmIdRequest) -> Result<impl warp::Reply, Infallible> {
    let mut vms_guard = vms.lock().unwrap();

    if req.id >= MAX_VMS {
        return Ok(warp::reply::with_status(
            "VM ID out of range",
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if let Some(vm) = &mut vms_guard[req.id] {
        if vm.state == VmState::Stopped {
            return Ok(warp::reply::with_status(
                "VM already stopped",
                warp::http::StatusCode::CONFLICT,
            ));
        }
        // Stop VM logic — here just set the state and active flag
        vm.state = VmState::Stopped;
        vm.active = false;

        // Sync back to global unsafe VMS storage
        unsafe {
            vmcore::VMS[req.id] = Some(*vm);
        }

        Ok(warp::reply::with_status(
            "VM stopped",
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            "VM not found",
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    // Initialize thread-safe VMs wrapper (copy from unsafe global)
    let vms = Arc::new(Mutex::new(unsafe { vmcore::VMS }));

    // Filters with cloning Arc for thread safety
    let vms_filter = warp::any().map(move || vms.clone());

    // Routes
    let list_route = warp::path!("vms")
        .and(warp::get())
        .and(vms_filter.clone())
        .and_then(list_vms);

    let create_route = warp::path!("vms" / "create")
        .and(warp::post())
        .and(warp::body::json())
        .and(vms_filter.clone())
        .and_then(create_vm);

    let delete_route = warp::path!("vms" / "delete")
        .and(warp::post())
        .and(warp::body::json())
        .and(vms_filter.clone())
        .and_then(delete_vm);

    let start_route = warp::path!("vms" / "start")
        .and(warp::post())
        .and(warp::body::json())
        .and(vms_filter.clone())
        .and_then(start_vm);

    let stop_route = warp::path!("vms" / "stop")
        .and(warp::post())
        .and(warp::body::json())
        .and(vms_filter)
        .and_then(stop_vm);

    let routes = list_route
        .or(create_route)
        .or(delete_route)
        .or(start_route)
        .or(stop_route)
        .with(warp::cors().allow_any_origin());

    println!("API server running at http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
