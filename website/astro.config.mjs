import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLlmsTxt from 'starlight-llms-txt';
import { sqlcleanDark, sqlcleanLight } from './src/code-themes.mjs';
import { redirects, sidebar } from './src/navigation.mjs';

export default defineConfig({
	site: 'https://sqlclean.io',
	redirects,
	integrations: [
		starlight({
			title: 'SQLClean',
			description: 'Fast SQL linting, safe fixes and one canonical format.',
			logo: {
				light: './src/assets/logo-light.png',
				dark: './src/assets/logo-dark.png',
				replacesTitle: true,
			},
			favicon: '/favicon.svg',
			head: [
				// PNG fallback for browsers without SVG favicons; the SVG favicon uses dark ink on light tabs and white on dark.
				{ tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '128x128', href: '/favicon.png' } },
				{ tag: 'meta', attrs: { property: 'og:image', content: 'https://sqlclean.io/assets/og-sqlclean-docs.png' } },
				{ tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
				{ tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
				{ tag: 'meta', attrs: { name: 'twitter:image', content: 'https://sqlclean.io/assets/og-sqlclean-docs.png' } },
			],
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/sqlstack-io/sqlclean' },
			],
			components: {
				PageFrame: './src/components/PageFrame.astro',
				TwoColumnContent: './src/components/TwoColumnContent.astro',
				Header: './src/components/Header.astro',
				PageTitle: './src/components/PageTitle.astro',
				SiteTitle: './src/components/SiteTitle.astro',
				Pagination: './src/components/Pagination.astro',
				ThemeSelect: './src/components/ThemeSelect.astro',
			},
			customCss: ['./src/styles/fonts.css', './src/styles/sqlclean.css'],
			expressiveCode: {
				themes: [sqlcleanDark, sqlcleanLight],
				defaultProps: {
					// Plain boxes for shell snippets, without the fake terminal title bar.
					overridesByLang: { 'bash,sh,shell,zsh,console,powershell': { frame: 'none' } },
				},
				styleOverrides: {
					borderRadius: '2px',
					borderWidth: '1px',
					codeFontFamily: 'var(--sl-font-mono)',
					codeFontSize: '13px',
					codeLineHeight: '1.65',
					uiFontFamily: 'var(--sl-font)',
					frames: {
						shadowColor: 'transparent',
						inlineButtonBorder: 'transparent',
						inlineButtonBackgroundIdleOpacity: '0',
						inlineButtonBackgroundHoverOrFocusOpacity: '0.08',
						inlineButtonForeground: 'var(--sqc-muted)',
						frameBoxShadowCssValue: 'none',
						editorTabBarBackground: 'var(--sqc-code-bg)',
						editorActiveTabBackground: 'var(--sqc-code-bg)',
						editorActiveTabIndicatorTopColor: 'var(--sqc-primary)',
						editorTabBarBorderBottomColor: 'var(--sqc-line)',
						terminalTitlebarBackground: 'var(--sqc-code-bg)',
						terminalTitlebarBorderBottomColor: 'var(--sqc-line)',
						terminalTitlebarDotsOpacity: '0.35',
						terminalBackground: 'var(--sqc-code-bg)',
					},
					borderColor: 'var(--sqc-line)',
				},
			},
			sidebar,
			plugins: [
				starlightLlmsTxt({
					projectName: 'SQLClean',
					description:
						'SQLClean is a free, open-source SQL linter and formatter with safe fixes and custom Rules in Python.',
				}),
			],
		}),
	],
});
