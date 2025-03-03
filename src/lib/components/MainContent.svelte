<script>
  let { main, leftColumn = null, header = null} = $props();
</script>

<main class="md:ml-64 h-auto max-h-full mt-16 min-h-screen">
  {#if header}
    <div class="mb-4">
      {@render header()}
    </div>
  {/if}
  {#if leftColumn}
    <div class="grid grid-cols-3 gap-4 mb-4">
      <div class="overflow-y-auto rounded-lg border-gray-300 dark:border-gray-600 max-h-screen">
        {@render leftColumn()}
      </div>
      <div class="rounded-lg border-gray-300 dark:border-gray-600 col-span-2 h-48 md:h-72">
        {#if main}
          {@render main()}
        {/if}
      </div>
    </div>
  {:else if main}
    <div class="rounded-lg border-gray-300 dark:border-gray-600 h-96 mb-4">
      {@render main()}
    </div>
  {/if}
  <!-- <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
    <div class="border-2 border-dashed border-gray-300 rounded-lg dark:border-gray-600 h-32 md:h-64"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-32 md:h-64"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-32 md:h-64"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-32 md:h-64"></div>
  </div> -->
  <!-- <div class="grid grid-cols-2 gap-4 mb-4">
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
  </div>
  <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-96 mb-4"></div>
  <div class="grid grid-cols-2 gap-4">
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
    <div class="border-2 border-dashed rounded-lg border-gray-300 dark:border-gray-600 h-48 md:h-72"></div>
  </div> -->
</main>
