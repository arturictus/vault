<script lang="ts">
  const languages = [
    { code: 'us', label: 'English (US)', flag: '...' },
    { code: 'de', label: 'Deutsch', flag: '...' },
    { code: 'it', label: 'Italiano', flag: '...' },
    { code: 'cn', label: '中文 (繁體)', flag: '...' }
  ];
</script>

<div
  class="hidden absolute bottom-0 left-0 justify-center p-4 space-x-4 w-full lg:flex bg-white dark:bg-gray-800 z-20"
>
  <!-- Settings button -->
  <a
    href="#"
    class="inline-flex justify-center p-2 text-gray-500 rounded cursor-pointer dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-600"
  >
    <svg
      aria-hidden="true"
      class="w-6 h-6"
      fill="currentColor"
      viewBox="0 0 20 20"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M5 4a1 1 0 00-2 0v7.268a2 2 0 000 3.464V16a1 1 0 102 0v-1.268a2 2 0 000-3.464V4zM11 4a1 1 0 10-2 0v1.268a2 2 0 000 3.464V16a1 1 0 102 0V8.732a2 2 0 000-3.464V4zM16 3a1 1 0 011 1v7.268a2 2 0 010 3.464V16a1 1 0 11-2 0v-1.268a2 2 0 010-3.464V4a1 1 0 011-1z"
      ></path>
    </svg>
  </a>

  <!-- Language selector -->
  <button
    type="button"
    data-dropdown-toggle="language-dropdown"
    class="inline-flex justify-center p-2 text-gray-500 rounded cursor-pointer dark:hover:text-white dark:text-gray-400 hover:text-gray-900 hover:bg-gray-100 dark:hover:bg-gray-600"
  >
    <!-- Current language flag -->
  </button>

  <!-- Language dropdown menu -->
  <div
    class="hidden z-50 my-4 text-base list-none bg-white rounded divide-y divide-gray-100 shadow dark:bg-gray-700"
    id="language-dropdown"
  >
    <ul class="py-1" role="none">
      {#each languages as { code, label, flag }}
        <li>
          <!-- Language option -->
        </li>
      {/each}
    </ul>
  </div>
</div>
