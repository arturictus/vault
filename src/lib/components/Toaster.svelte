<script>
    // import { CloseCircleSolid } from "flowbite-svelte";
    // import { Toast } from "flowbite-svelte";
    import { getToastState } from '$lib/stores/toaster-state.svelte';
    let toastState = getToastState();
    console.log(toasts);
</script>

<div class="absolute top-2 right-2 flex flex-col">
    {#each toastState.toasts as toast (toast.id)}
        <div class="toast">
            {toast.message}
        </div>
    {/each}
</div>

<!-- <script lang="ts">
    // TODO: not working
    import { getToasts } from '$lib/stores/toast.svelte';
    let toasts = getToasts();
</script>

<div class="toast-container">
    {#each toasts as toast (toast.id)}
        <div class="toast">
            {toast.message}
        </div>
    {/each}
</div>

<style>
    .toast-container {
        position: fixed;
        bottom: 1rem;
        right: 1rem;
        z-index: 1000;
    }

    .toast {
        padding: 1rem;
        margin: 0.5rem;
        border-radius: 0.25rem;
        background: white;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        min-width: 200px;
    }

    .toast-success { background: #4CAF50; color: white; }
    .toast-error { background: #F44336; color: white; }
    .toast-info { background: #2196F3; color: white; }
    .toast-warning { background: #FF9800; color: white; }
</style> -->
