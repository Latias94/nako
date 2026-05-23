# Nako Context

Nako is a self-hosted media server backend. This glossary defines the project
language used when discussing media libraries, extension surfaces, automation,
and playback boundaries.

## Language

**Addon**:
A user-enabled extension that adds Nako capabilities outside the core server trust boundary while providing a Jellyfin-like extensibility experience.
_Avoid_: Plugin, Jellyfin plugin compatibility

**Addon Protocol**:
The compatibility contract an **Addon** follows so Nako can discover and call its capabilities.
_Avoid_: Plugin ABI, native plugin contract

**Addon Protocol Version**:
A declared compatibility version of the **Addon Protocol**.
_Avoid_: Nako server version

**Addon Version**:
The implementation or package version of an **Addon Sidecar**.
_Avoid_: Addon Protocol Version, Nako release version

**Addon Sidecar**:
An independently running process or service that implements the **Addon Protocol**.
_Avoid_: In-process plugin

**Addon Token**:
A scoped credential that lets an **Addon Sidecar** call Nako APIs for granted capabilities.
_Avoid_: Server admin token, database credential

**Addon Token Rotation**:
The act of replacing an **Addon Token** without changing the addon registration or granted capabilities.
_Avoid_: OAuth flow

**Addon Event Subscription**:
A manifest-declared interest in Nako domain events that Nako delivers to an **Addon Sidecar**.
_Avoid_: Database polling, hidden scheduler

**Addon Task**:
A Nako-tracked job whose execution logic is provided by an **Addon Sidecar**.
_Avoid_: Hidden addon background job

**Addon External Fetch**:
Network or storage access an **Addon Sidecar** performs for its own execution.
_Avoid_: Nako-managed download

**Nako-Managed Artifact**:
Downloaded or generated content that Nako stores, indexes, serves, or treats as part of a library or playback experience.
_Avoid_: Addon private cache

**Library File Write**:
A change to files stored in a media library, such as subtitles, NFO files, artwork, or sidecar assets.
_Avoid_: Direct addon path write

**Addon Manager**:
Future Nako functionality for discovering, installing, updating, starting, or removing addons.
_Avoid_: Addon Protocol

**Addon Install Guide**:
Instructions or generated deployment snippets that help a user run an **Addon Sidecar** outside Nako.
_Avoid_: Nako-managed container lifecycle

**Addon Health Check**:
Nako's check that an **Addon Sidecar** is reachable and still matches its registered manifest contract.
_Avoid_: Process supervision

**Addon Resource**:
A declared capability of an **Addon** that Nako may call after the user grants the required access.
_Avoid_: Plugin hook, arbitrary callback

**Addon Entry Point**:
A manifest-declared place where Nako surfaces an **Addon** action or view to a user.
_Avoid_: Embedded frontend plugin

**Addon Hosted Page**:
An external page served by an **Addon Sidecar** for advanced settings or diagnostics.
_Avoid_: Embedded trusted admin UI

**Addon Configuration Schema**:
A manifest-declared shape for settings Nako stores and presents for an **Addon**.
_Avoid_: Addon-owned settings UI state

**Secret Reference**:
A stored reference to a secret value resolved by Nako at runtime.
_Avoid_: Plaintext secret

**Media Library**:
A configured collection boundary that gives media sources shared storage, metadata, and permission context.
_Avoid_: Folder

**Media Domain**:
A broad processing capability class for media, such as video, audio, image, document, mixed, or online.
_Avoid_: Library type

**Media Server Scope**:
Nako's long-term product scope as a self-hosted server for video, audio, image, document, mixed, and online media.
_Avoid_: Video-only product

**Video-First Phase**:
The current implementation phase that prioritizes self-hosted movie, series, anime, home-video, playback, metadata, and transcode workflows.
_Avoid_: Permanent video-only architecture

**Library Preset**:
A user-facing template that configures defaults for a media library without permanently defining every item in that library.
_Avoid_: Hard content type

**Media Source**:
One discoverable playable file or remote object within a **Media Library**.
_Avoid_: Movie, episode, item

**Source Locator**:
The library-scoped address Nako uses to find a **Media Source**.
_Avoid_: Global path identity

**Source Fingerprint**:
Evidence used to compare whether two **Media Sources** may contain the same media bytes.
_Avoid_: Source identity

**Source Duplicate Relationship**:
A relationship indicating that multiple **Media Sources** likely refer to the same content.
_Avoid_: Merged source

**Source Variant**:
A playable technical variant of the same media item, such as different resolution, codec, bitrate, HDR, or optimized output.
_Avoid_: Edition

**Source Variant Label**:
A user-facing label for a **Source Variant** derived from technical facts, local inference, or user override.
_Avoid_: Playback decision fact

