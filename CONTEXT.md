# Taru Context

Taru is a self-hosted media server backend. This glossary defines the project
language used when discussing media libraries, extension surfaces, automation,
and playback boundaries.

## Language

**Addon**:
A user-enabled extension that adds Taru capabilities outside the core server trust boundary while providing a Jellyfin-like extensibility experience.
_Avoid_: Plugin, Jellyfin plugin compatibility

**Addon Protocol**:
The compatibility contract an **Addon** follows so Taru can discover and call its capabilities.
_Avoid_: Plugin ABI, native plugin contract

**Addon Protocol Version**:
A declared compatibility version of the **Addon Protocol**.
_Avoid_: Taru server version

**Addon Sidecar**:
An independently running process or service that implements the **Addon Protocol**.
_Avoid_: In-process plugin

**Addon Token**:
A scoped credential that lets an **Addon Sidecar** call Taru APIs for granted capabilities.
_Avoid_: Server admin token, database credential

**Addon Token Rotation**:
The act of replacing an **Addon Token** without changing the addon registration or granted capabilities.
_Avoid_: OAuth flow

**Addon Event Subscription**:
A manifest-declared interest in Taru domain events that Taru delivers to an **Addon Sidecar**.
_Avoid_: Database polling, hidden scheduler

**Addon Task**:
A Taru-tracked job whose execution logic is provided by an **Addon Sidecar**.
_Avoid_: Hidden addon background job

**Addon External Fetch**:
Network or storage access an **Addon Sidecar** performs for its own execution.
_Avoid_: Taru-managed download

**Taru-Managed Artifact**:
Downloaded or generated content that Taru stores, indexes, serves, or treats as part of a library or playback experience.
_Avoid_: Addon private cache

**Library File Write**:
A change to files stored in a media library, such as subtitles, NFO files, artwork, or sidecar assets.
_Avoid_: Direct addon path write

**Addon Manager**:
Future Taru functionality for discovering, installing, updating, starting, or removing addons.
_Avoid_: Addon Protocol

**Addon Resource**:
A declared capability of an **Addon** that Taru may call after the user grants the required access.
_Avoid_: Plugin hook, arbitrary callback

**Addon Entry Point**:
A manifest-declared place where Taru surfaces an **Addon** action or view to a user.
_Avoid_: Embedded frontend plugin

**Addon Hosted Page**:
An external page served by an **Addon Sidecar** for advanced settings or diagnostics.
_Avoid_: Embedded trusted admin UI

**Addon Configuration Schema**:
A manifest-declared shape for settings Taru stores and presents for an **Addon**.
_Avoid_: Addon-owned settings UI state

**Secret Reference**:
A stored reference to a secret value resolved by Taru at runtime.
_Avoid_: Plaintext secret

**Media Library**:
A configured collection boundary that gives media sources shared storage, metadata, and permission context.
_Avoid_: Folder

**Media Domain**:
A broad processing capability class for media, such as video, audio, image, document, mixed, or online.
_Avoid_: Library type

**Media Server Scope**:
Taru's long-term product scope as a self-hosted server for video, audio, image, document, mixed, and online media.
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
The library-scoped address Taru uses to find a **Media Source**.
_Avoid_: Global path identity

**Source Fingerprint**:
Evidence used to compare whether two **Media Sources** may contain the same media bytes.
_Avoid_: Source identity

**Source Duplicate Relationship**:
A relationship indicating that multiple **Media Sources** likely refer to the same content.
_Avoid_: Merged source

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
The authoritative metadata Taru uses for browsing, search, playback presentation, and exports.
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
The ordered preference Taru uses when multiple metadata sources can fill the same field.
_Avoid_: Field-level permission

**NFO Import**:
Reading local NFO metadata into Taru's metadata pipeline.
_Avoid_: NFO sync

**NFO Export**:
Writing Taru metadata back to NFO sidecar files for a media library.
_Avoid_: Unconditional NFO overwrite

**NFO Round Trip**:
Preserving existing NFO content that Taru does not own while importing or exporting known metadata.
_Avoid_: Destructive NFO rewrite

**Artwork Source**:
The original provider, file, or addon reference from which artwork was discovered.
_Avoid_: Canonical artwork file

