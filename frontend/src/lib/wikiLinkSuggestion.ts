import { Extension } from '@tiptap/core';
import Suggestion, {
	type SuggestionProps,
	type SuggestionKeyDownProps,
} from '@tiptap/suggestion';
import { PluginKey } from '@tiptap/pm/state';
import tippy, { type Instance as TippyInstance } from 'tippy.js';
import type { Editor, Range } from '@tiptap/core';
import { escapeHtml } from './utils';

const WikiLinkSuggestionKey = new PluginKey('wikiLinkSuggestion');

interface WikiItem {
	display: string;   // text shown in the menu
	target: string;    // canonical note name used in the link
	aliasOf?: string;  // if this is an alias, the canonical note name
}

function buildMenu(onSelect: (item: WikiItem) => void) {
	let selectedIndex = 0;
	let items: WikiItem[] = [];

	const el = document.createElement('div');
	el.className = 'slash-menu';

	function render() {
		el.innerHTML = '';
		if (items.length === 0) {
			const empty = document.createElement('div');
			empty.className = 'slash-menu-empty';
			empty.textContent = 'No notes found';
			el.appendChild(empty);
			return;
		}
		items.forEach((item, i) => {
			const btn = document.createElement('button');
			btn.className = 'slash-menu-item' + (i === selectedIndex ? ' selected' : '');

			const icon = document.createElement('span');
			icon.className = 'slash-menu-icon';
			icon.textContent = '[[';

			const text = document.createElement('span');
			text.className = 'slash-menu-text';
			if (item.aliasOf) {
				text.innerHTML =
					`<span class="slash-menu-title">${escapeHtml(item.display)}</span>` +
					`<span class="slash-menu-desc">→ ${escapeHtml(item.aliasOf)}</span>`;
			} else {
				text.innerHTML = `<span class="slash-menu-title">${escapeHtml(item.display)}</span>`;
			}

			btn.appendChild(icon);
			btn.appendChild(text);
			btn.addEventListener('mousedown', (e) => {
				e.preventDefault();
				onSelect(item);
			});
			el.appendChild(btn);
		});
	}

	return {
		el,
		update(newItems: WikiItem[]) {
			items = newItems;
			selectedIndex = 0;
			render();
		},
		move(delta: number) {
			if (!items.length) return;
			selectedIndex = (selectedIndex + delta + items.length) % items.length;
			render();
		},
		select(): WikiItem | null {
			return items[selectedIndex] ?? null;
		},
	};
}

function insertWikiLink(editor: Editor, range: Range, target: string) {
	editor
		.chain()
		.focus()
		.deleteRange(range)
		.insertContent({ type: 'wikiLink', attrs: { target } })
		.run();
}

/**
 * Creates the WikiLink suggestion extension.
 * @param getNotes  Returns the current note name list (called on each keystroke).
 * @param getAliases Returns the current alias → canonical note map.
 */
export function createWikiLinkSuggestion(
	getNotes: () => string[],
	getAliases: () => Record<string, string>
) {
	return Extension.create({
		name: 'wikiLinkSuggestion',

		addProseMirrorPlugins() {
			return [
				Suggestion<WikiItem>({
					editor: this.editor,
					pluginKey: WikiLinkSuggestionKey,
					char: '[[',
					allowSpaces: true,
					startOfLine: false,

					items: ({ query }): WikiItem[] => {
						const q = query.toLowerCase();
						const all = getNotes();

						// Normal notes
						const filtered = q
							? all.filter((n) => n.toLowerCase().includes(q))
							: all;
						const noteItems: WikiItem[] = filtered.slice(0, 10).map((name) => ({
							display: name,
							target: name,
						}));

						if (!q) return noteItems;

						// Alias matches (only when there's a query)
						const aliasMap = getAliases();
						const noteSet = new Set(noteItems.map((i) => i.target));
						const aliasItems: WikiItem[] = [];
						for (const [alias, canonical] of Object.entries(aliasMap)) {
							if (!alias.toLowerCase().includes(q)) continue;
							// Skip if canonical already shown as a normal note match
							if (noteSet.has(canonical)) continue;
							aliasItems.push({ display: alias, target: canonical, aliasOf: canonical });
						}

						return [...noteItems, ...aliasItems].slice(0, 12);
					},

					render: () => {
						let popup: TippyInstance[];
						let menu: ReturnType<typeof buildMenu>;
						let editorRef: Editor;
						let rangeRef: Range;

						const doInsert = (item: WikiItem) => {
							insertWikiLink(editorRef, rangeRef, item.target);
						};

						return {
							onStart(props: SuggestionProps<WikiItem>) {
								editorRef = props.editor;
								rangeRef = props.range;
								menu = buildMenu(doInsert);
								menu.update(props.items);

								popup = tippy('body', {
									getReferenceClientRect: () =>
										props.clientRect?.() ?? new DOMRect(0, 0, 0, 0),
									appendTo: () => document.body,
									content: menu.el,
									showOnCreate: true,
									interactive: true,
									trigger: 'manual',
									placement: 'bottom-start',
								});
							},

							onUpdate(props: SuggestionProps<WikiItem>) {
								editorRef = props.editor;
								rangeRef = props.range;
								menu.update(props.items);
								popup?.[0]?.setProps({
									getReferenceClientRect: () =>
										props.clientRect?.() ?? new DOMRect(0, 0, 0, 0),
								});
							},

							onKeyDown({ event }: SuggestionKeyDownProps): boolean {
								if (event.key === 'ArrowDown') { menu.move(1); return true; }
								if (event.key === 'ArrowUp')   { menu.move(-1); return true; }
								if (event.key === 'Enter' || event.key === 'Tab') {
									const item = menu.select();
									if (item) doInsert(item);
									return true;
								}
								if (event.key === 'Escape') { popup?.[0]?.hide(); return true; }
								return false;
							},

							onExit() {
								if (popup?.[0] && !popup[0].state.isDestroyed) {
									popup[0].destroy();
								}
							},
						};
					},

					command: ({ editor, range, props }) => {
						insertWikiLink(editor, range, props.target);
					},
				}),
			];
		},
	});
}
