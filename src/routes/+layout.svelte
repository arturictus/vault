<script>
  import "../app.css";
  let { data, children } = $props();
</script>

<div class="layout">
  <main class="container">
    {@render children()}
  </main>
</div>