**Managed Artwork**:
Artwork stored or cached by Taru for stable browsing and client presentation.
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
A configured public or private URL through which clients or integrations reach Taru.
_Avoid_: Built-in NAT traversal

**Network Tunnel Provider**:
An external system that exposes Taru across networks.
_Avoid_: Taru relay

**User**:
A person or service identity that can access Taru.
_Avoid_: Global admin

**Role**:
A coarse permission set assigned to a **User**.
_Avoid_: Hard-coded user type

**Library Access**:
A user's allowed access to one or more **Media Libraries**.
_Avoid_: Global library visibility

**Single-Admin Mode**:
The first implementation mode where one administrator identity controls Taru.
_Avoid_: Permanent single-user model

**Client Application**:
Any user-facing application that consumes Taru APIs for browsing, playback, or library interaction.
_Avoid_: Flutter client

**Public Client API**:
A stable API surface intended for **Client Applications**.
_Avoid_: Internal admin route

**Admin API**:
An API surface for server administration, diagnostics, configuration, and operational workflows.
_Avoid_: Public client contract

**Generated Artifact**:
Output produced by an **Automation Provider** or **Addon** that Taru can inspect, store, accept, reject, or apply.
_Avoid_: Direct AI mutation

**Acceptance Workflow**:
The Taru-owned process that turns a **Generated Artifact** into canonical state.
_Avoid_: Implicit AI write

**Addon Permission**:
A coarse capability grant declared by an **Addon** and accepted by a user or administrator before the addon can perform protected actions.
_Avoid_: Field-level permission, implicit trust

**Library-Scoped Addon Grant**:
An **Addon Permission** limited to one or more media libraries.
_Avoid_: Global-only plugin permission

**Addon Side Effect**:
A protected change an **Addon** performs through Taru-owned APIs after receiving an **Addon Permission**.
_Avoid_: Unmediated mutation, direct database write

**Metadata Scrape**:
The act of collecting candidate metadata for a media item from local files, built-in providers, or addons.
_Avoid_: Metadata write

**Bulk Metadata Scrape**:
A user- or policy-initiated **Metadata Scrape** over a library, collection, or selected group of media items.
_Avoid_: Unbounded background scrape

**Playback Resource Suggestion**:
A play-related URL, subtitle, image, or similar resource proposed by an **Addon** for Taru to evaluate.
_Avoid_: Custom playback runtime, FFmpeg override

**Playback Runtime**:
Taru's owned boundary for playback sessions, streaming decisions, remuxing, transcoding, budgets, and playback errors.
_Avoid_: Addon player, stream plugin

**Jellyfin Plugin Compatibility**:
Compatibility with Jellyfin's plugin API or internal server object model.
_Avoid_: Addon Protocol

**Native Plugin**:
Extension code loaded into the Taru server process.
_Avoid_: Addon

**Automation Provider**:
An external provider that produces generated suggestions or artifacts for later user or policy acceptance.
_Avoid_: AI plugin, direct metadata writer

**Webhook Endpoint**:
A user-configured receiver for Taru event notifications.
_Avoid_: Addon callback

## Relationships

