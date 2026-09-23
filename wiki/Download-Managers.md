# 📥 Downloads & In-App Streaming

> Links between wiki pages are relative and omit the `.md` extension.

## 🧠 Model

LewdZone Launcher resolves a go-link token into a **real URL**, then dispatches
it based on the host class:

- **Direct-file hosts** (currently `fileknot`) are **streamed inside the app**:
  the file downloads directly to `<DownloadRoot>/Games/<Title>/` with live byte
  progress on the **Downloads** page.
- **Everything else** (cloud apps, page-gated hosts) hands the resolved URL to
  the **OS default handler**: the installed desktop app for the service
  (MEGA, Google Drive, Dropbox, OneDrive, ...) or the browser otherwise. No
  configuration needed — no download manager to install, detect, or select.

The download-manager layer (FDM / IDM / torrent adapters) was removed: DM
detection was unreliable and every host has a free in-app or native path.

## 🚦 Dispatch rules

1. The reveal URL is **not** the final file URL — it is the *file host's page*
   (the go-button leaves for the file host; only `fileknot` returns a direct
   zip). Cloud/page hosts open in their native handler.
2. Non-2xx redirects are followed manually, **same-owner only**: a redirect may
   stay on the host or move to a dot-boundary subdomain of it — anything else is
   refused and the download fails (never routed to a foreign host).
3. At most 3 redirect hops per stream; retried (3×, backoff) like every polite
   network call (Rule 05).
4. Only a **resolved** real URL is ever acted on (see [Security](Security)).
   The go-link `#fragment` is never passed anywhere.

## 📂 Folder folding

Downloads land in:
`<DownloadRoot>/Games/<Title>/<Title> - <Version> - <Platform>[- <Variant>].<ext>`

- Never overwrite an existing file → append `(N)`.
- Path sanitization strips `\ : * ? " < > |` (and `/` where needed).

## 🧵 Queue

Downloads run one at a time through the scheduler (see
[Configuration](Configuration) `download-grace-seconds`). Statuses:
`queued → resolving → dispatching → downloading → dispatched`, or `failed`.

## 🔗 See also

- [Configuration](Configuration) — `download-root`, `source-priority`
- [Security](Security) — the host allowlist and redirect validation
- [CLI Reference](CLI-Reference) — `lewdzone download`