<script>
  import "../app.css";
  import { ButtonGroup, Button, GradientButton } from "flowbite-svelte";
  import AppState, { initPromise } from "$lib/AppState.svelte";
    import MasterPassword from "$lib/components/MasterPassword.svelte";

  // Wait for AppState to initialize
  const ready = initPromise.then(() => true);
  let { data, children } = $props();
</script>

<main>
  <div class="layout">
    {#await AppState}
      <div>Loading...</div>
    {:then}
      {#if AppState.isAuthenticated()}
        {@render children()}
      {:else}
        <MasterPassword />
      {/if}
    {/await}
    <ButtonGroup class="*:!ring-primary-700">
      <Button href="/authenticate/log_in">Access</Button>
      <Button href="/">Start page</Button>
      <Button href="/secrets">Secrets</Button>
      <Button href="/first_setup">First setup</Button>
      <Button href="/settings/new_pk">New PK</Button>
      <Button href="/settings">Settings</Button>
    </ButtonGroup>
  </div>
</main>
