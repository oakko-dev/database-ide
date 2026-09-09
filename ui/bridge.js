const nativeInvoke = window.__TAURI_INTERNALS__?.invoke;
const memory = [];

export const bridge = {
  native: Boolean(nativeInvoke),
  async list() { return nativeInvoke ? nativeInvoke('list_connections') : [...memory]; },
  async create(input) { if (nativeInvoke) return nativeInvoke('create_connection', { input }); const item = { ...input, id: crypto.randomUUID() }; delete item.password; memory.push(item); return item; },
  async update(id, input) { if (nativeInvoke) return nativeInvoke('update_connection', { request: { id, input } }); const item = memory.find((entry) => entry.id === id); Object.assign(item, input); delete item.password; return item; },
  async remove(id) { if (nativeInvoke) return nativeInvoke('delete_connection', { id }); const index = memory.findIndex((entry) => entry.id === id); if (index >= 0) memory.splice(index, 1); },
  async test(id) { if (nativeInvoke) return nativeInvoke('test_connection', { id }); return { success: true }; },
};
