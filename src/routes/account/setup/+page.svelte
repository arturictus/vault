<script>
    import FirstSetup from "$lib/components/FirstSetup.svelte";
    import { superForm } from "sveltekit-superforms";
    import { z } from "zod";
    import { zod } from "sveltekit-superforms/adapters";

    // Define the setup form schema directly in the client-side
    const setupSchema = z
        .object({
            password: z
                .string()
                .min(8, { message: "Password must be at least 8 characters" })
                .refine((val) => /[A-Z]/.test(val), {
                    message:
                        "Password must contain at least one uppercase letter",
                })
                .refine((val) => /[a-z]/.test(val), {
                    message:
                        "Password must contain at least one lowercase letter",
                })
                .refine((val) => /[0-9]/.test(val), {
                    message: "Password must contain at least one number",
                }),
            password_confirmation: z.string(),
            private_key: z.string().optional(),
        })
        .refine((data) => data.password === data.password_confirmation, {
            message: "Passwords don't match",
            path: ["password_confirmation"],
        });

    // Create the form with initial data
    const { form, errors, enhance, submitting, constraints } = superForm(
        {
            password: "",
            password_confirmation: "",
            private_key: "",
        },
        {
            SPA: true,
            dataType: "json",
            validators: zod(setupSchema),
            onSubmit: ({ formData, cancel }) => {
                // We'll handle the submit in the FirstSetup component
                // This is just for validation
                cancel();
            },
        },
    );
</script>

<FirstSetup {form} {errors} {enhance} {submitting} {constraints} />
