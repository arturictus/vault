export function load(payload) {
  console.log(payload);
  return { id: 1, name: 'password', value: '123456', description: 'password' };
}

// import { invoke } from '@tauri-apps/api/core';
// export async function load(id) {
//   console.log(id);
//   return await invoke('get_secret', { id });
// }