- An **Addon** conforms to exactly one **Addon Protocol** version.
- An **Addon Protocol Version** changes only when the compatibility contract changes.
- An **Addon** runs as an **Addon Sidecar** in the first implementation phase.
- An **Addon Sidecar** may call Taru APIs with an **Addon Token**.
- An **Addon** declares one or more **Addon Resources**.
- An **Addon** may declare **Addon Entry Points** for settings, tasks, item actions, admin actions, or diagnostics.
- An **Addon Entry Point** may link to an **Addon Hosted Page** for advanced workflows.
- An **Addon** may declare an **Addon Configuration Schema**.
- An **Addon** may declare **Addon Event Subscriptions**.
- An **Addon** may provide the execution logic for an **Addon Task**.
- An **Addon** declares coarse **Addon Permissions** before installation or enablement.
- An **Addon Permission** may be granted globally or narrowed by a **Library-Scoped Addon Grant**.
- An **Addon Side Effect** must pass through Taru-owned APIs, permissions, audit, and resource boundaries.
- An **Addon Token** carries only the **Addon Permissions** and library grants accepted for that addon.
- An **Addon Token** is long-lived in the first phase, but must be revocable and replaceable through **Addon Token Rotation**.
- An **Addon Event Subscription** uses Taru-owned event delivery; event-triggered writes still use an **Addon Token**.
- An **Addon Task** has a Taru-owned lifecycle, progress model, cancellation model, audit trail, and result boundary.
- An **Addon External Fetch** may be performed by an **Addon Sidecar**, but a **Taru-Managed Artifact** must enter Taru through Taru APIs.
- A **Library File Write** initiated by an **Addon** must go through Taru storage, NFO, artwork, or subtitle APIs.
- Taru stores and presents settings declared by an **Addon Configuration Schema**.
- An **Addon Hosted Page** is not trusted with Taru admin credentials.
- Sensitive **Addon Configuration Schema** fields store **Secret References**, not plaintext secret values.
- An **Addon** may participate in **Metadata Scrape** and **Bulk Metadata Scrape** workflows.
- An **Addon** may offer a Jellyfin-like extensibility experience without providing **Jellyfin Plugin Compatibility**.
- A **Bulk Metadata Scrape** may write canonical metadata when the **Addon** has the required **Addon Permission**.
- A **Playback Resource Suggestion** may influence playback options, but the **Playback Runtime** remains owned by Taru.
- An **Automation Provider** may produce suggestions, but it does not directly rewrite canonical metadata.
- A **Webhook Endpoint** receives event notifications from Taru; it is not an **Addon**.
- A **Native Plugin** is intentionally distinct from an **Addon** and is not the current extension model.
- An **Addon Manager** may automate addon installation and lifecycle later, but it is not required for the **Addon Protocol**.
- A **Media Library** contains many **Media Sources**.
- Taru's **Media Server Scope** is broader than the **Video-First Phase**.
- A **Media Library** has a **Media Domain** and may start from a **Library Preset**.
- A **Library Preset** sets defaults for naming, providers, local metadata policy, refresh behavior, and presentation.
- A **Library Preset** does not replace per-item **Media Item** kind or provider mapping.
- A **Media Source** has a **Source Locator** that is unique only within its **Media Library**.
- A **Source Fingerprint** may support a **Source Duplicate Relationship**, but it does not replace **Media Source** identity.
- A **Source Duplicate Relationship** preserves each source's library, file, metadata, permission, and playback context.
- A **Media Item** may be linked to one or more **Media Sources**.
- A **Source Duplicate Relationship** does not automatically merge sources into one **Media Item**.
- A **Media Item Hierarchy** is provider-neutral.
- A **Provider Subject** is mapped to Taru through **Provider Mapping** rather than replacing **Media Item** identity.
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
- Taru does not own **Network Tunnel Provider** behavior in the first implementation phase.
- **Single-Admin Mode** may be the first implementation mode, but it should not erase **User**, **Role**, or **Library Access** concepts.
- A **Client Application** may be implemented with any client technology.
- **Public Client API** should be versioned and more stable than **Admin API**.
- AI-like features produce **Generated Artifacts** through **Automation Providers** or **Addons**.
- A **Generated Artifact** becomes canonical state only through an **Acceptance Workflow** or an explicitly granted addon write path.

## Example Dialogue

