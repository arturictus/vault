
<script>
  // TODO: Implement TS
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import TableSearch from "$lib/components/TableSearch.svelte";
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
  
</script>

<TableSearch secrets={data.secrets}/>