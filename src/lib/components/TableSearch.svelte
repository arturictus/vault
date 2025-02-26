<script>
    import SearchBar from "./table/SearchBar.svelte";
    import ActionButtons from "./table/ActionButtons.svelte";
    import TableRow from "./table/TableRow.svelte";
    import Pagination from "./table/Pagination.svelte";

    let products = [
        // ... your products data
    ];
</script>

<!-- <section class="bg-gray-50 dark:bg-gray-900 p-3 sm:p-5"> -->
<div class="mx-auto max-w-screen-xl px-4 lg:px-12">
    <div
        class="bg-white dark:bg-gray-800 relative shadow-md sm:rounded-lg overflow-hidden"
    >
        <div
            class="flex flex-col md:flex-row items-center justify-between space-y-3 md:space-y-0 md:space-x-4 p-4"
        >
            <SearchBar />
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
                        <th scope="col" class="px-4 py-3">Product name</th>
                        <th scope="col" class="px-4 py-3">Category</th>
                        <th scope="col" class="px-4 py-3">Brand</th>
                        <th scope="col" class="px-4 py-3">Description</th>
                        <th scope="col" class="px-4 py-3">Price</th>
                        <th scope="col" class="px-4 py-3">
                            <span class="sr-only">Actions</span>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {#each products as product}
                        <TableRow {product} />
                    {/each}
                </tbody>
            </table>
        </div>
        <Pagination />
    </div>
</div>
<!-- </section> -->
