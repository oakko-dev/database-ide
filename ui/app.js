import { bridge } from './bridge.js';

let connections = [];
let selectedId = null;
let selectedSchema = null;
const $ = (selector) => document.querySelector(selector);
const list = $('#connection-list');
const form = $('#connection-form');
const empty = $('#empty-state');
const browser = $('#schema-browser');
const tree = $('#schema-tree');
const details = $('#relation-details');

function escapeHtml(value) { return String(value).replace(/[&<>"']/g, (char) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char])); }
function setSchemaStatus(message, isError = false) { const status = $('#schema-status'); status.textContent = message; status.style.color = isError ? '#ef9b9b' : ''; }
function renderList() {
  $('#connection-count').textContent = connections.length;
  list.innerHTML = connections.map((item) => `<button class="connection-row ${item.environment === 'PRODUCTION' ? 'production' : ''} ${item.id === selectedId ? 'selected' : ''}" data-id="${item.id}"><div class="connection-name">${escapeHtml(item.name)}</div><div class="connection-meta">${escapeHtml(item.host)} · ${escapeHtml(item.database)}</div><span class="badge">${item.environment}${item.read_only ? ' · READ ONLY' : ''}</span></button>`).join('');
  list.querySelectorAll('[data-id]').forEach((row) => row.addEventListener('click', () => select(row.dataset.id)));
}
function renderDetails(item) {
  if (!item) { details.innerHTML = '<p class="browser-placeholder">Select a table or view to inspect its columns, indexes, and constraints.</p>'; return; }
  const rows = (items, columns) => items.length ? `<table class="detail-table"><thead><tr>${columns.map((column) => `<th>${column.label}</th>`).join('')}</tr></thead><tbody>${items.map((entry) => `<tr>${columns.map((column) => `<td>${escapeHtml(entry[column.key])}</td>`).join('')}</tr>`).join('')}</tbody></table>` : '<p class="browser-placeholder">None found.</p>';
  details.innerHTML = `<h3 class="detail-title">${escapeHtml(item.relation.name)} <span class="badge">${escapeHtml(item.relation.relation_type)}</span></h3><div class="detail-section"><h3>Columns</h3>${rows(item.columns, [{key:'name',label:'Name'},{key:'data_type',label:'Type'},{key:'nullable',label:'Nullable'}])}</div><div class="detail-section"><h3>Indexes</h3>${rows(item.indexes, [{key:'name',label:'Name'},{key:'definition',label:'Definition'}])}</div><div class="detail-section"><h3>Constraints</h3>${rows(item.constraints, [{key:'name',label:'Name'},{key:'constraint_type',label:'Type'}])}</div>`;
}
async function loadRelations(schemaName) {
  selectedSchema = schemaName;
  setSchemaStatus(`Loading ${schemaName}…`);
  const relationButtons = document.querySelectorAll('.relation-node'); relationButtons.forEach((button) => { button.hidden = true; });
  try {
    const relations = await bridge.listRelations(selectedId, schemaName);
    document.querySelectorAll('[data-schema-group]').forEach((group) => group.remove());
    const schemaButton = document.querySelector(`[data-schema="${CSS.escape(schemaName)}"]`);
    const group = document.createElement('div'); group.dataset.schemaGroup = schemaName;
    group.innerHTML = relations.length ? relations.map((relation) => `<button class="relation-node" data-relation="${escapeHtml(relation.name)}" title="${escapeHtml(relation.relation_type)}">${escapeHtml(relation.name)}</button>`).join('') : '<p class="browser-placeholder">No tables or views.</p>';
    schemaButton?.after(group);
    group.querySelectorAll('[data-relation]').forEach((button) => button.addEventListener('click', () => loadDetails(schemaName, button.dataset.relation)));
    setSchemaStatus(`${relations.length} relation${relations.length === 1 ? '' : 's'} in ${schemaName}`);
  } catch (error) { setSchemaStatus(`Could not load relations: ${error}`, true); }
}
async function loadDetails(schemaName, relationName) {
  setSchemaStatus(`Loading ${schemaName}.${relationName}…`);
  try { renderDetails(await bridge.describeRelation(selectedId, schemaName, relationName)); setSchemaStatus(`${schemaName}.${relationName}`); } catch (error) { renderDetails(null); setSchemaStatus(`Could not load relation: ${error}`, true); }
}
async function loadSchemaBrowser(id) {
  browser.hidden = false; tree.innerHTML = '<p class="browser-placeholder">Loading schemas…</p>'; renderDetails(null); setSchemaStatus('Loading…');
  try {
    const schemas = await bridge.listSchemas(id);
    tree.innerHTML = schemas.length ? schemas.map((schema) => `<button class="schema-node" data-schema="${escapeHtml(schema.name)}">▾ ${escapeHtml(schema.name)}</button>`).join('') : '<p class="browser-placeholder">No user schemas found. Run the native app to explore PostgreSQL.</p>';
    tree.querySelectorAll('[data-schema]').forEach((button) => button.addEventListener('click', () => loadRelations(button.dataset.schema)));
    setSchemaStatus(schemas.length ? `${schemas.length} schema${schemas.length === 1 ? '' : 's'}` : 'No schemas');
  } catch (error) { tree.innerHTML = '<p class="browser-placeholder">Schema loading failed.</p>'; setSchemaStatus(`Could not load schemas: ${error}`, true); }
}
function select(id) { selectedId = id; const item = connections.find((entry) => entry.id === id); if (!item) return; empty.hidden = true; form.hidden = false; $('#form-kicker').textContent = 'SAVED CONNECTION'; $('#form-title').textContent = item.name; $('#delete-connection').hidden = false; Object.entries(item).forEach(([key, value]) => { const input = form.elements[key]; if (input && key !== 'id' && key !== 'password') input.type === 'checkbox' ? input.checked = value : input.value = value; }); $('#status').textContent = ''; renderList(); loadSchemaBrowser(id); }
function newConnection() { selectedId = null; browser.hidden = true; form.reset(); form.elements.port.value = 5432; empty.hidden = true; form.hidden = false; $('#form-kicker').textContent = 'NEW CONNECTION'; $('#form-title').textContent = 'Connection details'; $('#delete-connection').hidden = true; $('#status').textContent = ''; renderList(); }
form.addEventListener('submit', async (event) => { event.preventDefault(); const data = Object.fromEntries(new FormData(form)); const {password, ...metadata} = data; const input = {...metadata, password, port: Number(data.port), read_only: form.elements.read_only.checked}; try { const item = selectedId ? await bridge.update(selectedId, input) : await bridge.create(input); connections = selectedId ? connections.map((entry) => entry.id === selectedId ? item : entry) : [...connections, item]; selectedId = item.id; $('#status').textContent = bridge.native ? 'Saved securely' : 'Saved in browser preview'; form.elements.password.value = ''; renderList(); loadSchemaBrowser(item.id); } catch (error) { $('#status').textContent = `Save failed: ${error}`; } });
$('#new-connection').addEventListener('click', newConnection); $('#empty-new').addEventListener('click', newConnection); $('#refresh-schema').addEventListener('click', () => selectedId && loadSchemaBrowser(selectedId));
$('#delete-connection').addEventListener('click', async () => { if (!selectedId || !confirm('Delete this saved connection?')) return; try { await bridge.remove(selectedId); connections = connections.filter((item) => item.id !== selectedId); selectedId = null; browser.hidden = true; form.hidden = true; empty.hidden = false; renderList(); } catch (error) { $('#status').textContent = `Delete failed: ${error}`; } });
$('#test-connection').addEventListener('click', async () => { if (!selectedId) { $('#status').textContent = 'Save the connection before testing'; return; } $('#status').textContent = 'Testing connection…'; try { await bridge.test(selectedId); $('#status').textContent = 'Connection successful'; } catch (error) { $('#status').textContent = `Connection failed: ${error}`; } });
connections = await bridge.list(); renderList();
