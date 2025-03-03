<script>
  import { on } from "svelte/events";

  let { secrets, onclick, selectedId } = $props();
  let repeat = Array.from({ length: 20 }, (_, i) => i + 1);
  let activeClass =
    "w-full px-4 py-2 font-medium text-left rtl:text-right text-white bg-blue-700 border-b border-gray-200 rounded-t-lg cursor-pointer focus:outline-none dark:bg-gray-800 dark:border-gray-600";
  let inactiveClass =
    "w-full px-4 py-2 font-medium text-left rtl:text-right border-b border-gray-200 cursor-pointer hover:bg-gray-100 hover:text-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-700 focus:text-blue-700 dark:border-gray-600 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-500 dark:focus:text-white";
  let onSelected = (event) => {
    const id = event.currentTarget.dataset.id;
    onclick(id);
    console.log("Selected ID:", id);
  };
</script>

<div
  class="w-full text-sm font-medium text-gray-900 bg-white border border-gray-200 rounded-lg dark:bg-gray-700 dark:border-gray-600 dark:text-white"
>
  <!-- <button
    aria-current="true"
    type="button"
    class="w-full px-4 py-2 font-medium text-left rtl:text-right text-white bg-blue-700 border-b border-gray-200 rounded-t-lg cursor-pointer focus:outline-none dark:bg-gray-800 dark:border-gray-600"
  >
    Profile
    <p>
      Cryptography, or cryptology (from Ancient Greek: κρυπτός, romanized:
      kryptós "hidden, secret"; and γράφειν graphein, "to write", or -λογία
      -logia, "study", respectively[1])
    </p>
  </button> -->

  {#each secrets as secret}
    <button
      id="list-{secret.id}"
      data-id={secret.id}
      onclick={onSelected}
      type="button"
      class={secret.id === selectedId ? activeClass : inactiveClass}
    >
      <h3
        class="text-lg mt-1 font-semibold tracking-tights"
      >
        {secret.name}
      </h3>
      <!-- <b>
        {secret.name}
      </b> -->
      <p class="mt-3 font-light">Description</p>
    </button>
  {/each}
</div>
