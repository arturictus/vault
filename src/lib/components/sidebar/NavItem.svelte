<script lang="ts">
  export let label: string;
  export let icon: string;
  export let href: string;
  export let badge: string | undefined = undefined;
</script>

<li>
  <a
    {href}
    class="flex items-center p-2 text-base font-medium text-gray-900 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 group"
  >
    <svg
      aria-hidden="true"
      class="w-6 h-6 text-gray-500 transition duration-75 dark:text-gray-400 group-hover:text-gray-900 dark:group-hover:text-white"
      fill="currentColor"
      viewBox="0 0 20 20"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path d={icon}></path>
    </svg>
    <span class="ml-3">{label}</span>
    {#if badge}
      <span class="inline-flex justify-center items-center w-5 h-5 text-xs font-semibold rounded-full text-primary-800 bg-primary-100 dark:bg-primary-200 dark:text-primary-800">
        {badge}
      </span>
    {/if}
  </a>
</li>
