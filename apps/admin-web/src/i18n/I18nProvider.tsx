import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

import {
  baseCatalogs,
  loadCatalogNamespace,
  type LocaleMessageCatalogs,
  type MessageCatalog,
} from "./catalogLoader";
import type { AdminLocale, I18nNamespace, MessageId } from "./messages";

type MessageValues = Record<
  string,
  boolean | number | string | null | undefined
>;

type I18nContextValue = {
  ensureNamespaces(namespaces: readonly I18nNamespace[]): Promise<void>;
  hasNamespaces(namespaces: readonly I18nNamespace[]): boolean;
  locale: AdminLocale;
  setLocale(locale: AdminLocale): void;
  t(id: MessageId, values?: MessageValues): string;
};

const defaultLocale: AdminLocale = "en-US";
const localeStorageKey = "nako-admin-locale";
const availableLocaleMap = {
  "en-US": true,
  "zh-Hans": true,
} satisfies Record<AdminLocale, true>;
const availableLocales = Object.keys(availableLocaleMap) as AdminLocale[];

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({
  children,
  initialLocale,
}: {
  children: ReactNode;
  initialLocale?: AdminLocale;
}) {
  const [locale, setLocaleState] = useState<AdminLocale>(
    () => initialLocale ?? readStoredLocale() ?? defaultLocale,
  );
  const [namespaceCatalogs, setNamespaceCatalogs] = useState<
    Partial<Record<I18nNamespace, LocaleMessageCatalogs>>
  >({});
  const namespaceCatalogsRef = useRef(namespaceCatalogs);
  const pendingNamespacesRef = useRef(new Map<I18nNamespace, Promise<void>>());

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  useEffect(() => {
    namespaceCatalogsRef.current = namespaceCatalogs;
  }, [namespaceCatalogs]);

  const setLocale = useCallback((nextLocale: AdminLocale) => {
    setLocaleState(nextLocale);
    try {
      window.localStorage.setItem(localeStorageKey, nextLocale);
    } catch {
      // Locale selection still works for the session when storage is unavailable.
    }
  }, []);

  const ensureNamespaces = useCallback(
    async (namespaces: readonly I18nNamespace[]) => {
      await Promise.all(
        namespaces.map((namespace) => {
          if (namespaceCatalogsRef.current[namespace]) {
            return Promise.resolve();
          }

          const pendingNamespace = pendingNamespacesRef.current.get(namespace);
          if (pendingNamespace) {
            return pendingNamespace;
          }

          const pending = loadCatalogNamespace(namespace).then((catalogs) => {
            pendingNamespacesRef.current.delete(namespace);
            setNamespaceCatalogs((current) => {
              if (current[namespace]) {
                return current;
              }

              const next = {
                ...current,
                [namespace]: catalogs,
              };
              namespaceCatalogsRef.current = next;
              return next;
            });
          });

          pendingNamespacesRef.current.set(namespace, pending);
          return pending;
        }),
      );
    },
    [],
  );

  const hasNamespaces = useCallback(
    (namespaces: readonly I18nNamespace[]) =>
      namespaces.every((namespace) => Boolean(namespaceCatalogs[namespace])),
    [namespaceCatalogs],
  );

  const activeCatalog = useMemo(() => {
    const mergedCatalog: Record<string, string> = {
      ...baseCatalogs[locale],
    };

    for (const catalogs of Object.values(namespaceCatalogs)) {
      Object.assign(mergedCatalog, catalogs?.[locale]);
    }

    return mergedCatalog as MessageCatalog;
  }, [locale, namespaceCatalogs]);

  const t = useCallback(
    (id: MessageId, values?: MessageValues) =>
      formatMessage(activeCatalog?.[id] ?? id, values),
    [activeCatalog],
  );

  const value = useMemo(
    () => ({
      ensureNamespaces,
      hasNamespaces,
      locale,
      setLocale,
      t,
    }),
    [ensureNamespaces, hasNamespaces, locale, setLocale, t],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function I18nNamespaceBoundary({
  children,
  namespace,
}: {
  children: ReactNode;
  namespace: I18nNamespace | readonly I18nNamespace[];
}) {
  const namespaces = useMemo(
    () => (typeof namespace === "string" ? [namespace] : [...namespace]),
    [namespace],
  );
  const { ensureNamespaces, hasNamespaces } = useI18n();

  useEffect(() => {
    void ensureNamespaces(namespaces);
  }, [ensureNamespaces, namespaces]);

  if (!hasNamespaces(namespaces)) {
    return null;
  }

  return <>{children}</>;
}

export function useI18n() {
  const value = useContext(I18nContext);
  if (!value) {
    throw new Error("useI18n must be used within I18nProvider");
  }

  return value;
}

export function supportedAdminLocales() {
  return availableLocales;
}

function readStoredLocale(): AdminLocale | null {
  try {
    return normalizeLocale(window.localStorage.getItem(localeStorageKey));
  } catch {
    return null;
  }
}

function normalizeLocale(locale: string | null): AdminLocale | null {
  if (!locale) {
    return null;
  }

  if (locale === "zh-Hans" || locale.toLowerCase().startsWith("zh")) {
    return "zh-Hans";
  }

  if (locale === "en-US" || locale.toLowerCase().startsWith("en")) {
    return "en-US";
  }

  return null;
}

function formatMessage(template: string, values?: MessageValues) {
  if (!values) {
    return template;
  }

  return template.replace(/\{([a-zA-Z0-9_]+)\}/g, (match, key: string) => {
    const value = values[key];
    return value === undefined || value === null ? match : String(value);
  });
}
