import { type UseFormProps, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import type { z } from "zod";

export const useZodForm = <TSchema extends z.ZodType>({
  schema,
  ...props
}: Omit<UseFormProps<TSchema["_input"]>, "resolver"> & {
  schema: TSchema;
}) => {
  const form = useForm<TSchema["_input"]>({
    ...props,
    resolver: zodResolver(schema, undefined),
  });

  return form;
};
