<script>
	import { Label, Input, Checkbox, A, Button, Card } from "flowbite-svelte";
	let title = "Access to your vault";
	let site = {
		name: "Flowbite",
		img: "/images/flowbite-svelte-icon-logo.svg",
		link: "/",
		imgAlt: "FlowBite Logo",
	};
	let rememberMe = true;
	let lostPassword = true;
	let createAccount = true;
	let lostPasswordLink = "";
	let loginTitle = "Login to your account";
	let registerLink = "";
	let createAccountTitle = "Create account";

	let mainClass = "bg-gray-50 dark:bg-gray-900 w-full";
	let mainDivClass =
		"flex flex-col items-center justify-center px-6 pt-8 mx-auto md:h-screen pt:mt-0 dark:bg-gray-900";
	let siteLinkClass =
		"flex items-center justify-center mb-8 text-2xl font-semibold lg:mb-10 dark:text-white";
	let siteImgClass = "mr-4 h-11";
	let cardH1Class = "text-2xl font-bold text-gray-900 dark:text-white";
	let { setup_fn = (event) => {}, children } = $props();
</script>

<main class={mainClass}>
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
				{@render children()}
				<Button type="submit" size="lg">{loginTitle}</Button>

				<div
					class="text-sm font-medium text-gray-500 dark:text-gray-400"
				>
					First time? <span
						class="text-primary-700 dark:text-blue-500 hover:underline cursor-pointer font-medium"
						on:click={setup_fn}>Setup</span
					>
				</div>
			</form>
		</Card>
	</div>
</main>
