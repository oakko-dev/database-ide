import { bridge } from './bridge.js';

let connections = [];
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
form.addEventListener('submit', async (event) => { event.preventDefault(); const data = Object.fromEntries(new FormData(form)); const {password, ...metadata} = data; const input = {...metadata, password, port: Number(data.port), read_only: form.elements.read_only.checked}; try { const item = selectedId ? await bridge.update(selectedId, input) : await bridge.create(input); connections = selectedId ? connections.map((entry) => entry.id === selectedId ? item : entry) : [...connections, item]; selectedId = item.id; $('#status').textContent = bridge.native ? 'Saved securely' : 'Saved in browser preview'; form.elements.password.value = ''; renderList(); } catch (error) { $('#status').textContent = `Save failed: ${error}`; } });
$('#new-connection').addEventListener('click', newConnection); $('#empty-new').addEventListener('click', newConnection);
$('#delete-connection').addEventListener('click', async () => { if (!selectedId || !confirm('Delete this saved connection?')) return; try { await bridge.remove(selectedId); connections = connections.filter((item) => item.id !== selectedId); selectedId = null; form.hidden = true; empty.hidden = false; renderList(); } catch (error) { $('#status').textContent = `Delete failed: ${error}`; } });
$('#test-connection').addEventListener('click', async () => { if (!selectedId) { $('#status').textContent = 'Save the connection before testing'; return; } $('#status').textContent = 'Testing connection…'; try { await bridge.test(selectedId); $('#status').textContent = 'Connection successful'; } catch (error) { $('#status').textContent = `Connection failed: ${error}`; } });
connections = await bridge.list();
renderList();