**Edition**:
A content-level version with meaningfully different cut, runtime, or release form.
_Avoid_: Source variant

**Edition Relationship**:
A relationship connecting media items that are different editions of the same work.
_Avoid_: Source variant group

**Playback Source Selection**:
The runtime decision that chooses which **Media Source** or **Source Variant** to play for a user and client.
_Avoid_: Permanent default source

**Media Item**:
A user-facing media entry such as a movie, season, episode, or collection.
_Avoid_: Source, file, path

**Media Item Hierarchy**:
The parent-child structure that relates movies, series, seasons, and episodes.
_Avoid_: Provider-specific hierarchy

**Provider Subject**:
A provider-specific media concept such as a Bangumi subject, TMDB movie, TMDB series, or Douban entry.
_Avoid_: Media Item

**Provider Mapping**:
The relationship between a **Media Item** and one or more **Provider Subjects**.
_Avoid_: Provider-owned item identity

**Local Inference**:
Low-confidence structure and identity evidence inferred from paths, file names, nearby local files, and media probe facts.
_Avoid_: Metadata provider, scraper

**Local Inference Evidence**:
The path, file-name, directory, or local-file facts that explain a **Local Inference** result.
_Avoid_: Canonical metadata

**Local Inference Version**:
The parser or rule version that produced a **Local Inference** result.
_Avoid_: Provider version

**Provisional Hierarchy**:
A locally inferred **Media Item Hierarchy** that can be browsed before provider or NFO confirmation.
_Avoid_: Provider-confirmed hierarchy

**Hierarchy Confirmation**:
The process of applying NFO, provider mapping, user edits, or accepted addon writes to a **Provisional Hierarchy**.
_Avoid_: New item replacement

**Hierarchy Repair**:
The exceptional process of splitting, merging, or reparenting items when local inference produced the wrong hierarchy.
_Avoid_: Normal provider refresh

**Unknown Media Item**:
A media item Nako discovered but cannot confidently classify yet.
_Avoid_: Guessed movie or episode

**Genre**:
A broad provider or local category used for browsing and filtering media items.
_Avoid_: Tag

**Tag**:
A flexible label from local metadata, providers, addons, or users used for filtering and organization.
_Avoid_: Genre, media kind

**Episode-Like Item**:
A provider-specific episode, special, OVA, OAD, or similar entry that belongs in watch order.
_Avoid_: Extra

**Extra Item**:
A media entry related to another item but not part of the primary watch order.
_Avoid_: Episode

**Franchise Collection**:
A collection that groups related movies, series, specials, or extras across one franchise.
_Avoid_: Forced season hierarchy

**Canonical Metadata**:
The authoritative metadata Nako uses for browsing, search, playback presentation, and exports.
_Avoid_: Provider payload, suggestion

**Media Technical Facts**:
Observed properties of a media source or stream, such as codec, resolution, bitrate, duration, audio language, or subtitle language.
_Avoid_: Canonical metadata

**User Library State**:
User- or library-specific state such as watch status, playback progress, user rating, date added, or view date.
_Avoid_: Canonical metadata

**Library Item State**:
Library-scoped state such as date added or library visibility that is shared across users.
_Avoid_: User playback state

**User Playback State**:
User-scoped state such as playback progress, watched status, last played time, favorites, hidden status, or user rating.
_Avoid_: Global item field

**Core Metadata Field**:
A field useful across media domains or required by the current video-first experience.
_Avoid_: Domain-specific field

**Domain-Specific Metadata**:
Metadata that only makes sense for a narrower media domain such as music, podcasts, photos, books, or online catalogs.
_Avoid_: Core metadata field

**Review Rating**:
A provider or community score for a media item, usually with scale and vote count.
_Avoid_: Content rating, user rating

**Content Rating**:
A content classification or age guidance label such as PG-13, R, or TV-MA.
_Avoid_: Review rating

**User Rating**:
A rating a specific user assigns to a media item.
_Avoid_: Review rating

**Browse Facet**:
A supported filtering dimension exposed to client applications.
_Avoid_: Raw database column

**Sort Key**:
A supported ordering option exposed to client applications.
_Avoid_: Implementation detail

**Metadata Source Priority**:
The ordered preference Nako uses when multiple metadata sources can fill the same field.
_Avoid_: Field-level permission

**NFO Import**:
Reading local NFO metadata into Nako's metadata pipeline.
_Avoid_: NFO sync

**NFO Export**:
Writing Nako metadata back to NFO sidecar files for a media library.
_Avoid_: Unconditional NFO overwrite

**NFO Round Trip**:
Preserving existing NFO content that Nako does not own while importing or exporting known metadata.
_Avoid_: Destructive NFO rewrite

**Artwork Source**:
The original provider, file, or addon reference from which artwork was discovered.
_Avoid_: Canonical artwork file

