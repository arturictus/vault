<script>
  import Yubikey from "$lib/components/Yubikey.svelte";
  import MainContent from "$lib/components/MainContent.svelte";
</script>

{#snippet main()}
  <Yubikey />
{/snippet}
<MainContent {main} />
