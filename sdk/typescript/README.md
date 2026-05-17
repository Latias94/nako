# Taru TypeScript SDK

This package contains the generated TypeScript SDK for the Taru public client
API. It is private for now and exists to compile-check the generated client
surface before npm publishing is designed.

## Generate

```bash
npm run generate --prefix sdk/typescript
```

The command refreshes `src/index.ts` from `taru-api`. Do not edit generated
source by hand.

## Check

```bash
npm run check --prefix sdk/typescript
```

The check runs `tsc --noEmit` with strict settings against the generated SDK.
