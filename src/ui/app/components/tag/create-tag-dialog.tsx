import * as React from "react";
import { z } from "zod";
import { useZodForm } from "@/hooks/use-form";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { ColorPicker } from "@/components/color-picker";
import { tagSchema } from "@/lib/schema";
import { ChooseMultiApps } from "@/components/app/choose-multi-apps";
import { randomColor } from "@/lib/color-utils";
import type { SubmitHandler } from "react-hook-form";
import { ScoreSlider } from "@/components/tag/score-slider";
import { getScoreDescription } from "@/components/tag/score";

type FormValues = z.infer<typeof tagSchema>;

interface CreateTagDialogProps {
  onSubmit: (values: FormValues) => Promise<void>;
  trigger?: React.ReactNode;
}

export function CreateTagDialog({ onSubmit, trigger }: CreateTagDialogProps) {
  const [open, setOpenInner] = React.useState(false);

  const form = useZodForm({
    schema: tagSchema,
    defaultValues: {
      name: "",
      color: randomColor(),
      apps: [],
      score: 0,
    },
  });

  const setOpen = React.useCallback(
    (open: boolean) => {
      setOpenInner(open);
      if (open) form.reset();
    },
    [setOpenInner, form],
  );

  const handleSubmit: SubmitHandler<FormValues> = async (values) => {
    await onSubmit(values);
    setOpen(false);
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={setOpen} modal>
      <DialogTrigger asChild>
        {trigger ?? <Button variant="outline">Create Tag</Button>}
      </DialogTrigger>
      <DialogContent className="flex flex-col max-h-[calc(100vh-6rem)] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Tag</DialogTitle>
          <DialogDescription>
            Create a new tag to organize your apps.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form id="create-tag" className="space-y-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Name</FormLabel>
                  <FormControl>
                    <Input placeholder="Enter tag name" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="color"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Color</FormLabel>
                  <FormControl>
                    <ColorPicker
                      className="block w-full"
                      color={field.value}
                      onChange={field.onChange}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="score"
              render={({ field }) => (
                <FormItem>
                  <FormLabel className="h-8 text-sm gap-2 flex items-center">
                    Score
                    <div>-</div>
                    <span className="text-sm text-muted-foreground min-w-0 truncate">
                      {getScoreDescription(field.value)}
                    </span>
                    <span className="text-sm text-muted-foreground">
                      ({field.value})
                    </span>
                    {field.value !== 0 && (
                      <Button
                        variant="outline"
                        size="sm"
                        className="px-2 py-1 h-6 m-0 text-xs"
                        onClick={() => field.onChange(0)}
                      >
                        Reset
                      </Button>
                    )}
                  </FormLabel>
                  <FormControl>
                    <ScoreSlider
                      className="w-full h-8"
                      value={field.value}
                      onValueChange={field.onChange}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="apps"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Apps</FormLabel>
                  <FormControl>
                    <div className="max-w-full overflow-hidden">
                      <ChooseMultiApps
                        placeholder="Select apps"
                        value={field.value}
                        onValueChanged={field.onChange}
                      />
                    </div>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button
                type="button"
                form="create-tag"
                onClick={form.handleSubmit(handleSubmit)}
              >
                Create
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
