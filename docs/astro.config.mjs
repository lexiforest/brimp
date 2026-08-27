import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://docs.brimp.ai',
  integrations: [
    starlight({
      title: 'Brimp',
      description: 'A lightweight, headless browser for agents.',
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
          label: 'Examples',
          items: [
            { label: 'Overview', slug: 'examples' },
            { label: 'Python', slug: 'examples/python' },
            { label: 'Node.js', slug: 'examples/node' },
            { label: 'CLI and CDP', slug: 'examples/cli-and-cdp' },
          ],
        },
        {
          label: 'API',
          items: [
            { label: 'Overview', slug: 'api' },
            { label: 'Python', slug: 'api/python' },
            { label: 'Node.js', slug: 'api/node' },
            { label: 'CLI', slug: 'api/cli' },
            { label: 'CDP', slug: 'api/cdp' },
          ],
        },
        { label: 'Development', slug: 'development' },
      ],
    }),
  ],
});