**Managed Artwork**:
Artwork stored or cached by Nako for stable browsing and client presentation.
_Avoid_: Remote provider URL

**Artwork Candidate**:
One possible artwork choice discovered from an **Artwork Source**.
_Avoid_: Selected artwork

**Selected Artwork**:
The artwork choice currently used for a media item's primary presentation.
_Avoid_: Only artwork

**Artwork Export**:
Writing **Managed Artwork** to media-library sidecar files such as posters or backdrops.
_Avoid_: Provider hotlinking

**Playback Transcode**:
A temporary transcode produced for an active or reusable playback session.
_Avoid_: Optimized version

**Optimized Version**:
A long-lived derived media version created ahead of playback.
_Avoid_: Playback transcode cache

**Hardware Capability Report**:
A cached summary of available transcode hardware acceleration options.
_Avoid_: Per-session hardware probe

**Hardware Capability Refresh**:
An explicit refresh of the **Hardware Capability Report** after configuration or host hardware changes.
_Avoid_: Implicit playback probe

**Hardware Acceleration Policy**:
The configured rule for choosing hardware or CPU transcode execution.
_Avoid_: Transcode profile

**Transcode Profile**:
A future rule set for choosing transcode behavior by client, media, quality, or library.
_Avoid_: Hardware acceleration policy

**Remote Access Endpoint**:
A configured public or private URL through which clients or integrations reach Nako.
_Avoid_: Built-in NAT traversal

**Network Tunnel Provider**:
An external system that exposes Nako across networks.
_Avoid_: Nako relay

**User**:
A person or service identity that can access Nako.
_Avoid_: Global admin

**Role**:
A coarse permission set assigned to a **User**.
_Avoid_: Hard-coded user type

**Library Access**:
A user's allowed access to one or more **Media Libraries**.
_Avoid_: Global library visibility

**Single-Admin Mode**:
The first implementation mode where one administrator identity controls Nako.
_Avoid_: Permanent single-user model

**Client Application**:
Any user-facing application that consumes Nako APIs for browsing, playback, or library interaction.
_Avoid_: Flutter client

**Public Client API**:
A stable API surface intended for **Client Applications**.
_Avoid_: Internal admin route

**Admin API**:
An API surface for server administration, diagnostics, configuration, and operational workflows.
_Avoid_: Public client contract

**Generated Artifact**:
Output produced by an **Automation Provider** or **Addon** that Nako can inspect, store, accept, reject, or apply.
_Avoid_: Direct AI mutation

**Acceptance Workflow**:
The Nako-owned process that turns a **Generated Artifact** into canonical state.
_Avoid_: Implicit AI write

**Addon Permission**:
A coarse capability grant declared by an **Addon** and accepted by a user or administrator before the addon can perform protected actions.
_Avoid_: Field-level permission, implicit trust

**Library-Scoped Addon Grant**:
An **Addon Permission** limited to one or more media libraries.
_Avoid_: Global-only plugin permission

**Addon Side Effect**:
A protected change an **Addon** performs through Nako-owned APIs after receiving an **Addon Permission**.
_Avoid_: Unmediated mutation, direct database write

**Metadata Scrape**:
The act of collecting candidate metadata for a media item from local files, built-in providers, or addons.
_Avoid_: Metadata write

**Bulk Metadata Scrape**:
A user- or policy-initiated **Metadata Scrape** over a library, collection, or selected group of media items.
_Avoid_: Unbounded background scrape

**Playback Resource Suggestion**:
A play-related URL, subtitle, image, or similar resource proposed by an **Addon** for Nako to evaluate.
_Avoid_: Custom playback runtime, FFmpeg override

**Playback Runtime**:
Nako's owned boundary for playback sessions, streaming decisions, remuxing, transcoding, budgets, and playback errors.
_Avoid_: Addon player, stream plugin

**Jellyfin Plugin Compatibility**:
Compatibility with Jellyfin's plugin API or internal server object model.
_Avoid_: Addon Protocol

**Native Plugin**:
Extension code loaded into the Nako server process.
_Avoid_: Addon

**Automation Provider**:
An external provider that produces generated suggestions or artifacts for later user or policy acceptance.
_Avoid_: AI plugin, direct metadata writer

**Webhook Endpoint**:
A user-configured receiver for Nako event notifications.
_Avoid_: Addon callback

## Relationships

