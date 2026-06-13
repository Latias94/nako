import type { ReactNode } from "react";

import { I18nNamespaceBoundary } from "../i18n/I18nProvider";
import type { I18nNamespace } from "../i18n/messages";

export function RouteI18n({
  children,
  namespace,
}: {
  children: ReactNode;
  namespace: I18nNamespace | readonly I18nNamespace[];
}) {
  return (
    <I18nNamespaceBoundary namespace={namespace}>
      {children}
    </I18nNamespaceBoundary>
  );
}
