---
name: resolve-go-token
description: Resolve a lewdzone.com go-link (#t=v1... token) into a real downloadable URL using the site's two-step api.php flow. Use when you need the actual file URL behind a download link.
---

# Resolve Go Token

Turn `https://lewdzone.com/go/#t=v1.<payload>.<sig>` into a real URL so FDM
can download the file.

## Why this is needed

FDM cannot use the go-link directly: the `#fragment` never reaches any server,
and feeding it to FDM would download a redirect artifact. The token MUST be
resolved here first.

## Flow (verified working)

```mermaid
sequenceDiagram
    participant App as lewdzone-launcher
    participant API as /go/api.php

    App->>App: extract token from go-link href
    App->>API: POST {"action":"start","token"}
    API-->>App: {"ok":true,"wait":3,"ticket":"...","host":...,"platform":...,"version":...,"game":...}
    App->>App: sleep(wait + 1)
    App->>API: POST {"action":"reveal","token","ticket"}
    alt ok
        API-->>App: {"ok":true,"url":"https://...zip/r"}
        App->>App: strip trailing \\r, validate host allowlist
        App-->>App: ResolvedUrl{url, host, platform, version, game_id}
    else retry
        API-->>App: {"retry_in": K}
        App->>App: sleep(K); retry (max 4)
    end
```

## Steps

1. **Extract** the `t=v1...` token from the go-link href (regex
   `#t=([^#]+)`).
2. **start**: POST `https://lewdzone.com/go/api.php` with
   Content-Type `application/json`, UA `Mozilla/5.0`, Referer
   `https://lewdzone.com/go/`. Body `{"action":"start","token":t}`.
   - If `ok:false` -> raise. If `captcha:1` -> surface to user (do not solve).
   - Keep `ticket`, and the free meta: `host`, `platform`, `version`, `game`.
3. **wait**: `sleep(wait + 1)`.
4. **reveal**: POST `{"action":"reveal","token":t,"ticket":ticket}`.
   - ok:take `url`, **strip trailing `\r`**.
   - `retry_in`: sleep that long, retry up to 4 times; then
     `RetryExhaustedError`.
5. **validate**: extracted host must be in the known host allowlist (from go.js
   ICONS). Scheme must be `http`/`https`. Then construct `ResolvedUrl`.

## Rules

- Never hand a `#t=` href to a download manager (see the `dm` family).
- Request headers are mandatory on both calls.
- The `u` payload field is obfuscated — never attempt to decode it; rely on
  start-response meta.

## Checkoffs

- [ ] Unit tests: ok path, `retry_in` retry, trailing `\r`, bad token, unknown host
- [ ] Live smoke (opt-in `#[ignore]`, `cargo test -- --ignored`) resolves one
      real token
- [ ] Resolver honors a global rate limiter