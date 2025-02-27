<script>
    import SearchBar from "./table/SearchBar.svelte";
    import ActionButtons from "./table/ActionButtons.svelte";
    import TableRow from "./table/TableRow.svelte";
    import Pagination from "./table/Pagination.svelte";
    import { onMount } from "svelte";

    let { secrets } = $props();
    console.log("secrets", secrets);
    const paginationData = secrets;
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
    let endRange = $derived(
        Math.min(currentPosition + itemsPerPage, totalItems),
    );

    // $effect(() => {
    //     renderPagination(paginationData.length);
    // });

    let currentPageItems = $derived(
        paginationData.slice(currentPosition, currentPosition + itemsPerPage),
    );
    let filteredItems = $derived(
        paginationData.filter((item) =>
            item.name.toLowerCase().includes(searchTerm.toLowerCase()),
        ),
    );

    let onSearch = (searchTerm) => {
        searchTerm = searchTerm;
        console.log("searchTerm", searchTerm);
        currentPosition = 0;
        updateDataAndPagination();
    };

    let divClass =
        "bg-white dark:bg-gray-800 relative shadow-md sm:rounded-lg overflow-hidden";
    // let innerDivClass =
    //     "flex flex-col md:flex-row items-center justify-between space-y-3 md:space-y-0 md:space-x-4 p-4";
    // let searchClass = "w-full md:w-1/2 relative";
    // let svgDivClass =
    //     "absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none";
    // let classInput =
    //     "text-gray-900 text-sm rounded-lg focus:ring-primary-500 focus:border-primary-500 block w-full p-2  pl-10";

    // current classes
    // let divClass = "mx-auto max-w-screen-xl px-4 lg:px-12"
</script>

<!-- <section class="bg-gray-50 dark:bg-gray-900 p-3 sm:p-5"> -->
<div class={divClass}>
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
        <Pagination />
    </div>
</div>
<!-- </section> -->
