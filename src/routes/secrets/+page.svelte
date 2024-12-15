<script>
  let { data } = $props();
</script>

<h1>All your secrets are here</h1>
<div class="overflow-x-auto">
  <table class="table table-zebra">
    <thead>
      <tr>
        <th></th>
        <th>Name</th>
        <th>Job</th>
      </tr>
    </thead>
    <tbody>
      {#each data.secrets as { id, name, description }}
        <tr>
          <th><a href="/secrets/{id}">{id}</a></th>
          <td>{name}</td>
          <td>{description}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
