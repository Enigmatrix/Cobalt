import { useZodForm } from "@/hooks/use-form";
import { alertSchema } from "@/lib/schema";
import type { z } from "zod";
import { Form } from "@/components/ui/form";

type FormValues = z.infer<typeof alertSchema>;

export function CreateAlertForm({
  onSubmit,
}: {
  onSubmit: (values: FormValues) => void;
}) {
  const form = useZodForm({
    schema: alertSchema,
    defaultValues: {
      reminders: [],
    },
  });

  return (
    <>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className="space-y-4"
        ></form>
      </Form>
    </>
  );
}
