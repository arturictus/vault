<script>
  import "../app.css";
  import { ButtonGroup, Button, GradientButton } from "flowbite-svelte";
  import AppState, { initPromise } from "$lib/AppState.svelte";
  import MasterPassword from "$lib/components/MasterPassword.svelte";
  import FirstSetup from "$lib/components/FirstSetup.svelte";

  // Wait for AppState to initialize
  const ready = initPromise.then(() => true);
  let { data, children } = $props();
  let authenticated = true || AppState.isAuthenticated();
  let setup_page = $state(false);
</script>
{#await AppState}
  <div>Loading...</div>
{:then}
  {#if authenticated}
    <main>
      {@render children()}
      <ButtonGroup class="*:!ring-primary-700">
        <Button href="/authenticate/log_in">Access</Button>
        <Button href="/">Start page</Button>
        <Button href="/secrets">Secrets</Button>
        <Button href="/first_setup">First setup</Button>
        <Button href="/settings/new_pk">New PK</Button>
        <Button href="/settings">Settings</Button>
      </ButtonGroup>
    </main>
  {:else}
    {#if setup_page}
    <button onclick={() => {setup_page = false}}>Back</button>
      <FirstSetup />

    {:else}
      <MasterPassword setupFn={() => {setup_page = true}}/>
    {/if}
  {/if}
{/await}
