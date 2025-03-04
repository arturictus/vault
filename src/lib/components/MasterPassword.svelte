<script lang="ts">
  import { superForm } from 'sveltekit-superforms';
  import {
    Label,
    Input,
    Checkbox,
    A,
    Button,
    Card,
    Toast,
    Alert,
    Spinner
  } from "flowbite-svelte";
  import {
    CheckCircleSolid,
    ExclamationCircleSolid,
    FireOutline,
    CloseCircleSolid,
  } from "flowbite-svelte-icons";
  import { invoke } from "@tauri-apps/api/core";
  import AppState from "$lib/AppState.svelte";
  import { goto } from "$app/navigation";
  import { toaster } from "$lib/stores/toaster.svelte.ts";
  import validators from "$lib/validators";
  
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

  // Initialize the superForm
  const { form, errors, enhance, submitting, message, reset } = superForm(validators.master_password.buildForm({ password: "" }), {
    // Client-side form validation options
    validators: validators.master_password.schema,
    // Enable client-side only mode (Single Page Application mode)
    SPA: true,
    // Custom validation handler
    onUpdate({ form }) {
      // Additional custom validation if needed
      if (form.data.password.includes('1234')) {
        errors.password = 'Please use a stronger password';
      }
    },
    // Handle form submission
    onSubmit: async ({ form, cancel }) => {
      // Prevent default form submission if validation fails
      if (!form.valid) {
        toaster.error("Please fix the form errors");
        cancel();
        return;
      }
      
      try {
        // Call the Tauri backend to verify the master password
        await invoke("verify_master_password", { password: form.data.password })
          .then(async (res) => {
            try {
              await AppState.refreshAuthState();
              if (AppState.isAuthenticated()) {
                // Clear the password field for security
                reset();
                // Navigate to the secrets page
                goto("/protected/secrets");
              }
            } catch (e) {
              toaster.error("Error refreshing auth state");
            }
          })
          .catch((e) => {
            toaster.error("Wrong password");
            // Stop form processing
            cancel();
          });
      } catch (error) {
        toaster.error("An error occurred");
        console.error(error);
        cancel();
      }
    },
    // After successful submission and processing
    onResult({ result }) {
      if (result.type === 'success') {
        toaster.success("Successfully logged in");
      }
    }
  });
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
    <form
      class="mt-8 space-y-6"
      method="POST"
      use:enhance
    >
      <div>
        <Label for="password" class="mb-2 dark:text-white">Main Password</Label>
        <Input
          bind:value={$form.password}
          type="password"
          name="password"
          id="password"
          placeholder="••••••••••••••••"
          required
          class="border outline-none dark:border-gray-600 dark:bg-gray-700"
          color={$errors.password ? 'red' : undefined}
          aria-invalid={$errors.password ? "true" : undefined}
        />
        {#if $errors.password}
          <Alert color="red" class="mt-2">
            <span class="font-medium">{$errors.password}</span>
          </Alert>
        {/if}
      </div>
      <Button type="submit" size="lg" disabled={$submitting}>
        {#if $submitting}
          <Spinner class="mr-3" size="4" color="white" />Loading...
        {:else}
          {loginTitle}
        {/if}
      </Button>
      
      {#if $message}
        <Alert color="blue" class="mt-3">
          <span class="font-medium">{$message}</span>
        </Alert>
      {/if}

      <div class="text-sm font-medium text-gray-500 dark:text-gray-400">
        First time? <A href="/account/setup">Setup</A>
      </div>
    </form>
  </Card>
</div>
