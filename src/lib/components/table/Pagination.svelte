<script>
    let {currentPage, totalItems, itemsPerPage} = $props();
</script>

<nav class="flex flex-col md:flex-row justify-between items-start md:items-center space-y-3 md:space-y-0 p-4" aria-label="Table navigation">
    <span class="text-sm font-normal text-gray-500 dark:text-gray-400">
        Showing
        <span class="font-semibold text-gray-900 dark:text-white">{(currentPage - 1) * itemsPerPage + 1}-{Math.min(currentPage * itemsPerPage, totalItems)}</span>
        of
        <span class="font-semibold text-gray-900 dark:text-white">{totalItems}</span>
    </span>
    <!-- ...existing pagination buttons code... -->
</nav>
