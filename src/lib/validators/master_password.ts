import { z } from 'zod';
import { superValidate } from 'sveltekit-superforms';
import { superForm, setMessage, setError } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

const schema = z.object({
  password: z.string().min(5).trim()
});

export type MasterPassword = z.infer<typeof schema>;

const buildForm = (data: MasterPassword) => {
  // { form, errors, message, constraints, enhance }
  const form = superForm(
    data,
    {
      SPA: true,
      validators: zod(schema),
      onUpdate({ form }) {
        // Form validation
        if (form.data.password.includes('1234')) {
          setError(form, 'password', 'Not a good password.');
        } else if (form.valid) {
          // TODO: Call an external API with form.data, await the result and update form
          setMessage(form, 'Valid data!');
        }
      }
    }
  );
  return form;
}

export default {schema, buildForm};