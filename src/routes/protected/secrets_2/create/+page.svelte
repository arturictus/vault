<script lang="ts">
    import { Section } from "flowbite-svelte-blocks";
    import { Label, Input, Button, Select, Textarea } from "flowbite-svelte";
    import { invoke } from '@tauri-apps/api/core';

    let name = $state("");
    let value = $state("");
    let kind = $state("");
    let error = $state("")
    
    const handleSubmit = async (event) => {
        event.preventDefault();
        const data = {
            name: name,
            value: value,
            kind: kind,
        };
        console.log(data);
        try {
            const response = await invoke('create_secret', {
                data: data
            });
            console.log(response);
        } catch (error) {
            console.error(error);
        }
    };
    let types = [
        { value: "login", name: "Login" },
        { value: "note", name: "Note" },
        { value: "crypto_key", name: "crypto_key" },
    ];
</script>
{#if error != ""}
<pre>{JSON.stringify(error)}</pre>
{/if}
<Section name="crudcreateform">
   
    
    <h2 class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        Create secret
    </h2>
    <form onsubmit={handleSubmit}>
        <div class="grid gap-4 sm:grid-cols-2 sm:gap-6">
            <div class="w-full">
                <Label
                    >Type
                    <Select
                        class="mt-2"
                        items={types}
                        bind:value={kind}
                        required
                    />
                </Label>
            </div>
            <div class="sm:col-span-2">
                <Label for="name" class="mb-2">Name</Label>
                <Input
                    type="text"
                    id="name"
                    bind:value={name}
                    placeholder="Type product name"
                    required
                />
            </div>
            <div class="sm:col-span-2">
                <Label for="description" class="mb-2">Value</Label>
                <Textarea
                    id="value"
                    placeholder="Value here..."
                    rows="4"
                    name="value"
                    bind:value={value}
                    required
                />
            </div>
            <Button type="submit" class="w-32">Add product</Button>
        </div>
    </form>
</Section>
