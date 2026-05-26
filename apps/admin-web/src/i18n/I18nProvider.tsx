import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import {
  messageCatalogs,
  type AdminLocale,
  type MessageId,
} from "./messages";

type MessageValues = Record<string, boolean | number | string | null | undefined>;

type I18nContextValue = {
  locale: AdminLocale;
  setLocale(locale: AdminLocale): void;
  t(id: MessageId, values?: MessageValues): string;
};

const defaultLocale: AdminLocale = "en-US";
const localeStorageKey = "nako-admin-locale";
const availableLocales = Object.keys(messageCatalogs) as AdminLocale[];

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

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const setLocale = useCallback((nextLocale: AdminLocale) => {
    setLocaleState(nextLocale);
    try {
      window.localStorage.setItem(localeStorageKey, nextLocale);
    } catch {
      // Locale selection still works for the session when storage is unavailable.
    }
  }, []);

  const t = useCallback(
    (id: MessageId, values?: MessageValues) => formatMessage(messageCatalogs[locale][id], values),
    [locale],
  );

  const value = useMemo(
    () => ({
      locale,
      setLocale,
      t,
    }),
    [locale, setLocale, t],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
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
