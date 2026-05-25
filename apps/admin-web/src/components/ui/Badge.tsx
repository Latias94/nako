import { cva, type VariantProps } from "class-variance-authority";
import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

const badgeVariants = cva("uiBadge", {
  variants: {
    tone: {
      neutral: "uiBadgeNeutral",
      success: "uiBadgeSuccess",
      warning: "uiBadgeWarning",
      danger: "uiBadgeDanger",
      info: "uiBadgeInfo",
    },
  },
  defaultVariants: {
    tone: "neutral",
  },
});

export type BadgeProps = HTMLAttributes<HTMLSpanElement> &
  VariantProps<typeof badgeVariants>;

export function Badge({ className, tone, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ tone }), className)} {...props} />;
}
