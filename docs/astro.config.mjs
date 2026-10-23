import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://docs.brimp.ai',
  integrations: [
    starlight({
      title: 'Brimp',
      description: 'A JavaScript runtime for JavaScript from the browser.',
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
            { label: 'CLI workflows', slug: 'examples/cli-workflows' },
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
            { label: 'Core APIs and plugins', slug: 'architecture/subsystems' },
          ],
        },
        { label: 'Development', slug: 'development' },
      ],
    }),
  ],
});
