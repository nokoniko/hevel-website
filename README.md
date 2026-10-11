
# Hevel website

- `src/` — components, routing and state (signals, `Routable` enum, typed props)
- `src/hooks.rs` — `latest_release` server function: runs **at build time**, so the latest GitHub
  release version and DMG link are baked into the HTML
- `src/main.rs` — `static_routes` server function lists the routes to pre-render
- `src/web/` — browser-only code (Web Audio player, canvas cover colours, DOM helpers). It only runs
  in effects/event handlers, never during rendering on the server.

## Error pages

`src/pages/error.rs` has one `ErrorPage` for every status code in `ERROR_CODES`
(400, 401, 403, 404, 405, 408, 410, 429, 500, 502, 503, 504). Each one is pre-rendered at
`/error/<code>`, so any host can point its error pages there (nginx `error_page`, Cloudflare, …).
Add a code by adding a line to `ERROR_CODES`.

- Unknown addresses render the 404 page and, when a server renders them, answer with a real `404` status.
- On Vercel, `build.sh` copies `/error/404` to `404.html`, which Vercel serves for every missing address.
  Vercel can’t be given custom pages for the other codes (its own 5xx pages are fixed).

## Develop

```sh
dx serve
```

## Build and deploy (Vercel)

```sh
./build.sh                   # pre-renders every route into ./dist
vercel deploy dist --prod    # upload the static folder
```

The release version is read when you build, so rebuild and redeploy after publishing a new release.
