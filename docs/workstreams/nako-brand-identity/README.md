# Nako Brand Identity

Status: Design record
Last updated: 2026-05-22

This record captures the accepted Nako brand direction, icon meaning, and final
source icon generation prompt. It is a brand-design record, not an
implementation lane.

## Selected Asset

- Canonical source icon: `assets/brand/nako-app-icon-1024.png`
- Source size: `1024x1024`
- Public tagline: "Your media home, gently kept."
- One-line introduction: "Nako is an open-source, self-hosted media home for
  gently organizing, keeping, and playing your films, shows, anime, and personal
  collection."

## Icon Meaning

Nako should feel like a small media companion for a private library: friendly,
calm, open, and personally owned. The mint/cyan fox-cat media spirit represents
a light, trustworthy keeper for a self-hosted collection rather than a platform
mascot or streaming-service mark.

The deep teal-blue background stands for a quiet, private self-hosted space.
The mint/cyan character palette keeps the product feeling fresh, approachable,
and technical without becoming corporate. The play orb anchors the icon in the
core experience: media that is kept with care and ready to play. The orb should
glow, but not dominate the mascot or become a harsh UI button.

The final direction deliberately avoids provider logos, letter marks,
streaming-service storefront language, and references to existing copyrighted
characters. The mascot should be cute enough to feel personal, but restrained
enough to work as an app icon and brand mark at small sizes.

## Final Icon Prompt

```text
Use case: logo-brand
Asset type: final 1024x1024 app icon master, restrained v3 candidate

Create a polished 1024x1024 rounded-square app icon for Nako, an open-source self-hosted media server.

Use the selected concept: a cute mint/cyan fox-cat media spirit mascot on a deep teal-blue background, hugging a glowing cyan-white play button orb. Keep the mascot simplified and logo-like: rounded fox/cat ears, clean fluffy head silhouette, compact small body, simple soft paws, black oval eyes, tiny calm mouth, and a curled tail.

Important design correction from v2: the play orb must be softer and less overexposed. Use a thinner cyan rim light, reduce the thick white halo, avoid a harsh white ring, and keep the orb edge integrated into the mascot. The paws should visibly overlap the orb so it feels genuinely hugged, not pasted on. Keep the white play triangle crisp and readable at 64px, but reduce the surrounding orb edge brightness and bottom glow.

Visual direction: more restrained and iconic than plush character art; simplified shape language, fewer fur spikes and micro-details, strong silhouette, clean vector-like edges, generous padding, centered composition.

Style: glossy modern 2.5D vector-like app icon, anime-adjacent but not VTuber-like, high saturation, soft gradients, polished indie app icon quality.

Composition: centered mascot, deep teal-blue rounded-square background, mint/cyan mascot, bright but controlled play orb in front, no text, no letters, no watermark.

Avoid: overexposed play orb rim, thick white circular border, harsh bloom, Netflix-like N marks, Bilibili TV shape, Discord mascot, Pokemon, Sanrio, Ghibli, VTuber-specific look, copyrighted characters, clutter, labels, photorealism, busy fur, oversized facial details, excessive whiskers, paw-pad detail overload, too much cuteness.
```

## Boundaries

- Keep final canonical brand assets in `assets/brand/`.
- Do not commit temporary prompt iterations or generated grids as canonical
  assets.
- Derive platform launcher icons from `assets/brand/nako-app-icon-1024.png`
  after the Nako rename/rebrand decision is accepted.
- Preserve the visual distinction from provider logos, streaming-service marks,
  and existing character IP.
