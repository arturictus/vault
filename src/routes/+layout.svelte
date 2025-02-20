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
{#await AppState}
  <div>Loading...</div>
{:then}
  <main>
    {@render children()}
    <ButtonGroup class="*:!ring-primary-700">
      <Button href="/account/log_in">Access</Button>
      <Button href="/">Start page</Button>
      <Button href="/protected/secrets">Secrets</Button>
      <Button href="/account/setup">First setup</Button>
      <Button href="/protected/settings/new_pk">New PK</Button>
      <Button href="/protected/settings">Settings</Button>
      <Button href="/protected/secrets/create">Create secret</Button>
    </ButtonGroup>
  </main>
{/await}
