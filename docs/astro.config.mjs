import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightUtils from '@lorenzo_lewis/starlight-utils';

const base = process.env.ASTRO_BASE ?? '/';
const site = process.env.ASTRO_SITE ?? 'https://vide.pascal-lab.net';

export default defineConfig({
  site,
  base,
  integrations: [
    starlight({
      title: {
        'zh-CN': 'VIDE',
        en: 'Vide Docs',
      },
      description:
        'Documentation for the Vide Verilog/SystemVerilog language server, VS Code extension, and playground.',
      locales: {
        root: {
          label: '简体中文',
          lang: 'zh-CN',
        },
        en: {
          label: 'English',
          lang: 'en',
        },
      },
      defaultLocale: 'root',
      editLink: {
        baseUrl: 'https://github.com/pascal-lab/vide/edit/master/docs/',
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/pascal-lab/vide',
        },
      ],
      components: {
        Footer: './src/components/SiteFooter.astro',
      },
      customCss: ['./src/assets/landing.css'],
      components: {
        Sidebar: './src/components/Sidebar.astro',
      },
      plugins: [
        starlightUtils({
          multiSidebar: {
            switcherStyle: 'hidden',
          },
          navLinks: {
            leading: { useSidebarLabelled: 'Header' },
          },
        }),
      ],
      routeMiddleware: './src/starlightRouteData.ts',
      sidebar: [
        {
          label: '用户手册',
          translations: { en: 'User Guide' },
          items: [
            'user-guide',
            'user-guide/online-experience',
            'user-guide/vscode-installation',
            'user-guide/first-project',
            {
              label: '功能特性',
              translations: { en: 'Features' },
              items: [
                'user-guide/daily-use',
                'user-guide/daily-use/navigation',
                'user-guide/daily-use/references',
                'user-guide/daily-use/hover',
                'user-guide/daily-use/completion',
                'user-guide/daily-use/rename',
                'user-guide/daily-use/syntax-highlighting',
                'user-guide/daily-use/semantic-highlighting',
                'user-guide/daily-use/inlay-hints',
                'user-guide/daily-use/document-symbols',
                'user-guide/daily-use/folding',
                'user-guide/daily-use/quick-fixes',
                'user-guide/daily-use/diagnostics',
                'user-guide/daily-use/signature-help',
                'user-guide/daily-use/selection-range',
                'user-guide/daily-use/formatting',
                'user-guide/daily-use/qihe',
              ],
            },
            {
              label: '参考',
              translations: { en: 'Reference' },
              items: ['user-guide/project-configuration', 'user-guide/vscode-settings'],
            },
          ],
        },
        {
          label: '进阶',
          translations: { en: 'Advanced' },
          items: [
            'advanced-guide',
            'advanced-guide/advanced-installation',
            'advanced-guide/build-from-source',
            'advanced-guide/check-server',
            'advanced-guide/commands-status-logs',
            'advanced-guide/troubleshooting',
            'advanced-guide/parsing-and-analysis',
          ],
        },
        {
          label: 'Changelog',
          translations: { en: 'Changelog' },
          items: [
            'changelog',
            'changelog/v0-1-5',
            'changelog/v0-1-4',
            'changelog/v0-1-3',
            'changelog/v0-1-2',
            'changelog/v0-1-1',
          ],
        },
        {
          label: 'Playground',
          translations: { en: 'Playground' },
          items: ['playground'],
        },
        {
          label: 'Header',
          items: [
            {
              label: '用户手册',
              translations: { en: 'User Guide' },
              link: '/user-guide/',
            },
            {
              label: '进阶',
              translations: { en: 'Advanced' },
              link: '/advanced-guide/',
            },
            {
              label: '更新日志',
              translations: { en: 'Changelog' },
              link: '/changelog/',
            },
            {
              label: 'Playground',
              translations: { en: 'Playground' },
              link: '/playground/',
            },
          ],
        },
      ],
    }),
  ],
});