- An **Addon** declares an **Addon Version** for the sidecar implementation and an **Addon Protocol Version** for runtime compatibility.
- An **Addon Protocol Version** changes only when the compatibility contract changes.
- An **Addon** runs as an **Addon Sidecar** in the first implementation phase.
- An **Addon Sidecar** may call Nako APIs with an **Addon Token**.
- An **Addon** declares one or more **Addon Resources**.
- An **Addon** may declare **Addon Entry Points** for settings, tasks, item actions, admin actions, or diagnostics.
- An **Addon Entry Point** may link to an **Addon Hosted Page** for advanced workflows.
- An **Addon** may declare an **Addon Configuration Schema**.
- An **Addon** may declare **Addon Event Subscriptions**.
- An **Addon** may provide the execution logic for an **Addon Task**.
- An **Addon** declares coarse **Addon Permissions** before installation or enablement.
- An **Addon Permission** may be granted globally or narrowed by a **Library-Scoped Addon Grant**.
- An **Addon Side Effect** must pass through Nako-owned APIs, permissions, audit, and resource boundaries.
- An **Addon Token** carries only the **Addon Permissions** and library grants accepted for that addon.
- An **Addon Token** is long-lived in the first phase, but must be revocable and replaceable through **Addon Token Rotation**.
- An **Addon Event Subscription** uses Nako-owned event delivery; event-triggered writes still use an **Addon Token**.
- An **Addon Task** has a Nako-owned lifecycle, progress model, cancellation model, audit trail, and result boundary.
- An **Addon External Fetch** may be performed by an **Addon Sidecar**, but a **Nako-Managed Artifact** must enter Nako through Nako APIs.
- A **Library File Write** initiated by an **Addon** must go through Nako storage, NFO, artwork, or subtitle APIs.
- Nako stores and presents settings declared by an **Addon Configuration Schema**.
- An **Addon Hosted Page** is not trusted with Nako admin credentials.
- Sensitive **Addon Configuration Schema** fields store **Secret References**, not plaintext secret values.
- An **Addon** may participate in **Metadata Scrape** and **Bulk Metadata Scrape** workflows.
- An **Addon** may offer a Jellyfin-like extensibility experience without providing **Jellyfin Plugin Compatibility**.
- A **Bulk Metadata Scrape** may write canonical metadata when the **Addon** has the required **Addon Permission**.
- A **Playback Resource Suggestion** may influence playback options, but the **Playback Runtime** remains owned by Nako.
- An **Automation Provider** may produce suggestions, but it does not directly rewrite canonical metadata.
- A **Webhook Endpoint** receives event notifications from Nako; it is not an **Addon**.
- A **Native Plugin** is intentionally distinct from an **Addon** and is not the current extension model.
- An **Addon Manager** may automate addon installation and lifecycle later, but it is not required for the **Addon Protocol**.
- The first **Addon Manager** should focus on registry, permissions, token rotation, **Addon Health Check**, and **Addon Install Guide** behavior.
- The first **Addon Manager** should not directly manage container or process lifecycle.
- A **Media Library** contains many **Media Sources**.
- Nako's **Media Server Scope** is broader than the **Video-First Phase**.
- A **Media Library** has a **Media Domain** and may start from a **Library Preset**.
- A **Library Preset** sets defaults for naming, providers, local metadata policy, refresh behavior, and presentation.
- A **Library Preset** does not replace per-item **Media Item** kind or provider mapping.
- A **Media Source** has a **Source Locator** that is unique only within its **Media Library**.
- A **Source Fingerprint** may support a **Source Duplicate Relationship**, but it does not replace **Media Source** identity.
- A **Source Duplicate Relationship** preserves each source's library, file, metadata, permission, and playback context.
- Multiple **Media Sources** for one **Media Item** may represent **Source Variants**.
- **Source Variants** may span multiple **Media Libraries**, but each underlying **Media Source** keeps its library context.
- A **Source Variant Label** helps users distinguish variants but does not replace **Media Technical Facts**.
- An **Edition** is content-level and should not be treated as an ordinary **Source Variant**.
- An **Edition** should be represented as its own **Media Item** and connected through an **Edition Relationship**.
- **Playback Source Selection** chooses a playable source at runtime instead of relying on a permanent item-level default.
- **Playback Source Selection** must respect **Library Access**.
- A **Media Item** may be linked to one or more **Media Sources**.
- A **Source Duplicate Relationship** does not automatically merge sources into one **Media Item**.
- A **Media Item Hierarchy** is provider-neutral.
- **Local Inference** may create a **Provisional Hierarchy** during scanning.
- **Local Inference Evidence** should be preserved for search, diagnostics, and rematching.
- **Local Inference Evidence** records should include enough information to explain the inferred kind, title, year, season, episode, confidence, evidence source, and **Local Inference Version**.
- **Local Inference Evidence** is primarily owned by the **Media Source** that produced it.
- **Local Inference Evidence** represents the current inference snapshot for a source and inference version, not a scan-history log.
- **Local Inference Evidence** may reference inferred item or hierarchy targets without becoming canonical item metadata.
- **Local Inference** may seed **Canonical Metadata** only while an item remains provisional.
- **Local Inference** must not overwrite **Canonical Metadata** after **Hierarchy Confirmation**, accepted **Provider Mapping**, NFO authority, or user edits.
- A **Provisional Hierarchy** may be corrected by NFO, provider mapping, user edits, or accepted addon writes.
- **Hierarchy Confirmation** should update existing provisional items in place when possible.
- **Hierarchy Repair** is reserved for cases where the inferred hierarchy is structurally wrong.
- **Local Inference** should create an **Unknown Media Item** when classification evidence is weak.
- A **Provider Subject** is mapped to Nako through **Provider Mapping** rather than replacing **Media Item** identity.
- A **Media Item** may have many **Genres** and **Tags**.
- A **Tag** is not a **Genre** and does not change **Media Item** kind.
- An **Episode-Like Item** maps to an episode in the **Media Item Hierarchy**.
- An **Extra Item** stays outside the primary watch order.
- A **Franchise Collection** groups related items without forcing them into one season or series hierarchy.
- **Canonical Metadata** may come from NFO, local edits, or provider data according to **Metadata Source Priority**.
- **Canonical Metadata**, **Media Technical Facts**, and **User Library State** are separate concepts.
- **User Library State** separates shared **Library Item State** from per-user **User Playback State**.
- **Review Rating**, **Content Rating**, and **User Rating** are distinct concepts.
- **Browse Facets** and **Sort Keys** may be backed by **Canonical Metadata**, **Media Technical Facts**, **Library Item State**, or **User Playback State**.
- A **Core Metadata Field** may be modeled early when it is cross-domain or needed in the **Video-First Phase**.
- **Domain-Specific Metadata** should wait for the owning media domain unless a current feature needs it.
- **NFO Import** is supported by default.
- **NFO Export** is controlled by media-library configuration and uses **Library File Write** behavior.
- **NFO Export** should preserve unknown or third-party NFO content through **NFO Round Trip** when safe.
- An **Artwork Source** may be remote or local, but client presentation should use **Managed Artwork**.
- A **Media Item** may have many **Artwork Candidates**.
- **Selected Artwork** is chosen from available **Artwork Candidates** or imported local artwork.
- **Artwork Export** is controlled by media-library configuration and uses **Library File Write** behavior.
- A **Playback Transcode** belongs to playback session lifecycle and runtime budgets.
- An **Optimized Version** is a durable media asset with its own storage, quality, and cleanup policy.
- A **Playback Transcode** chooses hardware acceleration from a **Hardware Capability Report**.
- A **Hardware Capability Report** is created at startup or configuration change and may be updated by **Hardware Capability Refresh**.
- **Hardware Acceleration Policy** is global in the first implementation phase.
- **Transcode Profile** is a later feature, not the first hardware selection model.
- A **Remote Access Endpoint** may be backed by a **Network Tunnel Provider** or reverse proxy.
- Nako does not own **Network Tunnel Provider** behavior in the first implementation phase.
- **Single-Admin Mode** may be the first implementation mode, but it should not erase **User**, **Role**, or **Library Access** concepts.
- A **Client Application** may be implemented with any client technology.
- **Public Client API** should be versioned and more stable than **Admin API**.
- AI-like features produce **Generated Artifacts** through **Automation Providers** or **Addons**.
- A **Generated Artifact** becomes canonical state only through an **Acceptance Workflow** or an explicitly granted addon write path.

