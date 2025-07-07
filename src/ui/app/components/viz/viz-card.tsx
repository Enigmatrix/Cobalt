import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";

export function VizCard({
  className,
  ...props
}: React.ComponentProps<typeof Card>) {
  return <Card className={cn("gap-0 p-0", className)} {...props} />;
}

export function VizCardHeader({
  className,
  ...props
}: React.ComponentProps<typeof CardHeader>) {
  return <CardHeader className={cn("p-0", className)} {...props} />;
}

export function VizCardTitle({
  className,
  ...props
}: React.ComponentProps<typeof CardTitle>) {
  return <CardTitle className={cn("font-normal", className)} {...props} />;
}

export function VizCardDescription({
  className,
  ...props
}: React.ComponentProps<typeof CardDescription>) {
  return <CardDescription className={cn(className)} {...props} />;
}

export function VizCardAction({
  className,
  ...props
}: React.ComponentProps<typeof CardAction>) {
  return <CardAction className={cn(className)} {...props} />;
}

export function VizCardContent({
  className,
  ...props
}: React.ComponentProps<typeof CardContent>) {
  return (
    <CardContent
      className={cn("p-0 flex flex-col h-full", className)}
      {...props}
    />
  );
}

export function VizCardFooter({
  className,
  ...props
}: React.ComponentProps<typeof CardFooter>) {
  return <CardFooter className={cn(className)} {...props} />;
}
