<script>
    import { goto } from "$app/navigation";
    import AppState from "$lib/AppState.svelte";

    let { data, children } = $props();

    $effect(() => {
        if (!AppState.isAuthenticated()) {
            goto("/account/log_in");
        }
    });
</script>

{@render children()}
