import { Client, Stronghold } from "@tauri-apps/plugin-stronghold";
// when using `"withGlobalTauri": true`, you may use
// const { Client, Stronghold } = window.__TAURI__.stronghold;
import { appDataDir } from "@tauri-apps/api/path";
// when using `"withGlobalTauri": true`, you may use
// const { appDataDir } = window.__TAURI__.path;

const initStronghold = async () => {
  const vaultPath = `${await appDataDir()}/vault.hold`;
  const vaultPassword = "vault password";
  const stronghold = await Stronghold.load(vaultPath, vaultPassword);

  let client: Client;
  const clientName = "name your client";
  try {
    client = await stronghold.loadClient(clientName);
  } catch {
    client = await stronghold.createClient(clientName);
  }

  return {
    stronghold,
    client,
  };
};

// Insert a record to the store
async function insertRecord(store: any, key: string, value: string) {
  const data = Array.from(new TextEncoder().encode(value));
  await store.insert(key, data);
}

// Read a record from store
async function getRecord(store: any, key: string): Promise<string> {
  const data = await store.get(key);
  return new TextDecoder().decode(new Uint8Array(data));
}

export default async () => {
  const { stronghold, client } = await initStronghold();
  const store = client.getStore();
  return {
    stronghold,
    client,
    store,
    insert: async (key: string, value: string) => {
      const result = await insertRecord(store, key, value);
      await stronghold.save();
      return result;
    },
    get: async (key: string) => await getRecord(store, key),
    remove: async (key: string) => {
      const result = await store.remove(key);
      await stronghold.save();
      return result;
    },
  };
};
