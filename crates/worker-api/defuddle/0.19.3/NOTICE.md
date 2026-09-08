# Defuddle browser bundle

Brimp vendors `dist/index.full.js` from the `defuddle@0.19.3` npm package.
The bundle is evaluated locally in a page realm; release builds do not invoke
npm and do not download runtime code.

- Source: <https://github.com/kepano/defuddle/tree/0.19.3>
- Source commit: `a332b4d5d539066ddfe19fc4ef6f1b6ffaf914b8`
- npm package integrity:
  `sha512-5ZbOQ/B+iiRRqSQWwmCx/zEuqZOA/5q7gxyE2/4O2Bxq7nUC0PgKw9EdyxqWbFF1nrrrYemSeR25jg4TmA2fsA==`
- npm tarball SHA-256:
  `5ee0e894b27f8342975f7acbbb96dd31b79baa0e2f1bba47d0d25f16cc49d153`
- `index.full.js` SHA-256:
  `50ac3cec17c11139833a05cf0f61a812f89c06019c3af190284b49c093091294`

The full web bundle includes these packages from Defuddle's lockfile:

| Package | Version | License |
| --- | --- | --- |
| Defuddle | 0.19.3 | MIT |
| MathML-to-LaTeX | 1.8.0 | MIT |
| @xmldom/xmldom | 0.9.10 | MIT |
| Temml | 0.13.3 | MIT |
| Turndown | 7.2.0 | MIT |

Their complete license texts are in `licenses/`. Defuddle's Node CLI dependency
(`commander`) and optional server DOM (`linkedom`) are not present in this web
bundle. Turndown's browser build excludes its Node-only Domino dependency.
