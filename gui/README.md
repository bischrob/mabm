# MABM GUI (Vite + React)

This UI reads run manifests from `outputs/run_index.json` and can trigger model runs by invoking the Rust CLI backend.

## Start API

From `gui/`:

```bash
npm install
npm run api
```

## Start UI

In another terminal from `gui/`:

```bash
npm run dev
```

The UI proxies `/api/*` to `http://localhost:8787`.