## Example Dialogue

> **Dev:** "Should this Jellyfin-like plugin run inside Nako?"
> **Domain expert:** "No. In Nako language that is an **Addon** if it uses the **Addon Protocol**. It should feel extensible like Jellyfin, but **Jellyfin Plugin Compatibility** is not the goal."
>
> **Dev:** "Can an **Addon** affect playback?"
> **Domain expert:** "Yes, by returning a **Playback Resource Suggestion**. The **Playback Runtime** still owns the session, budget, and error behavior."
>
> **Dev:** "Can an **Addon** scrape metadata for a whole library?"
> **Domain expert:** "Yes. With the right **Addon Permission**, it can perform a **Bulk Metadata Scrape** and write canonical metadata through Nako's APIs."
>
> **Dev:** "Should a metadata **Addon** be able to write every library?"
> **Domain expert:** "Only if it has a global grant. Prefer a **Library-Scoped Addon Grant** when the addon is meant for a specific library."
>
> **Dev:** "Can an **Addon** add UI actions?"
> **Domain expert:** "Yes, by declaring **Addon Entry Points**. Nako surfaces those entry points without executing arbitrary frontend plugin code."
>
> **Dev:** "Does Nako install and launch addons itself?"
> **Domain expert:** "Not in the first phase. An **Addon** is an **Addon Sidecar** registered through the **Addon Protocol**; an **Addon Manager** can come later."
>
> **Dev:** "Should Nako control Docker to install addons?"
> **Domain expert:** "Not first. Prefer **Addon Install Guide** output and **Addon Health Check** over Nako-managed container lifecycle."
>
> **Dev:** "Can an **Addon Sidecar** actively change Nako state?"
> **Domain expert:** "Yes, through Nako APIs using an **Addon Token** scoped to the accepted permissions and libraries."
>
> **Dev:** "Does the first Addon auth model require OAuth?"
> **Domain expert:** "No. Use a revocable long-lived **Addon Token** with **Addon Token Rotation** first."
>
> **Dev:** "Can we change the Addon manifest shape in place?"
> **Domain expert:** "Only for compatible additions. Breaking contract changes require a new **Addon Protocol Version**."
>
> **Dev:** "Can an **Addon** react when a scan finishes?"
> **Domain expert:** "Yes, by declaring an **Addon Event Subscription**. Nako delivers the event, and the addon uses an **Addon Token** for any follow-up writes."
>
> **Dev:** "Who owns a full-library addon scrape task?"
> **Domain expert:** "It is an **Addon Task**: Nako owns the job lifecycle and the **Addon Sidecar** owns the execution logic."
>
> **Dev:** "Can an **Addon** download artwork itself?"
> **Domain expert:** "It may perform an **Addon External Fetch**, but if the result becomes library artwork, it must be submitted as a **Nako-Managed Artifact** through Nako APIs."
>
> **Dev:** "Can an **Addon** write poster.jpg next to a movie file?"
> **Domain expert:** "Only through a **Library File Write** API owned by Nako, not by writing the path directly."
>
> **Dev:** "Can an **Addon** provide settings?"
> **Domain expert:** "Yes, by declaring an **Addon Configuration Schema** that Nako stores and renders."
>
> **Dev:** "Can an **Addon** expose its own diagnostics page?"
> **Domain expert:** "Yes, as an **Addon Hosted Page**, but Nako does not treat that page as trusted admin UI."
>
> **Dev:** "Can Addon settings store an API key?"
> **Domain expert:** "Store a **Secret Reference** instead; Nako resolves the secret at runtime."
>
> **Dev:** "Are hard-linked files in two libraries the same **Media Source**?"
> **Domain expert:** "No. They are separate **Media Sources** connected by a **Source Duplicate Relationship** when the evidence supports it."
>
> **Dev:** "Is a 4K file a different movie from a 1080p file?"
> **Domain expert:** "Usually no. They are **Source Variants** for one **Media Item**. A director's cut is an **Edition**, not just a source variant."
>
> **Dev:** "Can the UI show a version name like 4K HDR?"
> **Domain expert:** "Yes, through a **Source Variant Label**, while playback decisions still use **Media Technical Facts**."
>
> **Dev:** "Can one item have variants from different libraries?"
> **Domain expert:** "Yes, but each **Media Source** keeps its library context and **Playback Source Selection** only uses sources allowed by **Library Access**."
>
> **Dev:** "Should the director's cut live inside the theatrical item?"
> **Domain expert:** "No. It should be its own **Media Item** linked by an **Edition Relationship**."
>
> **Dev:** "Is a photo library a different database model from a movie library?"
> **Domain expert:** "No. It is a **Media Library** with a different **Media Domain** and **Library Preset**."
>
> **Dev:** "Is Nako only a video server?"
> **Domain expert:** "No. Nako is in a **Video-First Phase**, but its **Media Server Scope** should remain broader."
>
> **Dev:** "Do duplicate sources automatically become one item?"
> **Domain expert:** "No. A **Media Item** may link to multiple sources, but automatic merging needs a separate high-confidence rule."
>
> **Dev:** "Should Bangumi subjects create a separate anime item model?"
> **Domain expert:** "No. Map **Provider Subjects** into Nako's provider-neutral **Media Item Hierarchy** through **Provider Mapping**."
>
> **Dev:** "Can Nako show a series before provider matching?"
> **Domain expert:** "Yes, through **Local Inference** creating a **Provisional Hierarchy** from path and file-name evidence."
>
> **Dev:** "Should Nako guess a movie when it is unsure?"
> **Domain expert:** "No. Weak evidence should produce an **Unknown Media Item** instead of a confident but wrong classification."
>
> **Dev:** "If provider metadata renames a locally inferred series, do we lose the local name?"
> **Domain expert:** "No. Canonical metadata may change, but keep the local name as **Local Inference Evidence**."
>
> **Dev:** "Can inference evidence be transient?"
> **Domain expert:** "No. Persist enough **Local Inference Evidence** to explain, diagnose, and rerun inference when the **Local Inference Version** changes."
>
> **Dev:** "Does inference evidence belong to the item or the source?"
> **Domain expert:** "Primarily the **Media Source**, because the evidence comes from source path, file name, local files, and probe facts."
>
> **Dev:** "Can local inference overwrite canonical metadata on rescan?"
> **Domain expert:** "Only while the item is provisional. After confirmation, rescan updates source state and **Local Inference Evidence**, not canonical fields."
>
> **Dev:** "Should provider matching replace local provisional items?"
> **Domain expert:** "No. Use **Hierarchy Confirmation** to update provisional items in place unless **Hierarchy Repair** is required."
>
> **Dev:** "Can Bangumi tags be stored?"
> **Domain expert:** "Yes, as **Tags**. Keep **Genres** for broader category browsing and do not use tags as item identity."
>
> **Dev:** "Is an anime movie part of a season?"
> **Domain expert:** "Usually no. Keep it as a movie and relate it through a **Franchise Collection** unless provider evidence says it is an **Episode-Like Item**."
>
> **Dev:** "If TMDB and NFO disagree, which one wins?"
> **Domain expert:** "Use **Metadata Source Priority**. Nako should default to local and NFO first, then provider fallback, with per-library overrides if needed."
>
> **Dev:** "Is playback progress metadata?"
> **Domain expert:** "No. It is **User Library State**. Codec and bitrate are **Media Technical Facts**. Title and overview are **Canonical Metadata**."
>
> **Dev:** "Is watched status global?"
> **Domain expert:** "No. Watched status and progress are **User Playback State**; date added is **Library Item State**."
>
> **Dev:** "Can one rating field store both score and age guidance?"
> **Domain expert:** "No. Use **Review Rating** for scores, **Content Rating** for age guidance, and **User Rating** for per-user scores."
>
> **Dev:** "Can the client filter by every database column?"
> **Domain expert:** "No. It should use supported **Browse Facets** and **Sort Keys**."
>
> **Dev:** "Should album track number be in the first video metadata model?"
> **Domain expert:** "No. Treat it as **Domain-Specific Metadata** until the music domain owns it."
>
> **Dev:** "Does Nako always write NFO files?"
> **Domain expert:** "No. **NFO Import** is default, while **NFO Export** is enabled per library and must respect local file-write rules."
>
> **Dev:** "Can Nako replace the whole NFO with its own format?"
> **Domain expert:** "Avoid that. Use **NFO Round Trip** so unknown fields and third-party data survive when safe."
>
> **Dev:** "Should clients hotlink TMDB posters?"
> **Domain expert:** "No. Treat TMDB as an **Artwork Source** and serve **Managed Artwork** from Nako."
>
> **Dev:** "Can a movie have several poster options?"
> **Domain expert:** "Yes. Store them as **Artwork Candidates** and use **Selected Artwork** for the current presentation."
>
> **Dev:** "Is a pre-transcoded mobile copy just a finished playback transcode?"
> **Domain expert:** "No. A **Playback Transcode** is session-oriented; an **Optimized Version** is a durable media asset."
>
> **Dev:** "Should Nako probe GPU support before every playback?"
> **Domain expert:** "No. Use a cached **Hardware Capability Report** and refresh it explicitly when needed."
>
> **Dev:** "Should hardware selection vary by client in the first slice?"
> **Domain expert:** "No. Start with a global **Hardware Acceleration Policy** and add **Transcode Profiles** later."
>
> **Dev:** "Should Nako implement NAT traversal itself first?"
> **Domain expert:** "No. Start with **Remote Access Endpoints** backed by external **Network Tunnel Providers** or reverse proxies."
>
> **Dev:** "Can the MVP be single-user?"
> **Domain expert:** "Yes, as **Single-Admin Mode**, but the language should still preserve **User**, **Role**, and **Library Access** for later sharing."
>
> **Dev:** "Should service APIs assume Flutter?"
> **Domain expert:** "No. Treat Flutter, web, or native apps as **Client Applications** that consume **Public Client API** contracts."
>
> **Dev:** "Should Nako core own a local AI model runtime?"
> **Domain expert:** "No. AI-like features should enter as **Generated Artifacts** from **Automation Providers** or **Addons** first."

