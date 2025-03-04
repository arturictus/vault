<script lang="ts">
  import { goto } from "$app/navigation";
  import MainContent from "$lib/components/MainContent.svelte";
  import SearchBar from "$lib/components/secrets/index/SearchBar.svelte";
  import List from "$lib/components/secrets/index/List.svelte";
  import Detail from "$lib/components/secrets/index/Detail.svelte";
  import { toaster } from "$lib/stores/toaster.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { refreshTrigger } from "$lib/stores/refresh";
  import {type Secret, type Secrets} from "$lib/types";


  const { data }: {data: {secrets: Secrets}} = $props();
  let searchTerm = $state("");
  let filteredData: Secrets = $state(data.secrets);
  let selected: Secret | null = $state(null);
  let selectedId: string | null = $state(null);
  
  // Subscribe to refresh events
  refreshTrigger.subscribe(async () => {
    await loadSecrets();
  });
  
  // Function to reload data
  async function loadSecrets() {
    try {
      const secrets: Secrets = await invoke('get_secrets');
      filteredData = secrets;
      if (selectedId) {
        selected = secrets.find((secret) => secret.id === selectedId) || null;
        if (!selected) selectedId = null;
      }
      // Re-apply search filter if needed
      if (searchTerm) {
        onSearch(searchTerm);
      }
    } catch (e) {
      console.error(e);
      toaster.error("Error loading secrets");
    }
  }

  // Actions
  let createSecret = () => {
    goto("/protected/secrets/create");
  };

  let onSearch = (term: string) => {
    searchTerm = term;
    if (term === "") {
      filteredData = data.secrets;
      return;
    }
    filteredData = filteredData.filter((item) =>
      item.name.toLowerCase().includes(term.toLowerCase()),
    );
  };
  let onSelected = (id: string) => {
    selectedId = id;
    selected = data.secrets.find((secret) => secret.id === id) || null;
  };
</script>

{#snippet header()}
  <SearchBar {onSearch} createAction={createSecret} />
{/snippet}

{#snippet leftColumn()}
  <List secrets={filteredData} onclick={onSelected} {selectedId} />
{/snippet}

{#snippet main()}
  {#if selected}
    <Detail secret={selected} />
  {:else}
    <h3>Select Secret</h3>
  {/if}
{/snippet}

<MainContent {leftColumn} {main} {header} />
