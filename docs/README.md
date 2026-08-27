# Brimp documentation

The public documentation site is built with Astro Starlight and configured for
`https://docs.brimp.ai`.

```sh
npm ci
npm run dev
```

Create a production build with:

```sh
npm run build
```

Documentation pages live in `src/content/docs/`. Keep the interface reference
aligned with the support matrices and public types in the repository root.

## GitHub Pages

The `docs-pages.yml` workflow builds and deploys this directory. In the GitHub
repository, select **Settings → Pages → GitHub Actions**, set the custom domain
to `docs.brimp.ai`, and enable HTTPS. At the DNS provider, create:

```text
Type:   CNAME
Name:   docs
Target: lexiforest.github.io
```

GitHub Actions deployments do not use a checked-in `CNAME` file; the custom
domain is stored in the repository's Pages settings.
