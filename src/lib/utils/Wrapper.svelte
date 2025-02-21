<script lang="ts">
    import type { Action } from 'svelte/action';
  
    // Exported props are automatically reactive in Svelte 5
    export let tag = 'div';
    export let show = false;
    /* eslint-disable @typescript-eslint/no-explicit-any */
    export let use: Action<HTMLElement, any> = () => {};
  </script>
  
  {#if show}
    <svelte:element this={tag} use:use {...$$restProps}><slot /></svelte:element>
  {:else}
    <slot />
  {/if}
  
  <!--
  @component
  [Go to docs](https://flowbite-svelte.com/)
  ## Props
  @prop export let tag = 'div';
  @prop export let show = false;
  @prop export let use: Action<HTMLElement, any> = () => {};
  -->