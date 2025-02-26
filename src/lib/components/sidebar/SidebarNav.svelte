<script lang="ts">
  import NavSearch from './NavSearch.svelte';
  import NavItem from './NavItem.svelte';
  import NavDropdown from './NavDropdown.svelte';
  
  const mainNavItems = [
    {
      label: 'Secrets',
      icon: 'M2 10a8 8 0 018-8v8h8a8 8 0 11-16 0z M12 2.252A8.014 8.014 0 0117.748 8H12V2.252z',
      href: '/protected/secrets'
    }
  ];

  const dropdowns: Array<{label: String, icon: String, items: String[]}> = [
    // {
    //   label: 'Pages',
    //   icon: 'M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 6a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 3a1 1 0 100 2h6a1 1 0 100-2H7z',
    //   items: ['Settings', 'Kanban', 'Calendar']
    // },
    // Add other dropdowns...
  ];
</script>

<NavSearch />

<ul class="space-y-2">
  {#each mainNavItems as item}
    <NavItem {...item} />
  {/each}
  
  {#each dropdowns as dropdown}
    <NavDropdown {...dropdown} />
  {/each}
</ul>

<ul class="pt-5 mt-5 space-y-2 border-t border-gray-200 dark:border-gray-700">
  <!-- Bottom nav items -->
  <!-- <NavItem 
    label="Docs"
    icon="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z"
    href="#"
  /> -->
  <NavItem 
    label="Settings"
    icon="M5 4a1 1 0 00-2 0v7.268a2 2 0 000 3.464V16a1 1 0 102 0v-1.268a2 2 0 000-3.464V4zM11 4a1 1 0 10-2 0v1.268a2 2 0 000 3.464V16a1 1 0 102 0V8.732a2 2 0 000-3.464V4zM16 3a1 1 0 011 1v7.268a2 2 0 010 3.464V16a1 1 0 11-2 0v-1.268a2 2 0 010-3.464V4a1 1 0 011-1z"
    href="/protected/settings"
  />
  <!-- Add other bottom items -->
</ul>
