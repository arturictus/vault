<script lang="ts">
    import type { HTMLAnchorAttributes } from 'svelte/elements';
    import { twMerge } from 'tailwind-merge';
  
    interface $$Props extends HTMLAnchorAttributes {
      aClass?: string;
      href?: string;
      label?: string;
      activeClass?: string;
      active?: boolean;
    }
  
    // In Svelte 5, exported variables are implicitly reactive with $state
    export let aClass = 'flex items-center p-2 ps-11 w-full text-base font-normal text-gray-900 rounded-lg transition duration-75 group hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700';
    export let href = '';
    export let label = '';
    export let activeClass = 'flex items-center p-2 ps-11 text-base font-normal text-gray-900 bg-gray-200 dark:bg-gray-700 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700';
    export let active = false;
  </script>
  
  <li>
    <a 
      {...$$restProps} 
      {href} 
      on:blur 
      on:click 
      on:focus 
      on:keydown 
      on:keypress 
      on:keyup 
      on:mouseenter 
      on:mouseleave 
      on:mouseover 
      class={twMerge(active ? activeClass : aClass, $$props.class)}
    >
      {label}
    </a>
  </li>
  
  <!--
  @component
  [Go to docs](https://flowbite-svelte.com/)
  ## Props
  @prop export let aClass = 'flex items-center p-2 ps-11 w-full text-base font-normal text-gray-900 rounded-lg transition duration-75 group hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700';
  @prop export let href = '';
  @prop export let label = '';
  @prop export let activeClass = 'flex items-center p-2 ps-11 text-base font-normal text-gray-900 bg-gray-200 dark:bg-gray-700 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700';
  @prop export let active = false;
  -->