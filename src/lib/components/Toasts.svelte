<!-- <script>
    import { CloseCircleSolid } from "flowbite-svelte";
    import { Toast } from "flowbite-svelte";
    import { toasts } from "$lib/stores/toast";
</script>

{#each toasts as toast (toast.id)}
    <Toast color="red">
        <svelte:fragment slot="icon">
            <CloseCircleSolid class="w-5 h-5" />
            <span class="sr-only">Error icon</span>
        </svelte:fragment>
        {toast.message}
    </Toast>
{/each} -->
<script lang="ts">
    import { toasts } from '../stores/toast';
</script>

<div class="toast-container">
    {#each toasts as toast (toast.id)}
        <div class="toast" class:toast-{toast.type}>
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
</style>