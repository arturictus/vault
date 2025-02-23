<script lang="ts">
    import Wrapper from "$lib/utils/Wrapper.svelte";
    import { twMerge } from "tailwind-merge";
    import { getContext } from "svelte";
    type DropdownType = {
        activeClass: string;
    };
    import type {
        HTMLButtonAttributes,
        HTMLAnchorAttributes,
    } from "svelte/elements";

    interface DropdownItemProps {
        defaultClass?: string;
        href?: string;
        activeClass?: string;
    }

    type Props = DropdownItemProps &
        (HTMLAnchorAttributes | HTMLButtonAttributes);

    let defaultClass = "font-medium py-2 px-4 text-sm hover:bg-gray-100 dark:hover:bg-gray-600";
    let href = undefined;
    let activeClass = undefined;

    const context = getContext<DropdownType>("DropdownType") ?? {};
    const activeUrlStore = getContext("activeUrl") as {
        subscribe: (callback: (value: string) => void) => void;
    };

    let { children, data, ...restProps } = $props();

    // Using $state for reactive variables
    let sidebarUrl = $state("");
    let active = $state(false);
    let liClass = $state("");
    let wrap = $state(true);

    // Handle store subscription with $effect
    $effect(() => {
        const unsubscribe = activeUrlStore.subscribe((value) => {
            sidebarUrl = value;
        });
        return unsubscribe; // Cleanup subscription
    });

    // Reactive updates using $effect
    $effect(() => {
        active = sidebarUrl ? href === sidebarUrl : false;
        liClass = twMerge(
            defaultClass,
            href ? "block" : "w-full text-left",
            active && (activeClass ?? context.activeClass),
            data.class,
        );
    });

    function init(node: HTMLElement) {
        wrap = node.parentElement?.tagName === "UL";
    }
</script>

<Wrapper tag="li" show={wrap} use={init}>
    <svelte:element
        this={href ? "a" : "button"}
        {href}
        type={href ? undefined : "button"}
        role={href ? "link" : "button"}
        {...restProps}
        class={liClass}
        on:click
        on:change
        on:keydown
        on:keyup
        on:focus
        on:blur
        on:mouseenter
        on:mouseleave
    >
        <slot />
    </svelte:element>
</Wrapper>

<!--
  @component
  [Go to docs](https://flowbite-svelte.com/)
  ## Props
  @prop export let defaultClass = 'font-medium py-2 px-4 text-sm hover:bg-gray-100 dark:hover:bg-gray-600';
  @prop export let href = undefined;
  @prop export let activeClass = undefined;
  -->