> **Dev:** "Should this Jellyfin-like plugin run inside Taru?"
> **Domain expert:** "No. In Taru language that is an **Addon** if it uses the **Addon Protocol**. It should feel extensible like Jellyfin, but **Jellyfin Plugin Compatibility** is not the goal."
>
> **Dev:** "Can an **Addon** affect playback?"
> **Domain expert:** "Yes, by returning a **Playback Resource Suggestion**. The **Playback Runtime** still owns the session, budget, and error behavior."
>
> **Dev:** "Can an **Addon** scrape metadata for a whole library?"
> **Domain expert:** "Yes. With the right **Addon Permission**, it can perform a **Bulk Metadata Scrape** and write canonical metadata through Taru's APIs."
>
> **Dev:** "Should a metadata **Addon** be able to write every library?"
> **Domain expert:** "Only if it has a global grant. Prefer a **Library-Scoped Addon Grant** when the addon is meant for a specific library."
>
> **Dev:** "Can an **Addon** add UI actions?"
> **Domain expert:** "Yes, by declaring **Addon Entry Points**. Taru surfaces those entry points without executing arbitrary frontend plugin code."
>
> **Dev:** "Does Taru install and launch addons itself?"
> **Domain expert:** "Not in the first phase. An **Addon** is an **Addon Sidecar** registered through the **Addon Protocol**; an **Addon Manager** can come later."
>
> **Dev:** "Can an **Addon Sidecar** actively change Taru state?"
> **Domain expert:** "Yes, through Taru APIs using an **Addon Token** scoped to the accepted permissions and libraries."
>
> **Dev:** "Does the first Addon auth model require OAuth?"
> **Domain expert:** "No. Use a revocable long-lived **Addon Token** with **Addon Token Rotation** first."
>
> **Dev:** "Can we change the Addon manifest shape in place?"
> **Domain expert:** "Only for compatible additions. Breaking contract changes require a new **Addon Protocol Version**."
>
> **Dev:** "Can an **Addon** react when a scan finishes?"
> **Domain expert:** "Yes, by declaring an **Addon Event Subscription**. Taru delivers the event, and the addon uses an **Addon Token** for any follow-up writes."
>
> **Dev:** "Who owns a full-library addon scrape task?"
> **Domain expert:** "It is an **Addon Task**: Taru owns the job lifecycle and the **Addon Sidecar** owns the execution logic."
>
> **Dev:** "Can an **Addon** download artwork itself?"
> **Domain expert:** "It may perform an **Addon External Fetch**, but if the result becomes library artwork, it must be submitted as a **Taru-Managed Artifact** through Taru APIs."
>
> **Dev:** "Can an **Addon** write poster.jpg next to a movie file?"
> **Domain expert:** "Only through a **Library File Write** API owned by Taru, not by writing the path directly."
>
> **Dev:** "Can an **Addon** provide settings?"
> **Domain expert:** "Yes, by declaring an **Addon Configuration Schema** that Taru stores and renders."
>
> **Dev:** "Can an **Addon** expose its own diagnostics page?"
> **Domain expert:** "Yes, as an **Addon Hosted Page**, but Taru does not treat that page as trusted admin UI."
>
> **Dev:** "Can Addon settings store an API key?"
> **Domain expert:** "Store a **Secret Reference** instead; Taru resolves the secret at runtime."
>
> **Dev:** "Are hard-linked files in two libraries the same **Media Source**?"
> **Domain expert:** "No. They are separate **Media Sources** connected by a **Source Duplicate Relationship** when the evidence supports it."
>
> **Dev:** "Is a photo library a different database model from a movie library?"
> **Domain expert:** "No. It is a **Media Library** with a different **Media Domain** and **Library Preset**."
>
> **Dev:** "Is Taru only a video server?"
> **Domain expert:** "No. Taru is in a **Video-First Phase**, but its **Media Server Scope** should remain broader."
>
> **Dev:** "Do duplicate sources automatically become one item?"
> **Domain expert:** "No. A **Media Item** may link to multiple sources, but automatic merging needs a separate high-confidence rule."
>
> **Dev:** "Should Bangumi subjects create a separate anime item model?"
> **Domain expert:** "No. Map **Provider Subjects** into Taru's provider-neutral **Media Item Hierarchy** through **Provider Mapping**."
>
> **Dev:** "Can Bangumi tags be stored?"
> **Domain expert:** "Yes, as **Tags**. Keep **Genres** for broader category browsing and do not use tags as item identity."
>
> **Dev:** "Is an anime movie part of a season?"
> **Domain expert:** "Usually no. Keep it as a movie and relate it through a **Franchise Collection** unless provider evidence says it is an **Episode-Like Item**."
>
> **Dev:** "If TMDB and NFO disagree, which one wins?"
> **Domain expert:** "Use **Metadata Source Priority**. Taru should default to local and NFO first, then provider fallback, with per-library overrides if needed."
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
> **Dev:** "Does Taru always write NFO files?"
> **Domain expert:** "No. **NFO Import** is default, while **NFO Export** is enabled per library and must respect local file-write rules."
>
> **Dev:** "Can Taru replace the whole NFO with its own format?"
> **Domain expert:** "Avoid that. Use **NFO Round Trip** so unknown fields and third-party data survive when safe."
>
> **Dev:** "Should clients hotlink TMDB posters?"
> **Domain expert:** "No. Treat TMDB as an **Artwork Source** and serve **Managed Artwork** from Taru."
>
> **Dev:** "Can a movie have several poster options?"
> **Domain expert:** "Yes. Store them as **Artwork Candidates** and use **Selected Artwork** for the current presentation."
>
> **Dev:** "Is a pre-transcoded mobile copy just a finished playback transcode?"
> **Domain expert:** "No. A **Playback Transcode** is session-oriented; an **Optimized Version** is a durable media asset."
>
> **Dev:** "Should Taru probe GPU support before every playback?"
> **Domain expert:** "No. Use a cached **Hardware Capability Report** and refresh it explicitly when needed."
>
> **Dev:** "Should hardware selection vary by client in the first slice?"
> **Domain expert:** "No. Start with a global **Hardware Acceleration Policy** and add **Transcode Profiles** later."
>
> **Dev:** "Should Taru implement NAT traversal itself first?"
> **Domain expert:** "No. Start with **Remote Access Endpoints** backed by external **Network Tunnel Providers** or reverse proxies."
>
> **Dev:** "Can the MVP be single-user?"
> **Domain expert:** "Yes, as **Single-Admin Mode**, but the language should still preserve **User**, **Role**, and **Library Access** for later sharing."
>
> **Dev:** "Should service APIs assume Flutter?"
> **Domain expert:** "No. Treat Flutter, web, or native apps as **Client Applications** that consume **Public Client API** contracts."
>
> **Dev:** "Should Taru core own a local AI model runtime?"
> **Domain expert:** "No. AI-like features should enter as **Generated Artifacts** from **Automation Providers** or **Addons** first."

