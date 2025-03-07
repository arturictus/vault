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
    import MainContent from "$lib/components/MainContent.svelte";
    import { superForm } from "sveltekit-superforms";
    import { z } from "zod";
    import { zod } from "sveltekit-superforms/adapters";
    import { toaster } from "$lib/stores/toaster.svelte";
    import Form from "$lib/components/secrets_create/Form.svelte";
    import SuperDebug from "sveltekit-superforms";

    // Define the schema for form validation
    const schema = z.object({
        name: z.string().min(5, { message: "Name is required" }),
        value: z.string().min(10000, { message: "Value is required" }),
        kind: z.enum(["login", "note", "crypto_key"], {
            required_error: "Please select a type",
        }),
    });

    // Define a type based on the schema
    type FormData = z.infer<typeof schema>;

    // Initialize the form with superValidateSync for client-side validation
    const { form, errors, enhance, submitting, constraints, validateForm } =
        superForm(
            {
                name: "",
                value: "",
                kind: "",
            },
            {
                SPA: true,
                dataType: "json",
                validators: zod(schema),
                onSubmit: async ({ formData, cancel }) => {
                    // Run explicit validation before proceeding
                    const result = await validateForm({ update: true });
                    if (result.valid) {
                        try {
                            toaster.info("Creating secret...");
                            // At this point validation has already passed
                            const data = {
                                name: $form.name,
                                value: $form.value,
                                kind: $form.kind,
                            };
                            const response = await invoke("create_secret", {
                                data: data,
                            });
                            goto("/protected/secrets");
                            toaster.success("Secret created successfully!");
                        } catch (err) {
                            toaster.error("Error creating secret");
                            cancel();
                        }
                    }
                    // If validation fails, prevent submission and return
                    toaster.error("Please fix the form errors");
                    cancel();
                    return;
                },
            },
        );

    let types = [
        { value: "login", name: "Login" },
        { value: "note", name: "Note" },
        { value: "crypto_key", name: "crypto_key" },
    ];
</script>

{#snippet singleBlock()}
    <Form {form} {errors} {enhance} {constraints} {submitting}/>
{/snippet}
<MainContent main={singleBlock} />
