<script lang="ts">
    import {
        Label,
        Input,
        Checkbox,
        A,
        Button,
        Card,
        Toast,
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
    import { toaster, type ToastOptions } from '$lib/stores/toaster.svelte';

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

    let main_password = $state("");
    const onSubmit = async (e: Event) => {
        e.preventDefault();
        await invoke("verify_master_password", { password: main_password })
            .catch((e) => {
                main_password = ""
                toaster.error("Wrong password");
            })
            .then(async (res) => {
                main_password = ""
                try {
                    await AppState.refreshAuthState();
                    if (AppState.isAuthenticated()) { goto("/protected/secrets") }
                } catch (e) {
                    toaster.error("Error refreshing auth state");
                }
            });
    };
    let {
        setupFn = (event: MouseEvent & { currentTarget: EventTarget & HTMLAnchorElement }) => {
            alert("First time setup.");
        },
    } = $props();
    let mainDivClass =
        "flex flex-col items-center justify-center px-6 pt-8 mx-auto md:h-screen pt:mt-0 dark:bg-gray-900";
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
        <form class="mt-8 space-y-6" onsubmit={onSubmit}>
            <div>
                <Label for="password" class="mb-2 dark:text-white"
                    >Main Password</Label
                >
                <Input
                    bind:value={main_password}
                    type="password"
                    name="password"
                    id="password"
                    placeholder="••••••••••••••••"
                    required
                    class="border outline-none dark:border-gray-600 dark:bg-gray-700"
                />
            </div>
            <Button type="submit" size="lg">{loginTitle}</Button>

            <div class="text-sm font-medium text-gray-500 dark:text-gray-400">
                First time? <A href="/account/setup">Setup</A>
            </div>
        </form>
    </Card>
</div>
