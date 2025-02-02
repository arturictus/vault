<script>
  import "../app.css";
  import { ButtonGroup, Button, GradientButton } from 'flowbite-svelte';
  let { data, children } = $props();
</script>

<div class="layout">
  <main class="container">
    <ButtonGroup class="*:!ring-primary-700">
      <Button href="/authenticate/log_in">Access</Button>
      <Button href="/">Set up</Button>
      <Button href="/secrets">Secrets</Button>
    </ButtonGroup>
    {@render children()}
  </main>
</div>
