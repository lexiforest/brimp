import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://docs.brimp.ai',
  integrations: [
    starlight({
      title: 'Brimp',
      description: 'Scalable browser runtime for agents.',
      customCss: ['./src/styles/custom.css'],
      editLink: {
        baseUrl: 'https://github.com/lexiforest/brimp/edit/main/docs/',
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/lexiforest/brimp',
        },
      ],
      sidebar: [
        { label: 'Introduction', slug: 'introduction' },
        { label: 'Install', slug: 'install' },
        { label: 'Quick start', slug: 'quick-start' },
        {
          label: 'Tutorials',
          items: [
            { label: 'Overview', slug: 'examples' },
            { label: 'Python', slug: 'examples/python' },
            { label: 'Node.js', slug: 'examples/node' },
            { label: 'CLI', slug: 'examples/cli' },
            { label: 'Extract, intercept and stream', slug: 'examples/fetch' },
            { label: 'Add a plugin to a page', slug: 'examples/plugins' },
            { label: 'CLI workflows', slug: 'examples/cli-workflows' },
            { label: 'Debug page scripts', slug: 'examples/debugging' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Sessions, pages and requests', slug: 'guides/automation' },
            { label: 'Values and handles', slug: 'guides/values' },
            { label: 'Plugins', slug: 'guides/plugins' },
            { label: 'Browser profiles', slug: 'guides/profiles' },
            { label: 'Diagnostics and time', slug: 'guides/diagnostics' },
          ],
        },
        {
          label: 'Browser support',
          items: [
            { label: 'Overview', slug: 'web-platform' },
            { label: 'DOM and documents', slug: 'web-platform/dom' },
            { label: 'Forms and input', slug: 'web-platform/forms' },
            { label: 'Events', slug: 'web-platform/events' },
            { label: 'Window and environment', slug: 'web-platform/environment' },
            { label: 'CSS, SVG and geometry', slug: 'web-platform/css-svg' },
            { label: 'Network, files and streams', slug: 'web-platform/network-files' },
            { label: 'Cryptography', slug: 'web-platform/crypto' },
            { label: 'Browser plugins', slug: 'web-platform/plugins' },
          ],
        },
        {
          label: 'API',
          items: [
            { label: 'Overview', slug: 'api' },
            { label: 'Python reference', link: '/reference/python/' },
            { label: 'Node reference', link: '/reference/node/' },
            { label: 'CLI reference', slug: 'api/cli' },
          ],
        },
        {
          label: 'Architecture',
          items: [
            { label: 'Follow a page', slug: 'architecture/javascript-runtime' },
            { label: 'Subsystem responsibilities', slug: 'architecture/subsystems' },
          ],
        },
        {
          label: 'Development',
          items: [
            { label: 'Build from source', slug: 'development/build-from-source' },
            { label: 'Build and test', slug: 'development' },
            { label: 'Package native bindings', slug: 'development/packaging' },
            { label: 'Network transport', slug: 'development/network' },
          ],
        },
      ],
    }),
  ],
});
