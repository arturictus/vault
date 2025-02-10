<script lang="ts">
    import { Label, Input, Checkbox, A, Button, Card } from "flowbite-svelte";

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


    const onSubmit = (e: Event) => {
        const formData = new FormData(e.target as HTMLFormElement);

        const data: Record<string, string | File> = {};
        for (const field of formData.entries()) {
            const [key, value] = field;
            data[key] = value;
        }
        console.log(data);
    };
    let {setupFn = (event: Event) => {
        alert("First time setup.");
    }} = $props();
    let mainDivClass =
		"flex flex-col items-center justify-center px-6 pt-8 mx-auto md:h-screen pt:mt-0 dark:bg-gray-900";

</script>

<div class={mainDivClass}>
    <a href={site.link} class={siteLinkClass}>
        <img src={site.img} class={siteImgClass} alt={site.imgAlt} />
        <span>{site.name}</span>
    </a>
    <!-- Card -->
    <Card class="w-full" size="md" border={false}>
        <h1 class={cardH1Class}>
            {title}
        </h1>
        <form class="mt-8 space-y-6">
            <div>
                <Label for="password" class="mb-2 dark:text-white"
                    >Main Password</Label
                >
                <Input
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
                First time? <span
                    class="text-primary-700 dark:text-blue-500 hover:underline cursor-pointer font-medium"
                    on:click={setupFn}>Setup</span
                >
            </div>
        </form>
    </Card>
</div>
