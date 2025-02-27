<script>
    import SearchBar from "./table/SearchBar.svelte";
    import ActionButtons from "./table/ActionButtons.svelte";
    import TableRow from "./table/TableRow.svelte";
    import Pagination from "./table/Pagination.svelte";
    import { onMount } from "svelte";

    let { secrets } = $props();
    const paginationData = secrets;
    let searchTerm = $state("");
    let currentPosition = $state(0);
    const itemsPerPage = $state(10);
    const showPage = $state(5);
    let totalPages = $state(0);
    let pagesToShow = $state([]);
    let totalItems = paginationData.length;
    let startPage = $state(null);
    let endPage = $state(null);
    let filteredData = $state(paginationData);

    // Function to update the pagination and filter the data
    const updateDataAndPagination = () => {
        const currentPageItems = filteredData.slice(
            currentPosition,
            currentPosition + itemsPerPage,
        );
        renderPagination(filteredData.length);
    };

    // Load next page of data
    const loadNextPage = () => {
        if (currentPosition + itemsPerPage < filteredData.length) {
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
        totalPages = Math.ceil(filteredData.length / itemsPerPage);
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
    let endRange = $derived(
        Math.min(currentPosition + itemsPerPage, totalItems),
    );

    onMount(() => {
        renderPagination(paginationData.length);
    });

    let currentPageItems = $derived(
        filteredData.slice(currentPosition, currentPosition + itemsPerPage),
    );

    let onSearch = (term) => {
        searchTerm = term;
        filteredData = paginationData.filter((item) =>
            item.name.toLowerCase().includes(term.toLowerCase())
        );
        currentPosition = 0;
        updateDataAndPagination();
    };

    let currentPage = $derived(currentPosition + 1);
</script>

<!-- <section class="bg-gray-50 dark:bg-gray-900 p-3 sm:p-5"> -->
<div class="bg-white dark:bg-gray-800 relative shadow-md sm:rounded-lg overflow-hidden">
    <div
        class="bg-white dark:bg-gray-800 relative shadow-md sm:rounded-lg overflow-hidden"
    >
        <div
            class="flex flex-col md:flex-row items-center justify-between space-y-3 md:space-y-0 md:space-x-4 p-4"
        >
            <SearchBar {onSearch}/>
            <ActionButtons />
        </div>
        <div class="overflow-x-auto">
            <table
                class="w-full text-sm text-left text-gray-500 dark:text-gray-400"
            >
                <thead
                    class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400"
                >
                    <tr>
                        <th scope="col" class="px-4 py-3">Name</th>
                        <th scope="col" class="px-4 py-3">Type</th>
                        <th scope="col" class="px-4 py-3">Description</th>
                        <th scope="col" class="px-4 py-3">Value</th>
                        <th scope="col" class="px-4 py-3">
                            <span class="sr-only">Actions</span>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {#each currentPageItems as secret}
                        <TableRow {secret} />
                    {/each}
                </tbody>
            </table>
        </div>
        <Pagination {currentPage} {totalItems} {itemsPerPage}/>
    </div>
</div>
<!-- </section> -->
