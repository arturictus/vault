<script>
  import Navbar from "$lib/components/Navbar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import MainContent from "$lib/components/MainContent.svelte";
  import Toaster from "$lib/components/Toaster.svelte";
</script>

<div class="antialiased bg-gray-50 dark:bg-gray-900">
  <Toaster position="top-right" maxToasts={5} />
  <Navbar />
  <Sidebar />
  <slot />
</div>
