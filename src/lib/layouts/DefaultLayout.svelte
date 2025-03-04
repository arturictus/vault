<script lang="ts">
  import Navbar from "$lib/components/Navbar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import MainContent from "$lib/components/MainContent.svelte";
  import Toaster from "$lib/components/Toaster.svelte";
  const mainNavItems = [
    {
      label: "Secrets",
      icon: "M2 10a8 8 0 018-8v8h8a8 8 0 11-16 0z M12 2.252A8.014 8.014 0 0117.748 8H12V2.252z",
      href: "/protected/secrets",
    },
  ];

  const navDropdowns: Array<{ label: string; icon: string; items: string[] }> =
    [
      // {
      //   label: 'Pages',
      //   icon: 'M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 6a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 3a1 1 0 100 2h6a1 1 0 100-2H7z',
      //   items: ['Settings', 'Kanban', 'Calendar']
      // },
      // Add other dropdowns...
    ];

  const footerNavItems = [
    {
      label: "Settings",
      icon: "M5 4a1 1 0 00-2 0v7.268a2 2 0 000 3.464V16a1 1 0 102 0v-1.268a2 2 0 000-3.464V4zM11 4a1 1 0 10-2 0v1.268a2 2 0 000 3.464V16a1 1 0 102 0V8.732a2 2 0 000-3.464V4zM16 3a1 1 0 011 1v7.268a2 2 0 010 3.464V16a1 1 0 11-2 0v-1.268a2 2 0 010-3.464V4a1 1 0 011-1z",
      href: "/protected/settings",
    },
  ];
  // let {children} = $props();
</script>

<div class="antialiased bg-gray-50 dark:bg-gray-900">
  <Toaster position="top-right" maxToasts={5} />
  <Navbar />
  <Sidebar {mainNavItems} {navDropdowns} {footerNavItems} />
  <slot />
</div>