## Flagged Ambiguities

- "plugin" was used to mean a Jellyfin-like extension experience. Resolved: use **Addon** for Nako's current extension model, and reserve **Native Plugin** for in-process extension code.
- "similar to Jellyfin plugins" was used to mean user-facing extensibility rather than **Jellyfin Plugin Compatibility**.
- "Addon affects playback" means **Playback Resource Suggestion**, not replacement of the **Playback Runtime**.
- "batch scraping" was resolved as **Bulk Metadata Scrape** and may include writes when backed by explicit **Addon Permissions**.
- Addon permissions are coarse-grained, but may be narrowed per library through **Library-Scoped Addon Grant**.
- Addon UI integration means **Addon Entry Points**, not embedded client-side plugin execution.
- Addon-hosted settings or diagnostics use **Addon Hosted Pages** and must not receive Nako admin credentials.
- Addon installation and automatic lifecycle management belong to a future **Addon Manager**, not the first **Addon Protocol** slice.
- The first **Addon Manager** should not require Docker socket or process-supervision authority.
- Addon-initiated writes use an **Addon Token** and Nako APIs, not direct database or filesystem mutation.
- Addon authentication starts with revocable long-lived tokens and explicit rotation, not OAuth.
- Addon event automation uses **Addon Event Subscriptions**, not database polling or hidden API polling loops.
- Addon background work is an **Addon Task** when Nako users need lifecycle, progress, cancellation, or audit visibility.
- Addon downloads are allowed as **Addon External Fetches**; Nako-related outputs become **Nako-Managed Artifacts** only through Nako APIs.
- Addon writes to media library files use **Library File Write** APIs, not raw filesystem or remote-storage paths.
- Addon settings use an **Addon Configuration Schema** managed by Nako rather than opaque addon-owned UI state.
- Addon settings store **Secret References** for sensitive values, not plaintext secrets.
- Addon compatibility is governed by **Addon Protocol Version**, not by assuming every Nako server version accepts every addon.
- Duplicate content was resolved as **Source Duplicate Relationship**, not merged **Media Source** identity.
- Technical alternatives are **Source Variants**; different cuts are **Editions**.
- **Source Variant Labels** are display aids, not substitutes for technical facts.
- Cross-library **Source Variants** are allowed, but visibility and playback are constrained by **Library Access**.
- **Editions** are separate **Media Items** linked by **Edition Relationships**.
- Playback should use **Playback Source Selection**, not a permanent default source.
- Duplicate **Media Sources** do not automatically collapse into one **Media Item**.
- Nako is **Video-First** now, but should not hard-code a video-only **Media Server Scope**.
- Nako should grow toward multiple **Media Domains** through **Library Presets**, not through hard-coded library types.
- Provider-specific concepts use **Provider Mapping** and do not split the core **Media Item Hierarchy**.
- Local path and file-name parsing is **Local Inference**, not a metadata provider or scraper.
- Local inferred titles and structure are **Local Inference Evidence** that may differ from canonical metadata.
- **Local Inference Evidence** should be persisted minimally rather than thrown away after scanning.
- **Local Inference Evidence** is source-owned and may point at inferred hierarchy targets.
- **Local Inference Evidence** should be updated as a current snapshot; historical inference attempts need a separate history concept.
- **Local Inference** can seed provisional **Canonical Metadata** but must not replace confirmed canonical fields during rescan.
- Confirming provider or NFO data should normally use **Hierarchy Confirmation**, not replacement item creation.
- Structural mistakes from local inference use **Hierarchy Repair**.
- **Local Inference** should prefer **Unknown Media Item** over overconfident classification.
- Nako has **Tags** and **Genres**; tags are flexible labels, not item kinds or source identity.
- Anime specials map as **Episode-Like Items** only when they belong in watch order; otherwise they are **Extra Items**.
- Metadata resolution uses **Metadata Source Priority** with local and NFO data ahead of external providers by default.
- Filtering and sorting fields must distinguish **Canonical Metadata**, **Media Technical Facts**, and **User Library State**.
- Playback progress, watched status, favorites, hidden status, last played time, and user rating are **User Playback State**, not global item fields.
- Do not overload "rating"; distinguish **Review Rating**, **Content Rating**, and **User Rating**.
- Client browsing should use explicit **Browse Facets** and **Sort Keys**, not inferred database fields.
- Add **Core Metadata Fields** early, but keep **Domain-Specific Metadata** out of the video-first core unless a current feature requires it.
- NFO handling is bidirectional, but **NFO Export** is opt-in per library.
- NFO writes should use **NFO Round Trip** instead of destructive rewrites when existing NFO content is present.
- Artwork uses provider URLs as **Artwork Sources**, while **Managed Artwork** is the stable client-facing asset.
- Artwork modeling keeps multiple **Artwork Candidates** instead of collapsing to a single poster field.
- Transcoding distinguishes temporary **Playback Transcode** output from durable **Optimized Version** assets.
- Hardware acceleration selection uses a cached **Hardware Capability Report**, not an implicit probe for every playback.
- Hardware selection starts with global **Hardware Acceleration Policy**; **Transcode Profile** is deferred.
- Remote access starts with configured **Remote Access Endpoints**, not built-in NAT traversal or relay infrastructure.
- MVP authentication may use **Single-Admin Mode**, but the domain should not become permanently single-user.
- Client planning should target **Client Applications** generally, not a Flutter-specific API contract.
- AI integration starts through **Generated Artifacts** and **Acceptance Workflow**, not a core model runtime.
