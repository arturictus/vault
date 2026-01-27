<script lang="ts">
    import { Section } from "flowbite-svelte-blocks";
    import {
        Card,
        Label,
        Input,
        Button,
        Select,
        Textarea,
        A,
        Alert,
    } from "flowbite-svelte";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import appState from "$lib/AppState.svelte";
    import { toaster } from "$lib/stores/toaster.svelte";

    // SuperForms props
    let { form, errors, enhance, submitting, constraints } = $props();

    async function handleSavePassword() {
        try {
            await appState.register($form.username, $form.password);
            toaster.success("Account created successfully!");
            goto("/protected/secrets");
        } catch (e: any) {
            console.error("Error creating account:", e);
            if (JSON.stringify(e).includes("User already exists")) {
                toaster.error("Username already taken");
            } else {
                toaster.error("Error creating account");
            }
        }
    }
</script>

<Section name="crudcreateform">
    <h2 class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        Create Account
    </h2>

    <form use:enhance on:submit|preventDefault={handleSavePassword}>
        <div class="grid gap-4 sm:grid-cols-2 sm:gap-6">
            <div class="sm:col-span-2">
                <Label for="username" class="mb-2">Username</Label>
                <Input
                    type="text"
                    id="username"
                    placeholder="username"
                    bind:value={$form.username}
                    name="username"
                    required
                    {...constraints.username}
                    color={$errors.username ? "red" : undefined}
                />
                {#if $errors.username}
                    <Alert color="red" class="mt-2">
                        <span class="font-medium">{$errors.username}</span>
                    </Alert>
                {/if}
            </div>

            <div class="sm:col-span-2">
                <Label for="password" class="mb-2">Password</Label>
                <Input
                    type="password"
                    id="password"
                    placeholder="••••••••"
                    bind:value={$form.password}
                    name="password"
                    required
                    {...constraints.password}
                    color={$errors.password ? "red" : undefined}
                />
                {#if $errors.password}
                    <Alert color="red" class="mt-2">
                        <span class="font-medium">{$errors.password}</span>
                    </Alert>
                {/if}
            </div>

            <div class="sm:col-span-2">
                <Label for="password_confirmation" class="mb-2"
                    >Confirm Password</Label
                >
                <Input
                    type="password"
                    id="password_confirmation"
                    placeholder="••••••••"
                    bind:value={$form.password_confirmation}
                    name="password_confirmation"
                    required
                    {...constraints.password_confirmation}
                    color={$errors.password_confirmation ? "red" : undefined}
                />
                {#if $errors.password_confirmation}
                    <Alert color="red" class="mt-2">
                        <span class="font-medium"
                            >{$errors.password_confirmation}</span
                        >
                    </Alert>
                {/if}
            </div>

            <Button type="submit" class="w-32" disabled={$submitting}>
                {#if $submitting}
                    <span class="inline-block mr-2">Loading...</span>
                {:else}
                    Sign Up
                {/if}
            </Button>
        </div>
    </form>

    <div class="text-sm font-medium text-gray-500 dark:text-gray-400 mt-5">
        Login instead <A href="/account/log_in">Access</A>
    </div>
</Section>
