<script lang="ts">
  import NavSearch from './NavSearch.svelte';
  import NavItem from './NavItem.svelte';
  import NavDropdown from './NavDropdown.svelte';
  let {mainNavItems = [], navDropdowns = [], footerNavItems = []} = $props();

</script>

<NavSearch />

<ul class="space-y-2">
  {#each mainNavItems as item}
    <NavItem {...item} />
  {/each}
  
  {#each navDropdowns as dropdown}
    <NavDropdown {...dropdown} />
  {/each}
</ul>

<ul class="pt-5 mt-5 space-y-2 border-t border-gray-200 dark:border-gray-700">
  {#each footerNavItems as item}
    <NavItem {...item} />
  {/each}
  <!-- Bottom nav items -->
  <!-- <NavItem 
    label="Docs"
    icon="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z"
    href="#"
  /> -->
  <!-- Add other bottom items -->
</ul>
