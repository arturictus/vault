<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import SignIn from '$lib/utils/authentication/SignIn.svelte';
    
	let title = 'Sign in to platform';
	let site = {
		name: 'Vault',
		img: '/svelte.svg',
		link: '/',
		imgAlt: 'FlowBite Logo'
	};
	let rememberMe = false;
	let lostPassword = false;
	let createAccount = true;
	let lostPasswordLink = 'forgot-password';
	let loginTitle = 'Access';
	let registerLink = '/first_setup';
	let createAccountTitle = 'Setup';

	const onSubmit = (e: Event) => {
		const formData = new FormData(e.target as HTMLFormElement);

		const data: Record<string, string | File> = {};
		for (const field of formData.entries()) {
			const [key, value] = field;
			data[key] = value;
		}
		console.log(data);
	};
</script>

<SignIn
	{title}
	{site}
	{rememberMe}
	{lostPassword}
	{createAccount}
	{lostPasswordLink}
	{loginTitle}
	{registerLink}
	{createAccountTitle}
	on:submit={onSubmit}
>
	<div>
		<Label for="password" class="mb-2 dark:text-white">Your password</Label>
		<Input
			type="password"
			name="password"
			id="password"
			placeholder="••••••••"
			required
			class="border outline-none dark:border-gray-600 dark:bg-gray-700"
		/>
	</div>
</SignIn>