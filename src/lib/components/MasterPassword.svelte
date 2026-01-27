<script lang="ts">
  import {
    Label,
    Input,
    A,
    Button,
    Card,
    Alert,
    Spinner,
  } from "flowbite-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AppState from "$lib/AppState.svelte";
  import { goto } from "$app/navigation";
  import { toaster } from "$lib/stores/toaster.svelte";

  // Page configuration
  let title = "Access your safe zone";
  let site = {
    name: "Vault",
    img: "/svelte.svg",
    link: "/",
    imgAlt: "FlowBite Logo",
  };
  let loginTitle = "Access";
  let siteLinkClass =
    "flex items-center justify-center mb-8 text-2xl font-semibold lg:mb-10 dark:text-white";
  let siteImgClass = "mr-4 h-11";
  let cardH1Class = "text-2xl font-bold text-gray-900 dark:text-white";
  let mainDivClass =
    "flex flex-col items-center justify-center px-6 pt-8 mx-auto md:h-screen pt:mt-0 dark:bg-gray-900";

  // Initialize the form
  let username = $state("");
  let password = $state("");
  let isSubmitting = $state(false);
  let errorMessage = $state("");

  // Handle form submission
  async function handleSubmit(event: { preventDefault: () => void }) {
    event.preventDefault();

    errorMessage = "";
    isSubmitting = true;
    try {
      await AppState.login(username, password);
      // Navigate to the secrets page
      goto("/protected/secrets");
    } catch (error: any) {
      console.error(error);
      const errStr = JSON.stringify(error);
      if (errStr.includes("Account locked")) {
        errorMessage = "Account is locked due to too many failed attempts";
      } else {
        errorMessage = "Invalid credentials";
      }
      toaster.error(errorMessage);
      password = "";
    } finally {
      isSubmitting = false;
    }
  }
</script>

<div class={mainDivClass}>
  <a href={site.link} class={siteLinkClass}>
    <img src={site.img} class={siteImgClass} alt={site.imgAlt} />
    <span>{site.name}</span>
  </a>
  <!-- Card -->
  <Card class="w-full" size="md">
    <h1 class={cardH1Class}>
      {title}
    </h1>
    <form class="mt-8 space-y-6" onsubmit={handleSubmit}>
      <div>
        <Label for="username" class="mb-2 dark:text-white">Username</Label>
        <Input
          bind:value={username}
          type="text"
          name="username"
          id="username"
          placeholder="username"
          required
          class="border outline-none dark:border-gray-600 dark:bg-gray-700"
        />
      </div>
      <div>
        <Label for="password" class="mb-2 dark:text-white">Password</Label>
        <Input
          bind:value={password}
          type="password"
          name="password"
          id="password"
          placeholder="••••••••••••••••"
          required
          class="border outline-none dark:border-gray-600 dark:bg-gray-700"
          color={errorMessage ? "red" : undefined}
          aria-invalid={errorMessage ? "true" : undefined}
        />
      </div>
      <Button type="submit" size="lg" disabled={isSubmitting}>
        {#if isSubmitting}
          <Spinner class="mr-3" size="4" color="primary" />Loading...
        {:else}
          {loginTitle}
        {/if}
      </Button>

      <div class="text-sm font-medium text-gray-500 dark:text-gray-400">
        First time? <A href="/account/setup">Register</A>
      </div>
    </form>
  </Card>
</div>
