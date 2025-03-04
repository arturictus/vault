<script lang="ts">
  import { Section } from "flowbite-svelte-blocks";
  import {
    Label,
    Input,
    Button,
    Select,
    Textarea,
    Alert,
  } from "flowbite-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { superForm } from "sveltekit-superforms";
  import { z } from "zod";
  import { zod } from "sveltekit-superforms/adapters";
  import { toaster } from "$lib/stores/toaster.svelte";


 

  
  // Destructure form helpers from props
  // Note: all form handling/submission logic is now in the parent component
  const { form, errors, enhance, constraints, submitting } = $props();

  let types = [
    { value: "login", name: "Login" },
    { value: "note", name: "Note" },
    { value: "crypto_key", name: "crypto_key" },
  ];
</script>

<Section name="crudcreateform">
  <h2 class="mb-4 text-xl font-bold text-gray-900 dark:text-white">
    Create secret
  </h2>
  <form use:enhance>
    <div class="grid gap-4 sm:grid-cols-2 sm:gap-6">
      <div class="w-full">
        <Label
          >Type
          <Select
            class="mt-2"
            name="kind"
            items={types}
            bind:value={$form.kind}
            {...$constraints.kind}
          />
        </Label>
        {#if $errors.kind}
          <Alert color="red" class="mt-2">
            <span class="font-medium">{$errors.kind}</span>
          </Alert>
        {/if}
      </div>
      <div class="sm:col-span-2">
        <Label for="name" class="mb-2">Name</Label>
        <Input
          type="text"
          id="name"
          name="name"
          bind:value={$form.name}
          placeholder="Enter secret name"
          {...$constraints.name}
        />
        {#if $errors.name}
          <Alert color="red" class="mt-2">
            <span class="font-medium">{$errors.name}</span>
          </Alert>
        {/if}
      </div>
      <div class="sm:col-span-2">
        <Label for="value" class="mb-2">Value</Label>
        <Textarea
          id="value"
          placeholder="Value here..."
          rows={4}
          name="value"
          bind:value={$form.value}
          {...$constraints.value}
        />
        {#if $errors.value}
          <Alert color="red" class="mt-2">
            <span class="font-medium">{$errors.value}</span>
          </Alert>
        {/if}
      </div>
      <Button type="submit" class="w-32" disabled={$submitting}>
        {#if $submitting}
            <span class="inline-block mr-2">Loading...</span>
        {:else}
            Create
        {/if}
    </Button>
    </div>
  </form>
</Section>
