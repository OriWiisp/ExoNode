const API_BASE = 'http://localhost:3030/vms';

async function fetchVMs() {
  try {
    const res = await fetch(API_BASE);
    if (!res.ok) throw new Error('Failed to fetch VMs');
    return await res.json();
  } catch (err) {
    showStatus(err.message, true);
    return [];
  }
}

async function refreshVMs() {
  clearStatus();
  const vmList = await fetchVMs();
  const tbody = document.querySelector('#vm-table tbody');
  tbody.innerHTML = '';

  vmList.forEach(vm => {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${vm.id}</td>
      <td>${vm.label}</td>
      <td>${vm.state}</td>
      <td>${vm.active}</td>
      <td>
        <button class="start" onclick="startVM(${vm.id})">Start</button>
        <button class="stop" onclick="stopVM(${vm.id})">Stop</button>
        <button class="delete" onclick="deleteVM(${vm.id})">Delete</button>
      </td>
    `;
    tbody.appendChild(tr);
  });
}

async function postAction(endpoint, data) {
  try {
    const res = await fetch(`${API_BASE}/${endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(text || 'Server error');
    }
    showStatus(`${endpoint.charAt(0).toUpperCase() + endpoint.slice(1)} successful`);
    refreshVMs();
  } catch (err) {
    showStatus(err.message, true);
  }
}

function showStatus(msg, isError = false) {
  const statusDiv = document.getElementById('status-message');
  statusDiv.textContent = msg;
  statusDiv.style.color = isError ? '#f44336' : '#4caf50';
}

function clearStatus() {
  const statusDiv = document.getElementById('status-message');
  statusDiv.textContent = '';
}

async function startVM(id) {
  clearStatus();
  await postAction('start', { id });
}

async function stopVM(id) {
  clearStatus();
  await postAction('stop', { id });
}

async function deleteVM(id) {
  if (!confirm(`Are you sure you want to delete VM ${id}?`)) return;
  clearStatus();
  await postAction('delete', { id });
}

async function createVM() {
  clearStatus();
  const idInput = document.getElementById('new-vm-id');
  const labelInput = document.getElementById('new-vm-label');
  const id = parseInt(idInput.value);
  const label = labelInput.value.trim();

  if (isNaN(id) || id < 0 || id > 1023) {
    showStatus('Please enter a valid VM ID (0-1023)', true);
    return;
  }
  if (!label) {
    showStatus('Please enter a VM label', true);
    return;
  }

  await postAction('create', { id, label });

  idInput.value = '';
  labelInput.value = '';
}

// Attach create button handler on load
document.addEventListener('DOMContentLoaded', () => {
  document.querySelector('button.create').addEventListener('click', createVM);
  document.querySelector('button[onclick="refreshVMs()"]').addEventListener('click', refreshVMs);
  refreshVMs();
});

// Expose functions for inline HTML onclick handlers
window.startVM = startVM;
window.stopVM = stopVM;
window.deleteVM = deleteVM;
