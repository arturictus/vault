<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button, P } from "flowbite-svelte";
  import Toaster  from "$lib/components/Toaster.svelte";
  export let title = "Welcome to Vault";
  export let description =
    'Seems that is your first time here. Please click on the "Setup" button to start.';
  export let image = {
    src: "/svelte.svg",
    alt: "maintenance",
  };

  export let mainDivClass =
    "flex flex-col justify-center items-center px-6 mx-auto h-screen xl:px-0 dark:bg-gray-900";
  export let h1Class =
    "mb-3 text-2xl font-bold leading-tight text-gray-900 sm:text-4xl lg:text-5xl dark:text-white";
  export let imgDiv = "block mb-5 md:max-w-md";
  export let div2Class = "text-center xl:max-w-4xl";
</script>

<div class={mainDivClass}>
  <div class={imgDiv}>
    <img src={image.src} alt={image.alt} />
  </div>

  <div class={div2Class}>
    <h1 class={h1Class}>{title}</h1>

    <P
      class="mb-5 text-base font-normal text-gray-500 dark:text-gray-400 md:text-lg"
    >
      {@html description}
    </P>

    <Button
      onclick={() => {
        goto("/account/log_in");
      }}>Access Your Vault</Button
    >
    <Button
      color="light"
      onclick={() => {
        goto("/account/setup");
      }}>Start a New vault</Button
    >
  </div>
</div>
