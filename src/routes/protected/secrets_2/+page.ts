import { invoke } from '@tauri-apps/api/core';
export async function load() {
  try {
    const secrets = await invoke('get_secrets');
    return { secrets };
  } catch (e) { 
    console.error(e); 
    return {
      secrets: []
    };
  }
  
}
