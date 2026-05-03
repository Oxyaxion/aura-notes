export const THEMES = [
    { id: 'default',     label: 'Default' },
    { id: 'rose-pine',   label: 'Rosé Pine' },
    { id: 'dracula',     label: 'Dracula' },
    { id: 'solarized',   label: 'Solarized' },
    { id: 'catppuccin',  label: 'Catppuccin' },
    { id: 'github-dark', label: 'GitHub Dark' },
] as const;

export type ThemeId = typeof THEMES[number]['id'];

const KEY = 'maunotes-theme';

export function loadTheme(): ThemeId {
    return (localStorage.getItem(KEY) as ThemeId) ?? 'default';
}

export function applyTheme(id: ThemeId) {
    if (id === 'default') {
        document.documentElement.removeAttribute('data-theme');
    } else {
        document.documentElement.setAttribute('data-theme', id);
    }
    localStorage.setItem(KEY, id);
}
