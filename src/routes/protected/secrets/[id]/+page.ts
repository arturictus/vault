import { invoke } from '@tauri-apps/api/core';
export async function load({params}) {
  console.log(params);
  return await invoke('get_secret', params);
} 