<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    TableSearch,
    Button,
    Dropdown,
    Checkbox,
    ButtonGroup,
  } from "flowbite-svelte";
  import { Section } from "flowbite-svelte-blocks";
  import {
    PlusOutline,
    ChevronDownOutline,
    FilterSolid,
    ChevronRightOutline,
    ChevronLeftOutline,
  } from "flowbite-svelte-icons";

  let divClass =
    "bg-white dark:bg-gray-800 relative shadow-md sm:rounded-lg overflow-hidden";
  let innerDivClass =
    "flex flex-col md:flex-row items-center justify-between space-y-3 md:space-y-0 md:space-x-4 p-4";
  let searchClass = "w-full md:w-1/2 relative";
  let svgDivClass =
    "absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none";
  let classInput =
    "text-gray-900 text-sm rounded-lg focus:ring-primary-500 focus:border-primary-500 block w-full p-2  pl-10";

  const { data } = $props();
  const paginationData = data.secrets;
  console.log(paginationData);
  let searchTerm = $state("");
  let currentPosition = $state(0);
  const itemsPerPage = $state(10);
  const showPage = $state(5);
  let totalPages = $state(0);
  let pagesToShow = $state([]);
  let totalItems = paginationData.length;
  let startPage = $state(null);
  let endPage = $state(null);

 

  // Function to update the pagination and filter the data
  const updateDataAndPagination = () => {
    const currentPageItems = paginationData.slice(
      currentPosition,
      currentPosition + itemsPerPage,
    );
    renderPagination(currentPageItems.length);
  };

  // Load next page of data
  const loadNextPage = () => {
    if (currentPosition + itemsPerPage < paginationData.length) {
      currentPosition += itemsPerPage;
      updateDataAndPagination();
    }
  };

  // Load previous page of data
  const loadPreviousPage = () => {
    if (currentPosition - itemsPerPage >= 0) {
      currentPosition -= itemsPerPage;
      updateDataAndPagination();
    }
  };

  // Render the pagination based on the current state
  const renderPagination = (totalItems) => {
    totalPages = Math.ceil(paginationData.length / itemsPerPage);
    const currentPage = Math.ceil((currentPosition + 1) / itemsPerPage);

    startPage = currentPage - Math.floor(showPage / 2);
    startPage = Math.max(1, startPage);
    endPage = Math.min(startPage + showPage - 1, totalPages);

    pagesToShow = Array.from(
      { length: endPage - startPage + 1 },
      (_, i) => startPage + i,
    );
  };

  // Go to a specific page
  const goToPage = (pageNumber) => {
    currentPosition = (pageNumber - 1) * itemsPerPage;
    updateDataAndPagination();
  };

  // Reactive statements for ranges
  let startRange = $derived(currentPosition + 1);
  let endRange = $derived(Math.min(currentPosition + itemsPerPage, totalItems));

  onMount(() => {
    renderPagination(paginationData.length);
  });

  let currentPageItems = $derived(
    paginationData.slice(currentPosition, currentPosition + itemsPerPage),
  );
  let filteredItems = $derived(
    paginationData.filter((item) =>
      item.name.toLowerCase().includes(searchTerm.toLowerCase()),
    ),
  );
</script>

<Section
  name="advancedTable"
  classSection="bg-gray-50 dark:bg-gray-900 p-3 sm:p-5"
>
  <TableSearch
    placeholder="Search"
    hoverable
    bind:inputValue={searchTerm}
    class={divClass}
    innerClass={innerDivClass}
    {searchClass}
    inputClass={classInput}
  >
    <div
      slot="header"
      class="w-full md:w-auto flex flex-col md:flex-row space-y-2 md:space-y-0 items-stretch md:items-center justify-end md:space-x-3 flex-shrink-0"
    >
      <Button color="primary" on:click={() => goto("/secrets/create")}>
        <PlusOutline class="h-3.5 w-3.5 mr-2" />Add
      </Button>
      <Button color="alternative"
        >Actions<ChevronDownOutline class="w-3 h-3 ml-2" /></Button
      >
    </div>

    <TableHead>
      <TableHeadCell padding="px-4 py-3" scope="col">Name</TableHeadCell>
      <TableHeadCell padding="px-4 py-3" scope="col">Type</TableHeadCell>
      <TableHeadCell padding="px-4 py-3" scope="col">Description</TableHeadCell>
      <TableHeadCell padding="px-4 py-3" scope="col">security</TableHeadCell>
    </TableHead>
    <TableBody class="divide-y">
      {#if searchTerm}
        {#each filteredItems as item (item.id)}
          <TableBodyRow on:click={() => goto(`/secrets/${item.id}`)}>
            <TableBodyCell tdClass="px-4 py-3">{item.name}</TableBodyCell>
            <TableBodyCell tdClass="px-4 py-3">{item.kind}</TableBodyCell>
            <TableBodyCell tdClass="px-4 py-3">{item.description}</TableBodyCell
            >
            <TableBodyCell tdClass="px-4 py-3">{item.security}</TableBodyCell>
          </TableBodyRow>
        {/each}
      {:else}
        {#each currentPageItems as item (item.id)}
          <TableBodyRow on:click={() => goto(`/secrets/${item.id}`)}>
            <TableBodyCell tdClass="px-4 py-3">{item.name}</TableBodyCell>
            <TableBodyCell tdClass="px-4 py-3">{item.kind}</TableBodyCell>
            <TableBodyCell tdClass="px-4 py-3">{item.description}</TableBodyCell
            >
            <TableBodyCell tdClass="px-4 py-3">{item.security}</TableBodyCell>
          </TableBodyRow>
        {/each}
      {/if}
    </TableBody>

    <div
      slot="footer"
      class="flex flex-col md:flex-row justify-between items-start md:items-center space-y-3 md:space-y-0 p-4"
      aria-label="Table navigation"
    >
      <span class="text-sm font-normal text-gray-500 dark:text-gray-400">
        Showing
        <span class="font-semibold text-gray-900 dark:text-white"
          >{startRange}-{endRange}</span
        >
        of
        <span class="font-semibold text-gray-900 dark:text-white"
          >{totalItems}</span
        >
      </span>
      <ButtonGroup>
        <Button on:click={loadPreviousPage} disabled={currentPosition === 0}>
          <ChevronLeftOutline size="xs" class="m-1.5" />
        </Button>
        {#each pagesToShow as pageNumber}
          <Button on:click={() => goToPage(pageNumber)}>{pageNumber}</Button>
        {/each}
        <Button on:click={loadNextPage} disabled={totalPages === endPage}>
          <ChevronRightOutline size="xs" class="m-1.5" />
        </Button>
      </ButtonGroup>
    </div>
  </TableSearch>
</Section>
