<script lang="ts">
    import { Section } from "flowbite-svelte-blocks";
    import { Label, Input, Button, Select, Textarea } from "flowbite-svelte";
    const handleSubmit = () => {
        alert("Form submited.");
    };
    let selected;
    let countries = [
        { value: "tv", name: "TV/Monitors" },
        { value: "pc", name: "PC" },
        { value: "phone", name: "Phones" },
    ];
</script>

<Section name="crudcreateform">
    <h2 class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        Initial Settings
    </h2>
    <form on:submit={handleSubmit}>
        <div class="grid gap-4 sm:grid-cols-2 sm:gap-6">
            <div class="sm:col-span-2">
                <Label for="name" class="mb-2">Main Password</Label>
                <Input
                    type="password"
                    id="password"
                    placeholder="••••••••"
                    required
                />
            </div>
            <div class="sm:col-span-2">
                <Label for="name" class="mb-2">Confirm Password</Label>
                <Input
                    type="password"
                    id="password_confirmation"
                    placeholder="••••••••"
                    required
                />
            </div>
            <div class="sm:col-span-2">
                <Label for="description" class="mb-2">Private Key <small>(optional)</small></Label>
                <Textarea
                    id="description"
                    placeholder="-----BEGIN OPENSSH PRIVATE KEY-----
••••••••
••••••••
••••••••
••••••••
-----END OPENSSH PRIVATE KEY-----"
                    rows="4"
                    name="description"
                    required
                />
            </div>
            <Button type="submit" class="w-32">Done</Button>
        </div>
    </form>
</Section>
