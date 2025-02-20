<script lang="ts">
    import { Section } from "flowbite-svelte-blocks";
    import { Card, Label, Input, Button, Select, Textarea, A } from "flowbite-svelte";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import appState from "$lib/AppState.svelte";
    let password = $state("");
    let password_confirmation = $state("");
    let private_key = $state("");

    const handleSubmit = () => {
        if (password !== password_confirmation) {
            // addToast("Passwords do not match");
            password = "";
            password_confirmation = "";
            return;
        }
        invoke("save_master_password", { password, private_key }).then(async () => {
            await appState.refreshAuthState();
            // addToast("Password saved");
            goto("/protected/secrets");
        }).catch((e) => {
            // addToast("Error saving password");
        });
    };
</script>

<Section name="crudcreateform">
    
    <h2 class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        Initial Settings
    </h2>
    <form onsubmit={handleSubmit}>
        <div class="grid gap-4 sm:grid-cols-2 sm:gap-6">
            <div class="sm:col-span-2">
                <Label for="name" class="mb-2">Main Password</Label>
                <Input
                    type="password"
                    id="password"
                    placeholder="••••••••"
                    bind:value={password}
                    required
                />
            </div>
            <div class="sm:col-span-2">
                <Label for="name" class="mb-2">Confirm Password</Label>
                <Input
                    type="password"
                    id="password_confirmation"
                    placeholder="••••••••"
                    bind:value={password_confirmation}
                    required
                />
            </div>
            <div class="sm:col-span-2">
                <Label for="private_key" class="mb-2">Private Key <small>(optional)</small></Label>
                <Textarea
                    id="private_key"
                    placeholder="-----BEGIN OPENSSH PRIVATE KEY-----
••••••••
••••••••
••••••••
••••••••
-----END OPENSSH PRIVATE KEY-----"
                    rows={6}
                    name="private_key"
                    bind:value={private_key}
                />
            </div>
            <Button type="submit" class="w-32">Let's Go!</Button>
        </div>
    </form>
    <div class="text-sm font-medium text-gray-500 dark:text-gray-400 mt-5">
        Login instead <A href="/account/log_in">Access</A>
    </div>
    
</Section>