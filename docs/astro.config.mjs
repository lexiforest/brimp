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
          label: 'Examples',
          items: [
            { label: 'Overview', slug: 'examples' },
            { label: 'Fetch and extract', slug: 'examples/fetch' },
          ],
        },
        {
          label: 'API',
          items: [
            { label: 'Overview', slug: 'api' },
          ],
        },
        {
          label: 'Architecture',
          items: [
            { label: 'JavaScriptCore integration', slug: 'architecture/javascript-runtime' },
            { label: 'Core APIs and plugins', slug: 'architecture/subsystems' },
          ],
        },
        { label: 'Development', slug: 'development' },
      ],
    }),
  ],
});
