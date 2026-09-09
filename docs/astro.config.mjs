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
            { label: 'Fetch and extract', slug: 'examples/fetch' },
            { label: 'Crawl a website', slug: 'examples/crawl' },
            { label: 'CLI and CDP', slug: 'examples/cli-and-cdp' },
          ],
        },
        {
          label: 'API',
          items: [
            { label: 'Overview', slug: 'api' },
            { label: 'CLI', slug: 'api/cli' },
            { label: 'CDP', slug: 'api/cdp' },
          ],
        },
        {
          label: 'Architecture',
          items: [
            { label: 'JavaScriptCore integration', slug: 'architecture/javascript-runtime' },
            { label: 'Subsystem implementation', slug: 'architecture/subsystems' },
          ],
        },
        { label: 'Development', slug: 'development' },
      ],
    }),
  ],
});
