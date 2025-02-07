<script>
  import "../app.css";
  import { ButtonGroup, Button, GradientButton } from "flowbite-svelte";
  let { data, children } = $props();
</script>

<main>
  <div class="layout">
    <ButtonGroup class="*:!ring-primary-700">
      <Button href="/authenticate/log_in">Access</Button>
      <Button href="/">Start page</Button>
      <Button href="/secrets">Secrets</Button>
      <Button href="/first_setup">First setup</Button>
      <Button href="/settings/new_pk">New PK</Button>
      <Button href="/settings">Settings</Button>
    </ButtonGroup>
    {@render children()}
  </div>
</main>
