import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const ts = require(path.resolve("apps/admin-web/node_modules/typescript"));

const sourcePath = path.resolve("apps/admin-web/src/i18n/messages.ts");
const catalogsDir = path.resolve("apps/admin-web/src/i18n/catalogs");
const sourceText = fs.readFileSync(sourcePath, "utf8");
const sourceFile = ts.createSourceFile(
  sourcePath,
  sourceText,
  ts.ScriptTarget.Latest,
  true,
);

const basePrefixes = new Set(["locale", "nav", "shell", "source"]);

const enMessages = readObject("enMessages");
const zhHansMessages = readObject("zhHansMessages");
const prefixes = Array.from(
  new Set(enMessages.map((entry) => prefixOf(entry.key))),
);
const namespacePrefixes = prefixes.filter(
  (prefix) => !basePrefixes.has(prefix),
);

fs.mkdirSync(catalogsDir, { recursive: true });

writeCatalog(
  "base",
  prefixes.filter((prefix) => basePrefixes.has(prefix)),
);
for (const namespace of namespacePrefixes) {
  writeCatalog(namespace, [namespace]);
}

writeMessagesEntrypoint(["base", ...namespacePrefixes]);
writeCatalogLoader(namespacePrefixes);

function readObject(name) {
  const declaration = findVariable(name);
  const initializer = declaration
    ? unwrapExpression(declaration.initializer)
    : null;
  if (!initializer || !ts.isObjectLiteralExpression(initializer)) {
    throw new Error(`Expected ${name} object literal`);
  }

  return initializer.properties.map((property) => {
    if (
      !ts.isPropertyAssignment(property) ||
      !ts.isStringLiteral(property.name)
    ) {
      throw new Error(`Expected string literal property in ${name}`);
    }

    return {
      key: property.name.text,
      valueText: property.initializer.getText(sourceFile),
    };
  });
}

function unwrapExpression(expression) {
  if (ts.isAsExpression(expression) || ts.isSatisfiesExpression(expression)) {
    return unwrapExpression(expression.expression);
  }

  return expression;
}

function findVariable(name) {
  let result = null;

  function visit(node) {
    if (
      ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      node.name.text === name
    ) {
      result = node;
      return;
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return result;
}

function prefixOf(key) {
  return key.split(".")[0];
}

function pascalCase(name) {
  return name
    .replace(/(^|[^a-zA-Z0-9]+)([a-zA-Z0-9])/g, (_match, _separator, char) =>
      char.toUpperCase(),
    )
    .replace(/[^a-zA-Z0-9]/g, "");
}

function catalogConstName(prefix, locale) {
  const pascal = pascalCase(prefix);
  return `${locale === "en-US" ? "en" : "zhHans"}${pascal}Messages`;
}

function catalogTypeName(prefix) {
  return `${pascalCase(prefix)}MessageId`;
}

function messageCatalogsConstName(prefix) {
  return `${prefix}MessageCatalogs`;
}

function writeCatalog(prefix, includedPrefixes) {
  const enEntries = enMessages.filter((entry) =>
    includedPrefixes.includes(prefixOf(entry.key)),
  );
  const zhEntries = zhHansMessages.filter((entry) =>
    includedPrefixes.includes(prefixOf(entry.key)),
  );
  const enKeys = new Set(enEntries.map((entry) => entry.key));
  const zhKeys = new Set(zhEntries.map((entry) => entry.key));

  for (const key of enKeys) {
    if (!zhKeys.has(key)) {
      throw new Error(`Missing zh-Hans translation for ${key}`);
    }
  }

  for (const key of zhKeys) {
    if (!enKeys.has(key)) {
      throw new Error(`Missing en-US translation for ${key}`);
    }
  }

  const enConst = catalogConstName(prefix, "en-US");
  const zhConst = catalogConstName(prefix, "zh-Hans");
  const typeName = catalogTypeName(prefix);
  const catalogConst = messageCatalogsConstName(prefix);
  const lines = [
    `export const ${enConst} = {`,
    ...enEntries.map(
      (entry) => `  ${JSON.stringify(entry.key)}: ${entry.valueText},`,
    ),
    `} as const;`,
    "",
    `export type ${typeName} = keyof typeof ${enConst};`,
    "",
    `export const ${zhConst} = {`,
    ...zhEntries.map(
      (entry) => `  ${JSON.stringify(entry.key)}: ${entry.valueText},`,
    ),
    `} satisfies Record<${typeName}, string>;`,
    "",
    `export const ${catalogConst} = {`,
    `  "en-US": ${enConst},`,
    `  "zh-Hans": ${zhConst},`,
    `} as const;`,
    "",
  ];

  fs.writeFileSync(
    path.join(catalogsDir, `${prefix}.ts`),
    lines.join("\n"),
    "utf8",
  );
}

function writeMessagesEntrypoint(prefixesToImport) {
  const imports = prefixesToImport.map(
    (prefix) =>
      `import type { ${catalogConstName(prefix, "en-US")} } from "./catalogs/${prefix}";`,
  );
  const messageIdLines = prefixesToImport.map(
    (prefix) => `  | keyof typeof ${catalogConstName(prefix, "en-US")}`,
  );
  const namespaceLines = namespacePrefixes.map(
    (prefix) => `  | ${JSON.stringify(prefix)}`,
  );
  const lines = [
    ...imports,
    "",
    `export type AdminLocale = "en-US" | "zh-Hans";`,
    "",
    "export type I18nNamespace =",
    ...namespaceLines.map((line, index) =>
      index === namespaceLines.length - 1 ? `${line};` : line,
    ),
    "",
    "export type MessageId =",
    ...messageIdLines.map((line, index) =>
      index === messageIdLines.length - 1 ? `${line};` : line,
    ),
    "",
  ];

  fs.writeFileSync(sourcePath, lines.join("\n"), "utf8");
}

function writeCatalogLoader(namespaces) {
  const loaderPath = path.resolve("apps/admin-web/src/i18n/catalogLoader.ts");
  const loaderLines = namespaces.map((namespace) => {
    const catalogConst = messageCatalogsConstName(namespace);
    return `  ${namespace}: () => import("./catalogs/${namespace}").then((module) => module.${catalogConst}),`;
  });
  const lines = [
    `import { baseMessageCatalogs } from "./catalogs/base";`,
    `import type { AdminLocale, I18nNamespace, MessageId } from "./messages";`,
    "",
    "export type MessageCatalog = Readonly<Partial<Record<MessageId, string>>>;",
    "export type LocaleMessageCatalogs = Readonly<Record<AdminLocale, MessageCatalog>>;",
    "",
    "export const baseCatalogs: LocaleMessageCatalogs = baseMessageCatalogs;",
    "",
    "const namespaceCatalogLoaders = {",
    ...loaderLines,
    "} satisfies Record<I18nNamespace, () => Promise<LocaleMessageCatalogs>>;",
    "",
    "export function loadCatalogNamespace(namespace: I18nNamespace) {",
    "  return namespaceCatalogLoaders[namespace]();",
    "}",
    "",
  ];

  fs.writeFileSync(loaderPath, lines.join("\n"), "utf8");
}