## Flagged Ambiguities

- "plugin" was used to mean a Jellyfin-like extension experience. Resolved: use **Addon** for Taru's current extension model, and reserve **Native Plugin** for in-process extension code.
- "similar to Jellyfin plugins" was used to mean user-facing extensibility rather than **Jellyfin Plugin Compatibility**.
- "Addon affects playback" means **Playback Resource Suggestion**, not replacement of the **Playback Runtime**.
- "batch scraping" was resolved as **Bulk Metadata Scrape** and may include writes when backed by explicit **Addon Permissions**.
- Addon permissions are coarse-grained, but may be narrowed per library through **Library-Scoped Addon Grant**.
- Addon UI integration means **Addon Entry Points**, not embedded client-side plugin execution.
- Addon-hosted settings or diagnostics use **Addon Hosted Pages** and must not receive Taru admin credentials.
- Addon installation and automatic lifecycle management belong to a future **Addon Manager**, not the first **Addon Protocol** slice.
- Addon-initiated writes use an **Addon Token** and Taru APIs, not direct database or filesystem mutation.
- Addon authentication starts with revocable long-lived tokens and explicit rotation, not OAuth.
- Addon event automation uses **Addon Event Subscriptions**, not database polling or hidden API polling loops.
- Addon background work is an **Addon Task** when Taru users need lifecycle, progress, cancellation, or audit visibility.
- Addon downloads are allowed as **Addon External Fetches**; Taru-related outputs become **Taru-Managed Artifacts** only through Taru APIs.
- Addon writes to media library files use **Library File Write** APIs, not raw filesystem or remote-storage paths.
- Addon settings use an **Addon Configuration Schema** managed by Taru rather than opaque addon-owned UI state.
- Addon settings store **Secret References** for sensitive values, not plaintext secrets.
- Addon compatibility is governed by **Addon Protocol Version**, not by assuming every Taru server version accepts every addon.
- Duplicate content was resolved as **Source Duplicate Relationship**, not merged **Media Source** identity.
- Duplicate **Media Sources** do not automatically collapse into one **Media Item**.
- Taru is **Video-First** now, but should not hard-code a video-only **Media Server Scope**.
- Taru should grow toward multiple **Media Domains** through **Library Presets**, not through hard-coded library types.
- Provider-specific concepts use **Provider Mapping** and do not split the core **Media Item Hierarchy**.
- Taru has **Tags** and **Genres**; tags are flexible labels, not item kinds or source identity.
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
