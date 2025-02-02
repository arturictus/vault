<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { enhance } from "$app/forms";
  import { goto } from "$app/navigation";
  

  let name = $state("");
  let greetMsg = $state("");

  async function access(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    // greetMsg = await invoke("access", { name });

    goto("/secrets");
  }
  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<div class="card mx-auto w-full max-w-5xl shadow-xl">
  <div class="grid md:grid-cols-2 grid-cols-1 bg-base-100 rounded-xl">
    <div class="py-24 px-10">
      <form onsubmit={access}>
        <div class="mb-4">
          <input
            type="password"
            class="input input-bordered w-full"
            placeholder="Password"
            required
          />
        </div>

        <button type="submit" class="btn mt-2 w-full btn-primary">Access</button
        >
      </form>
    </div>
  </div>
</div>

<style>
</style>
