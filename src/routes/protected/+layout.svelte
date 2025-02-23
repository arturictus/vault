<script>
    import { goto } from "$app/navigation";
    import AppState from "$lib/AppState.svelte";
    import {
        Navbar,
        NavBrand,
        NavLi,
        NavUl,
        NavHamburger,
        ImagePlaceholder,
        Skeleton,
        TextPlaceholder,
        A
    } from "flowbite-svelte";
    import StickyNavbar from "$lib/components/StickyNavbar.svelte";

    let { data, children } = $props();

    $effect(() => {
        if (!AppState.isAuthenticated()) {
            goto("/account/log_in");
        }
    });
</script>

<StickyNavbar >
    <A href="/sidebar">Sidebar</A>
    {@render children()}
</StickyNavbar>