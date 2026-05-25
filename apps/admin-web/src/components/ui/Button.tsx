import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";

import { cn } from "../../lib/cn";

const buttonVariants = cva("uiButton", {
  variants: {
    variant: {
      default: "uiButtonDefault",
      outline: "uiButtonOutline",
      ghost: "uiButtonGhost",
    },
    size: {
      sm: "uiButtonSm",
      md: "uiButtonMd",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "md",
  },
});

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof buttonVariants>;

export function Button({ className, variant, size, ...props }: ButtonProps) {
  return (
    <button
      className={cn(buttonVariants({ variant, size }), className)}
      type={props.type ?? "button"}
      {...props}
    />
  );
}
