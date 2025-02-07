<script lang="ts">
    import { Section } from "flowbite-svelte-blocks";
    import { Label, Input, Button, Select, Textarea } from "flowbite-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";

    let name = $state("");
    const handleSubmit = async (event: { preventDefault: () => void; }) => {
        event.preventDefault();
        console.log(name);
        // await invoke("create_new_rsa_key", { name });
        goto("/settings");
    };
</script>

<Section name="crudcreateform">
    <h2 class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        Create new RSA Key
    </h2>
    <form onsubmit={handleSubmit}>
        <div class="grid gap-4 sm:grid-cols-2 sm:gap-6">
            <div class="sm:col-span-2">
                <Label for="name" class="mb-2">Name</Label>
                <Input type="text" id="name" bind:value={name} />
            </div>
            <Button type="submit" class="w-32">Done</Button>
        </div>
    </form>
</Section>
