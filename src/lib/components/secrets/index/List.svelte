<script>
  import { on } from "svelte/events";

  let { secrets, onclick, selectedId } = $props();
  // TODO: clean up classes
  let activeClass =
    "w-full px-4 py-2 font-medium text-left rtl:text-right text-white bg-blue-700 border-b border-gray-200 cursor-pointer focus:outline-none dark:bg-gray-800 dark:border-gray-600";
  let inactiveClass =
    "w-full px-4 py-2 font-medium text-left rtl:text-right cursor-pointer hover:bg-gray-100 hover:text-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-700 focus:text-blue-700 dark:border-gray-600 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-500 dark:focus:text-white";
  
  let onSelected = (event) => {
    const id = event.currentTarget.dataset.id;
    onclick(id);
    console.log("Selected ID:", id);
  };

  const colorClasses = {
    alternative:
      "text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 dark:bg-gray-800 dark:text-gray-400 hover:text-primary-700 focus-within:text-primary-700 dark:focus-within:text-white dark:hover:text-white dark:hover:bg-gray-700",
    blue: "text-white bg-blue-700 hover:bg-blue-800 dark:bg-blue-600 dark:hover:bg-blue-700",
    dark: "text-white bg-gray-800 hover:bg-gray-900 dark:bg-gray-800 dark:hover:bg-gray-700",
    green:
      "text-white bg-green-700 hover:bg-green-800 dark:bg-green-600 dark:hover:bg-green-700",
    light:
      "text-gray-900 bg-white border border-gray-300 hover:bg-gray-100 dark:bg-gray-800 dark:text-white dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:border-gray-600",
    primary:
      "text-white bg-primary-700 hover:bg-primary-800 dark:bg-primary-600 dark:hover:bg-primary-700",
    purple:
      "text-white bg-purple-700 hover:bg-purple-800 dark:bg-purple-600 dark:hover:bg-purple-700",
    red: "text-white bg-red-700 hover:bg-red-800 dark:bg-red-600 dark:hover:bg-red-700",
    yellow: "text-white bg-yellow-400 hover:bg-yellow-500 ",
    none: "",
  };

  let getClass = (active = false) => {
    if (active) return colorClasses.primary + " " + activeClass;
    return colorClasses.alternative + " " + inactiveClass;
  };
</script>

{#snippet button({ selectedId = null, secret })}
  <button
    id="list-{secret.id}"
    data-id={secret.id}
    onclick={onSelected}
    type="button"
    class={getClass(selectedId === secret.id)}
  >
    <h3 class="text-lg mt-1 font-semibold tracking-tights">
      {secret.name}
    </h3>
    <p class="mt-3 font-light">Description</p>
  </button>
{/snippet}
<div
  class="w-full text-sm font-medium text-gray-900 bg-white border border-gray-200 rounded-lg dark:bg-gray-700 dark:border-gray-600 dark:text-white"
>
  {#each secrets as secret}
    {@render button({ selectedId, secret })}
  {/each}
</div>
