<script lang="ts">
    import type { HTMLAttributes } from 'svelte/elements';
    import { twMerge } from 'tailwind-merge';
    
    let { class: className = '', divClass = 'overflow-y-auto py-4 px-3 bg-gray-50 rounded-sm dark:bg-gray-800', children, ...restProps } = $props<{
        class?: string;
        divClass?: string;
    }>();
</script>

<div {...restProps} class={twMerge(divClass, className)}>
    {@render children()}
</div>

<!--
@component
[Go to docs](https://flowbite-svelte.com/)
## Props
@prop divClass - Default: 'overflow-y-auto py-4 px-3 bg-gray-50 rounded-sm dark:bg-gray-800'
@prop class - Default: ''
-->