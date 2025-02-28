<script>
  import "../app.css";
  import { ButtonGroup, Button, GradientButton } from "flowbite-svelte";
  import AppState, { initPromise } from "$lib/AppState.svelte";
  import MasterPassword from "$lib/components/MasterPassword.svelte";
  import FirstSetup from "$lib/components/FirstSetup.svelte";
  import { goto } from "$app/navigation";

  // Wait for AppState to initialize
  const ready = initPromise.then(() => true);
  let { data, children } = $props();
  let authenticated = AppState.isAuthenticated();
  let setup_page = $state(false);
</script>

<main class="bg-gray-50 dark:bg-gray-900">
  {#await AppState}
    <div>Loading...</div>
  {:then}
    {@render children()}
  {/await}
</main>
