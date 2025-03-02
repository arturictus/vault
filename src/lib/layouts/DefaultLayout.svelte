<script>
  import Navbar from "$lib/components/Navbar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import MainContent from "$lib/components/MainContent.svelte";
  let { children } = $props();
</script>

<div class="antialiased bg-gray-50 dark:bg-gray-900">
  <Navbar />
  <Sidebar />
  <MainContent> 
    {@render children()}
  </MainContent>
</div>
