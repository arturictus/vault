<script>
   let { secret } = $props();
</script>

<tr class="border-b dark:border-gray-700">
    <th scope="row" class="px-4 py-3 font-medium text-gray-900 whitespace-nowrap dark:text-white">{secret.name}</th>
    <td class="px-4 py-3">{secret.type}</td>
    <td class="px-4 py-3">{secret.description}</td>
    <td class="px-4 py-3">{secret.value}</td>
    <td class="px-4 py-3 flex items-center justify-end">
        <button id="{secret.id}-dropdown-button" data-dropdown-toggle="{secret.id}-dropdown" class="inline-flex items-center p-0.5 text-sm font-medium text-center text-gray-500 hover:text-gray-800 rounded-lg focus:outline-none dark:text-gray-400 dark:hover:text-gray-100" type="button">
            <svg class="w-5 h-5" aria-hidden="true" fill="currentColor" viewbox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                <path d="M6 10a2 2 0 11-4 0 2 2 0 014 0zM12 10a2 2 0 11-4 0 2 2 0 014 0zM16 12a2 2 0 100-4 2 2 0 000 4z" />
            </svg>
        </button>
        <!-- ...existing dropdown menu code... -->
    </td>
</tr>
