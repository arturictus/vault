import { writable } from "svelte/store";

interface Session {
  user?: boolean;
}
export const session = writable<Session>({ user: false });
