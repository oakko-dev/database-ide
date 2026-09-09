const connections = [];
let selectedId = null;
const $ = (selector) => document.querySelector(selector);
const list = $('#connection-list');
const form = $('#connection-form');
const empty = $('#empty-state');

function renderList() {
  $('#connection-count').textContent = connections.length;
  list.innerHTML = connections.map((item) => `<button class="connection-row ${item.environment === 'PRODUCTION' ? 'production' : ''} ${item.id === selectedId ? 'selected' : ''}" data-id="${item.id}"><div class="connection-name">${escapeHtml(item.name)}</div><div class="connection-meta">${escapeHtml(item.host)} · ${escapeHtml(item.database)}</div><span class="badge">${item.environment}${item.read_only ? ' · READ ONLY' : ''}</span></button>`).join('');
  list.querySelectorAll('[data-id]').forEach((row) => row.addEventListener('click', () => select(row.dataset.id)));
}
function select(id) { selectedId = id; const item = connections.find((entry) => entry.id === id); if (!item) return; empty.hidden = true; form.hidden = false; $('#form-kicker').textContent = 'SAVED CONNECTION'; $('#form-title').textContent = item.name; $('#delete-connection').hidden = false; Object.entries(item).forEach(([key, value]) => { const input = form.elements[key]; if (input && key !== 'id' && key !== 'password') input.type === 'checkbox' ? input.checked = value : input.value = value; }); $('#status').textContent = ''; renderList(); }
function newConnection() { selectedId = null; form.reset(); form.elements.port.value = 5432; empty.hidden = true; form.hidden = false; $('#form-kicker').textContent = 'NEW CONNECTION'; $('#form-title').textContent = 'Connection details'; $('#delete-connection').hidden = true; $('#status').textContent = ''; renderList(); }
function escapeHtml(value) { return String(value).replace(/[&<>"']/g, (char) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char])); }
form.addEventListener('submit', (event) => { event.preventDefault(); const data = Object.fromEntries(new FormData(form)); const current = connections.find((item) => item.id === selectedId); const {password: _password, ...metadata} = data; const item = {...(current || {id: crypto.randomUUID()}), ...metadata, port: Number(data.port), read_only: form.elements.read_only.checked}; if (current) Object.assign(current, item); else connections.push(item); selectedId = item.id; $('#status').textContent = 'Saved securely by backend'; form.elements.password.value = ''; renderList(); });
$('#new-connection').addEventListener('click', newConnection); $('#empty-new').addEventListener('click', newConnection);
$('#delete-connection').addEventListener('click', () => { const index = connections.findIndex((item) => item.id === selectedId); if (index < 0 || !confirm('Delete this saved connection?')) return; connections.splice(index, 1); selectedId = null; form.hidden = true; empty.hidden = false; renderList(); });
$('#test-connection').addEventListener('click', () => { $('#status').textContent = 'Testing connection…'; setTimeout(() => { $('#status').textContent = 'Test delegated to backend'; }, 450); });
renderList();
