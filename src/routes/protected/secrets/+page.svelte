<script>
  // TODO: Implement TS
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import MainContent from "$lib/components/MainContent.svelte";
  import SearchBar from "$lib/components/secrets_index/SearchBar.svelte";
  import List from "$lib/components/secrets_index/List.svelte";
  import Detail from "$lib/components/secrets_index/Detail.svelte";

  const { data } = $props();
  let searchTerm = $state("");
  let filteredData = $state(data.secrets);
  let selected = $state(null);
  let selectedId = $state(null);

  // Actions
  let createSecret = () => {
    goto("/protected/secrets/create");
  };

  let handleInput = (event) => {
    console.log(event.target.value);
  };

  let onSearch = (term) => {
    console.log(term);
    searchTerm = term;
    if (term === "") {
      filteredData = data.secrets;
      return;
    }
    filteredData = filteredData.filter((item) =>
      item.name.toLowerCase().includes(term.toLowerCase()),
    );
  };
  let onSelected = (id) => {
    selectedId = id;
    selected = data.secrets.find((secret) => secret.id === id);
  };
</script>

{#snippet header()}
  <SearchBar {onSearch} createAction={createSecret} />
{/snippet}

{#snippet leftColumn()}
  <List secrets={filteredData} onclick={onSelected} {selectedId}/>
{/snippet}

{#snippet main()}
  {#if selected}
    <Detail secret={selected} />
  {:else}
    <h3>Select Secret</h3>
  {/if}
{/snippet}

<MainContent {leftColumn} {main} {header} />

<!-- {#snippet singleBlock()}
  <TableSearch secrets={data.secrets} />
{/snippet} -->
<!-- <MainContent main={singleBlock}/> -->
