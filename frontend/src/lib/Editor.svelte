<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { Editor, InputRule } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Placeholder from '@tiptap/extension-placeholder';
	import Image from '@tiptap/extension-image';
	import { Table, TableRow, TableCell, TableHeader } from '@tiptap/extension-table';
	import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight';
	import { createLowlight, common } from 'lowlight';

	const lowlight = createLowlight(common);

	function makeCodeBlockNodeView() {
		return ({ node, getPos, editor: ed }: { node: any; getPos: any; editor: any }) => {
			const dom = document.createElement('div');
			dom.className = 'code-block-wrap';

			const header = document.createElement('div');
			header.className = 'code-block-header';

			// ── Language label (click to edit) ──────────────────
			const langSpan = document.createElement('span');
			langSpan.className = 'code-lang';
			langSpan.title = 'Click to change language';
			langSpan.textContent = node.attrs.language || 'text';

			langSpan.addEventListener('mousedown', (e) => e.preventDefault());
			langSpan.addEventListener('click', () => {
				const input = document.createElement('input');
				input.value = langSpan.textContent === 'text' ? '' : (langSpan.textContent ?? '');
				input.placeholder = 'ex: typescript';
				input.className = 'code-lang-input';
				input.spellcheck = false;

				let discarding = false;

				const applyLang = () => {
					if (discarding) return;
					const newLang = input.value.trim().toLowerCase();
					const pos = typeof getPos === 'function' ? getPos() : undefined;
					if (pos !== undefined) {
						// Use setNodeMarkup by position — updateAttributes relies on
						// the current selection which may have moved when blur fires.
						ed.view.dispatch(
							ed.view.state.tr.setNodeMarkup(pos, undefined, { language: newLang || null })
						);
					}
					if (header.contains(input)) header.replaceChild(langSpan, input);
				};

				input.addEventListener('mousedown', (e) => e.stopPropagation());
				input.addEventListener('blur', applyLang);
				input.addEventListener('keydown', (e) => {
					if (e.key === 'Enter')  { e.preventDefault(); input.blur(); }
					if (e.key === 'Escape') { discarding = true; header.replaceChild(langSpan, input); }
					e.stopPropagation();
				});

				header.replaceChild(input, langSpan);
				input.focus();
				input.select();
			});

			// ── Copy button ──────────────────────────────────────
			const copyBtn = document.createElement('button');
			copyBtn.className = 'code-copy-btn';
			copyBtn.textContent = 'Copy';
			copyBtn.addEventListener('mousedown', (e) => e.preventDefault());
			copyBtn.addEventListener('click', () => {
				navigator.clipboard.writeText(codeEl.innerText ?? '');
				copyBtn.textContent = '✓ Copied';
				setTimeout(() => { copyBtn.textContent = 'Copy'; }, 1500);
			});

			header.appendChild(langSpan);
			header.appendChild(copyBtn);

			const pre = document.createElement('pre');
			const codeEl = document.createElement('code');
			codeEl.className = node.attrs.language ? `language-${node.attrs.language}` : '';
			pre.appendChild(codeEl);

			dom.appendChild(header);
			dom.appendChild(pre);

			return {
				dom,
				contentDOM: codeEl,
				stopEvent(event: Event) {
					return header.contains(event.target as Node);
				},
				ignoreMutation(mutation: { target: Node }) {
					return !codeEl.contains(mutation.target);
				},
				update(updatedNode: any) {
					if (updatedNode.type.name !== 'codeBlock') return false;
					const lang = updatedNode.attrs.language || '';
					if (!header.contains(document.activeElement)) {
						langSpan.textContent = lang || 'text';
					}
					codeEl.className = lang ? `language-${lang}` : '';
					return true;
				},
			};
		};
	}
	import Typography from '@tiptap/extension-typography';
	import Link from '@tiptap/extension-link';
	import { Markdown, type MarkdownStorage } from 'tiptap-markdown';
	import { TaskListMd, TaskItemMd } from './taskExtensions';
	import { WikiLink } from './wikiLink';
	import { createWikiLinkSuggestion } from './wikiLinkSuggestion';
	import { getAliases, uploadAsset } from './api';
	import { SlashCommand } from './slashCommands';
	import { EmojiShortcodes } from './emojiShortcodes';
	import { QueryBlock } from './queryBlock';
	import { DrawingBlock } from './drawingBlock';
	import { Extension } from '@tiptap/core';
	import BubbleMenu from './BubbleMenu.svelte';
	import 'tippy.js/dist/tippy.css';

	// ── Resizable image NodeView ───────────────────────────────────────────────
	const ResizableImage = Image.extend({
		addAttributes() {
			return {
				...this.parent?.(),
				width: {
					default: null,
					parseHTML: (el) => {
						const w = (el as HTMLElement).getAttribute('data-width')
						       ?? (el as HTMLElement).getAttribute('width');
						return w ? parseInt(w, 10) || null : null;
					},
					renderHTML: (attrs) => attrs.width ? { 'data-width': String(attrs.width) } : {},
				},
			};
		},

		addNodeView() {
			return ({ node, getPos, editor: ed }) => {
				let currentNode = node;

				const wrapper = document.createElement('span');
				wrapper.className = 'image-wrapper';
				wrapper.setAttribute('contenteditable', 'false');

				const img = document.createElement('img');
				img.src = node.attrs.src ?? '';
				img.alt = node.attrs.alt ?? '';
				if (node.attrs.width) img.style.width = `${node.attrs.width}px`;

				const handle = document.createElement('div');
				handle.className = 'image-resize-handle';
				handle.title = 'Drag to resize';

				handle.addEventListener('mousedown', (e) => {
					e.preventDefault();
					e.stopPropagation();
					const startX = e.clientX;
					const startWidth = img.getBoundingClientRect().width;

					const onMove = (ev: MouseEvent) => {
						const newW = Math.max(40, startWidth + ev.clientX - startX);
						img.style.width = `${newW}px`;
					};
					const onUp = (ev: MouseEvent) => {
						document.removeEventListener('mousemove', onMove);
						document.removeEventListener('mouseup', onUp);
						const newW = Math.max(40, Math.round(startWidth + ev.clientX - startX));
						const pos = getPos();
						if (typeof pos === 'number') {
							ed.view.dispatch(
								ed.view.state.tr.setNodeMarkup(pos, undefined, {
									...currentNode.attrs,
									width: newW,
								})
							);
						}
					};
					document.addEventListener('mousemove', onMove);
					document.addEventListener('mouseup', onUp);
				});

				wrapper.append(img, handle);

				return {
					dom: wrapper,
					update(updatedNode) {
						if (updatedNode.type.name !== 'image') return false;
						currentNode = updatedNode;
						img.src = updatedNode.attrs.src ?? '';
						img.alt = updatedNode.attrs.alt ?? '';
						img.style.width = updatedNode.attrs.width ? `${updatedNode.attrs.width}px` : '';
						return true;
					},
					destroy() {},
				};
			};
		},

		addStorage() {
			return {
				markdown: {
					serialize(state: any, node: any) {
						const alt = (node.attrs.alt ?? '').replace(/[\[\]]/g, '\\$&');
						const src = node.attrs.src ?? '';
						const width = node.attrs.width;
						if (width) {
							state.write(`![${alt}](${src} "w:${width}")`);
						} else if (node.attrs.title) {
							state.write(`![${alt}](${src} "${node.attrs.title}")`);
						} else {
							state.write(`![${alt}](${src})`);
						}
					},
					parse: {
						// Convert title="w:NNN" → data-width="NNN" before TipTap parses the DOM
						updateDOM(element: Element) {
							element.querySelectorAll('img[title]').forEach((el) => {
								const t = el.getAttribute('title') ?? '';
								if (t.startsWith('w:')) {
									const w = parseInt(t.slice(2), 10);
									if (!isNaN(w)) {
										el.setAttribute('data-width', String(w));
										el.removeAttribute('title');
									}
								}
							});
						},
					},
				},
			};
		},
	});

	const ExitBlockquote = Extension.create({
		name: 'exitBlockquote',
		priority: 1001,
		addKeyboardShortcuts() {
			return {
				'Mod-Enter': () => {
					const sel = this.editor.state.selection;
					const from = sel.$from;

					let bqDepth = -1;
					for (let d = from.depth; d > 0; d--) {
						if (from.node(d).type.name === 'blockquote') { bqDepth = d; break; }
					}
					if (bqDepth === -1) return false;

					// Insert a paragraph right after the blockquote and move cursor there
					return this.editor.chain()
						.insertContentAt(from.after(bqDepth), { type: 'paragraph' })
						.run();
				},
			};
		},
	});

	const ExitEmptyListItem = Extension.create({
		name: 'exitEmptyListItem',
		priority: 1001,
		addKeyboardShortcuts() {
			return {
				Enter: () => {
					const { selection } = this.editor.state;
					if (!selection.empty) return false;
					const anchor = selection.$anchor;
					for (let d = anchor.depth; d > 0; d--) {
						const node = anchor.node(d);
						if (node.type.name === 'listItem' || node.type.name === 'taskItem') {
							// firstChild is the paragraph/block inside the listItem;
							// childCount === 0 means truly empty (no text, no atoms)
							if ((node.firstChild?.childCount ?? 0) === 0) {
								return this.editor.commands.liftListItem(node.type);
							}
							return false;
						}
					}
					return false;
				},
			};
		},
	});

	interface Props {
		noteContent: string;
		noteNames: string[];
		isIndex?: boolean;
		isLocked?: boolean;
		onEdit: (markdown: string) => void;
	}

	let { noteContent, noteNames, isIndex = false, isLocked = false, onEdit }: Props = $props();

	let element: HTMLDivElement;
	let editor: Editor | null = null;  // must NOT be $state — TipTap mutates it internally
	let editorReady = $state(false);   // reactive flag for template
	let isUpdatingFromProp = false;
	let aliasMap: Record<string, string> = {}; // plain let — read lazily in closure, not in template

	// ── Table toolbar ──────────────────────────────────────────────────────────
	let tableActive = $state(false);
	let toolbarStyle = $state('');

	// ── Link prompt ────────────────────────────────────────────────────────────
	let linkPromptOpen = $state(false);
	let linkPromptX = $state(0);
	let linkPromptY = $state(0);
	let linkPromptUrl = $state('');
	let linkPromptText = $state('');
	let linkHasSelection = $state(false);
	let linkHasExisting = $state(false);
	let linkUrlInput = $state<HTMLInputElement | null>(null);
	let linkTextInput = $state<HTMLInputElement | null>(null);

	function applyLink() {
		if (!editor) return;
		linkPromptOpen = false;
		const url = linkPromptUrl.trim();
		if (!url) {
			if (linkHasExisting) editor.chain().focus().unsetLink().run();
			else editor.chain().focus().run();
			return;
		}
		const href = /^https?:\/\//.test(url) ? url : `https://${url}`;
		if (linkHasSelection) {
			editor.chain().focus().setLink({ href }).run();
		} else if (linkHasExisting) {
			// Cursor inside an existing link — update the whole link span
			editor.chain().focus().extendMarkRange('link').setLink({ href }).run();
		} else {
			const label = linkPromptText.trim() || href;
			editor.chain().focus().insertContent({ type: 'text', text: label, marks: [{ type: 'link', attrs: { href } }] }).run();
		}
	}

	function removeLink() {
		if (!editor) return;
		linkPromptOpen = false;
		if (linkHasSelection) {
			editor.chain().focus().unsetLink().run();
		} else {
			editor.chain().focus().extendMarkRange('link').unsetLink().run();
		}
	}

	// ── Link tooltip (shown when cursor rests inside a link) ───────────────────
	let linkTooltipOpen = $state(false);
	let linkTooltipX = $state(0);
	let linkTooltipY = $state(0);
	let linkTooltipHref = $state('');

	function syncLinkTooltip() {
		if (!editor) return;
		const { from, empty } = editor.state.selection;
		if (!empty || !editor.isActive('link')) {
			linkTooltipOpen = false;
			return;
		}
		// Find the <a> element in the DOM to anchor the tooltip to the full link span
		const { node } = editor.view.domAtPos(from);
		let el: Node | null = node instanceof Text ? node.parentElement : node;
		while (el && !(el instanceof HTMLAnchorElement)) el = (el as Element).parentElement;
		if (el instanceof HTMLAnchorElement) {
			const rect = el.getBoundingClientRect();
			linkTooltipX = rect.left;
			linkTooltipY = rect.bottom + 4;
		} else {
			const coords = editor.view.coordsAtPos(from);
			linkTooltipX = coords.left;
			linkTooltipY = coords.bottom + 4;
		}
		linkTooltipHref = editor.getAttributes('link').href ?? '';
		linkTooltipOpen = true;
	}

	// DOM-level handlers stored here so onDestroy can remove them
	let _imgPasteHandler: ((e: ClipboardEvent) => void) | null = null;
	let _imgDropHandler: ((e: DragEvent) => void) | null = null;

	function syncTableToolbar() {
		if (!editor?.isActive('table')) {
			tableActive = false;
			return;
		}
		// Walk up from selection anchor to find the <table> DOM node
		const sel = window.getSelection();
		if (!sel || sel.rangeCount === 0) return;
		let el: Node | null = sel.getRangeAt(0).commonAncestorContainer;
		if (el.nodeType === Node.TEXT_NODE) el = el.parentElement;
		while (el && (el as Element).tagName !== 'TABLE') el = (el as Element).parentElement;
		if (!el) return;

		const rect = (el as Element).getBoundingClientRect();
		tableActive = true;
		const top = Math.max(4, rect.top - 38);
		toolbarStyle = `top:${top}px;left:${rect.left}px;width:${rect.width}px`;
	}

	function getMarkdown(ed: Editor): string {
		return (ed.storage as unknown as { markdown: MarkdownStorage }).markdown.getMarkdown();
	}

	// Custom event handler for palette-triggered image insertion (same pattern as wiki-navigate)
	function onInsertImageEvent(e: Event) {
		const url = (e as CustomEvent<string>).detail;
		if (url) editor?.chain().focus().setImage({ src: url }).run();
	}

	// Reload aliases whenever the note list changes
	$effect(() => {
		const _ = noteNames; // declare dependency first, before any early return
		getAliases().then(m => { aliasMap = m; }).catch(() => {});
	});

	// noteNames is a Svelte 5 prop getter — the closure reads the current value lazily on each call
	onMount(() => {
		const WikiLinkSuggestion = createWikiLinkSuggestion(() => noteNames, () => aliasMap);

		editor = new Editor({
			element,
			extensions: [
				StarterKit.configure({ codeBlock: false }),
				Placeholder.configure({ placeholder: 'Start writing… (type / for commands)' }),
				Markdown.configure({ html: false, transformPastedText: true }),
				ResizableImage,
				TaskListMd,
				TaskItemMd,
				CodeBlockLowlight.configure({ lowlight }).extend({ addNodeView: makeCodeBlockNodeView }),
				Table.configure({ resizable: false }),
				TableRow,
				TableCell,
				TableHeader,
				Typography,
				Link.configure({
					openOnClick: false,
					autolink: true,
					linkOnPaste: true,
				}).extend({
					// autolink:true sets inclusive:true, which causes typing after a link
					// to continue adding link marks. Override to always be non-inclusive.
					inclusive() { return false; },
					addInputRules() {
						return [
							new InputRule({
								find: /\[([^\]]+)\]\(([^)\s]+)\)$/,
								handler: ({ chain, range, match }) => {
									const label = match[1];
									const url = match[2];
									const href = /^https?:\/\//.test(url) ? url : `https://${url}`;
									chain()
										.deleteRange(range)
										.insertContent({ type: 'text', text: label, marks: [{ type: 'link', attrs: { href } }] })
										.run();
								},
							}),
						];
					},
					addKeyboardShortcuts() {
						return {
							'Mod-Shift-k': () => {
								const ed = this.editor;
								const { from, to, empty } = ed.state.selection;
								const coords = ed.view.coordsAtPos(from);
								ed.view.dom.dispatchEvent(new CustomEvent('link-prompt', {
									bubbles: true,
									detail: {
										x: coords.left,
										y: coords.bottom + 8,
										currentUrl: ed.getAttributes('link').href ?? '',
										selectedText: empty ? '' : ed.state.doc.textBetween(from, to),
									},
								}));
								return true;
							},
						};
					},
				}),
				WikiLink,
				WikiLinkSuggestion,
				SlashCommand,
				EmojiShortcodes,
				QueryBlock,
				DrawingBlock,
				ExitBlockquote,
				ExitEmptyListItem,
			],
			content: noteContent,
			editorProps: {
				attributes: {
					class: 'tiptap-editor',
					spellcheck: 'true',
				},
			},
			onUpdate({ editor: ed }) {
				if (isUpdatingFromProp) return;
				onEdit(getMarkdown(ed));
			},
		});

		editor.on('selectionUpdate', () => { syncTableToolbar(); syncLinkTooltip(); });
		// Defer to avoid mutating $state during Svelte's synchronous commit phase
		// (blur fires when Svelte tears down the toolbar DOM, which triggers state_unsafe_mutation)
		editor.on('blur', () => { setTimeout(() => { tableActive = false; linkTooltipOpen = false; }, 0); });
		document.addEventListener('insert-image', onInsertImageEvent);
		element.addEventListener('link-prompt', onLinkPromptEvent);

		// Capture-phase listeners run before ProseMirror sees the event.
		// Return early (without stopImmediatePropagation) for non-image pastes so
		// normal text/markdown paste continues to work.
		_imgPasteHandler = (e: ClipboardEvent) => {
			const imgItem = Array.from(e.clipboardData?.items ?? [])
				.find(it => it.type.startsWith('image/'));
			if (!imgItem) return;
			const file = imgItem.getAsFile();
			if (!file) return;
			e.preventDefault();
			e.stopImmediatePropagation();
			const ext = file.type.split('/')[1]?.replace('jpeg', 'jpg') ?? 'png';
			const named = new File([file], `paste-${Date.now()}.${ext}`, { type: file.type });
			uploadAsset(named)
				.then(url => { editor?.chain().focus().setImage({ src: url }).run(); })
				.catch(err => console.error('Image upload failed', err));
		};
		_imgDropHandler = (e: DragEvent) => {
			const imgFile = Array.from(e.dataTransfer?.files ?? [])
				.find(f => f.type.startsWith('image/'));
			if (!imgFile) return;
			e.preventDefault();
			e.stopImmediatePropagation();
			uploadAsset(imgFile)
				.then(url => { editor?.chain().focus().setImage({ src: url }).run(); })
				.catch(err => console.error('Image upload failed', err));
		};
		element.addEventListener('paste', _imgPasteHandler as EventListener, true);
		element.addEventListener('drop', _imgDropHandler as EventListener, true);

		editorReady = true;
	});

	$effect(() => {
		const content = noteContent; // read first to track the prop before any early return
		if (!editor) return;
		const current = getMarkdown(editor);
		if (content !== current) {
			isUpdatingFromProp = true;
			editor.commands.setContent(content);
			isUpdatingFromProp = false;
		}
	});

	$effect(() => {
		if (!editorReady || !editor) return;
		editor.setEditable(!isLocked);
	});

	async function onLinkPromptEvent(e: Event) {
		linkTooltipOpen = false;
		const { x, y, currentUrl, selectedText } = (e as CustomEvent).detail;
		linkPromptUrl = currentUrl ?? '';
		linkPromptText = '';
		linkHasExisting = !!currentUrl;
		linkHasSelection = !!selectedText;
		linkPromptX = x;
		linkPromptY = y;
		linkPromptOpen = true;
		await tick();
		linkUrlInput?.focus();
		linkUrlInput?.select();
	}

	onDestroy(() => {
		document.removeEventListener('insert-image', onInsertImageEvent);
		element?.removeEventListener('link-prompt', onLinkPromptEvent);
		if (_imgPasteHandler) element.removeEventListener('paste', _imgPasteHandler as EventListener, true);
		if (_imgDropHandler) element.removeEventListener('drop', _imgDropHandler as EventListener, true);
		editor?.destroy();
		editor = null;
		editorReady = false;
	});
</script>

<div bind:this={element} class="editor-wrap" class:index-page={isIndex} class:locked={isLocked}></div>

{#if editorReady && editor}
	<BubbleMenu {editor} />
{/if}

{#if linkTooltipOpen}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="link-tooltip" style="left:{linkTooltipX}px;top:{linkTooltipY}px;" onmousedown={(e) => e.preventDefault()}>
		<span class="lt-url" title={linkTooltipHref}>
			{linkTooltipHref.length > 40 ? linkTooltipHref.slice(0, 40) + '…' : linkTooltipHref}
		</span>
		<span class="lt-sep"></span>
		<button onclick={() => window.open(linkTooltipHref, '_blank', 'noopener noreferrer')} title="Open">↗</button>
		<button onmousedown={(e) => {
			e.preventDefault();
			linkTooltipOpen = false;
			const { from } = editor!.state.selection;
			const coords = editor!.view.coordsAtPos(from);
			editor!.view.dom.dispatchEvent(new CustomEvent('link-prompt', {
				bubbles: true,
				detail: { x: coords.left, y: coords.bottom + 8, currentUrl: linkTooltipHref, selectedText: '' },
			}));
		}} title="Edit">Edit</button>
		<button onmousedown={(e) => {
			e.preventDefault();
			editor!.chain().focus().extendMarkRange('link').unsetLink().run();
			linkTooltipOpen = false;
		}} title="Remove" class="lt-remove">×</button>
	</div>
{/if}

{#if linkPromptOpen}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="link-backdrop" onmousedown={() => { linkPromptOpen = false; editor?.chain().focus().run(); }}></div>
	<div class="link-prompt" style="left:{linkPromptX}px; top:{linkPromptY}px;">
		<input
			bind:this={linkUrlInput}
			bind:value={linkPromptUrl}
			type="url"
			placeholder="https://…"
			autocomplete="off"
			spellcheck={false}
			onkeydown={(e) => {
				if (e.key === 'Enter') { e.preventDefault(); linkHasSelection ? applyLink() : linkTextInput?.focus(); }
				if (e.key === 'Escape') { linkPromptOpen = false; editor?.chain().focus().run(); }
			}}
		/>
		{#if !linkHasSelection}
			<input
				bind:this={linkTextInput}
				bind:value={linkPromptText}
				type="text"
				placeholder="Display text (optional)"
				autocomplete="off"
				spellcheck={false}
				onkeydown={(e) => {
					if (e.key === 'Enter') { e.preventDefault(); applyLink(); }
					if (e.key === 'Escape') { linkPromptOpen = false; editor?.chain().focus().run(); }
				}}
			/>
		{/if}
		<div class="lp-actions">
			<button class="lp-apply" onmousedown={(e) => { e.preventDefault(); applyLink(); }}>Ajouter</button>
			{#if linkHasExisting}
				<button class="lp-remove" onmousedown={(e) => { e.preventDefault(); removeLink(); }}>Supprimer</button>
			{/if}
		</div>
	</div>
{/if}

{#if tableActive && editorReady}
	<div class="table-toolbar" role="toolbar" tabindex="-1" aria-label="Table actions" style={toolbarStyle} onmousedown={(e) => e.preventDefault()}>
		<span class="tb-group">
			<button onclick={() => editor!.chain().focus().addColumnBefore().run()} title="Add column before">
				<svg viewBox="0 0 16 16" fill="none"><rect x="1" y="3" width="4" height="10" rx="1" stroke="currentColor" stroke-width="1.2"/><path d="M9 8h6M12 5v6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
				Col ←
			</button>
			<button onclick={() => editor!.chain().focus().addColumnAfter().run()} title="Add column after">
				Col →
				<svg viewBox="0 0 16 16" fill="none"><rect x="11" y="3" width="4" height="10" rx="1" stroke="currentColor" stroke-width="1.2"/><path d="M1 8h6M4 5v6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
			</button>
		</span>
		<span class="tb-sep"></span>
		<span class="tb-group">
			<button onclick={() => editor!.chain().focus().addRowBefore().run()} title="Add row above">
				<svg viewBox="0 0 16 16" fill="none"><rect x="3" y="1" width="10" height="4" rx="1" stroke="currentColor" stroke-width="1.2"/><path d="M8 9v6M5 12h6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
				Row ↑
			</button>
			<button onclick={() => editor!.chain().focus().addRowAfter().run()} title="Add row below">
				Row ↓
				<svg viewBox="0 0 16 16" fill="none"><rect x="3" y="11" width="10" height="4" rx="1" stroke="currentColor" stroke-width="1.2"/><path d="M8 1v6M5 4h6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
			</button>
		</span>
		<span class="tb-sep"></span>
		<span class="tb-group">
			<button class="danger" onclick={() => editor!.chain().focus().deleteColumn().run()} title="Delete column">
				<svg viewBox="0 0 16 16" fill="none"><rect x="3" y="3" width="10" height="10" rx="1" stroke="currentColor" stroke-width="1.2"/><path d="M8 6v4M6 8h4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" transform="rotate(45 8 8)"/></svg>
				Col
			</button>
			<button class="danger" onclick={() => editor!.chain().focus().deleteRow().run()} title="Delete row">
				<svg viewBox="0 0 16 16" fill="none"><rect x="3" y="3" width="10" height="10" rx="1" stroke="currentColor" stroke-width="1.2"/><path d="M8 6v4M6 8h4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" transform="rotate(45 8 8)"/></svg>
				Row
			</button>
			<button class="danger" onclick={() => editor!.chain().focus().deleteTable().run()} title="Delete table">
				<svg viewBox="0 0 16 16" fill="none"><path d="M2 4h12M2 8h12M2 12h12M4 2v12M8 2v12M12 2v12" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity=".4"/><path d="M3 3l10 10M13 3L3 13" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>
				Table
			</button>
		</span>
	</div>
{/if}

<style>
	.editor-wrap {
		flex: 1;
		overflow-y: auto;
		padding: 2rem 3rem;
	}

	.locked :global(.tiptap-editor) {
		cursor: default;
		user-select: text;
	}

	/* ── Index page layout ──────────────────────────────── */
	.index-page :global(.tiptap-editor h1) {
		text-align: center;
		font-size: 2.4rem;
		margin-bottom: 2rem;
		padding-bottom: 0.8rem;
		border-bottom: 2px solid var(--border);
	}

	.index-page :global(.tiptap-editor) {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 380px), 1fr));
		gap: 1rem;
		align-items: start;
	}

	/* Everything except query blocks spans full width */
	.index-page :global(.tiptap-editor > *:not(.query-block)) {
		grid-column: 1 / -1;
	}

	/* Query blocks in the grid */
	.index-page :global(.query-block) {
		margin: 0;
	}

	:global(.tiptap-editor) {
		outline: none;
		min-height: 100%;
		font-family: 'Inter', system-ui, sans-serif;
		font-size: 1.1rem;
		line-height: 1.7;
		color: var(--text);
	}

	/* Headings */
	:global(.tiptap-editor h1) { font-size: 2rem; font-weight: 600; margin: 1.5rem 0 0.5rem; line-height: 1.3; }
	:global(.tiptap-editor h2) { font-size: 1.5rem; font-weight: 600; margin: 1.2rem 0 0.4rem; line-height: 1.3; }
	:global(.tiptap-editor h3) { font-size: 1.2rem; font-weight: 600; margin: 1rem 0 0.3rem; line-height: 1.3; }

	/* Paragraphs */
	:global(.tiptap-editor p) { margin: 0 0 0.6rem; }

	/* Links */
	:global(.tiptap-editor a) {
		color: var(--accent);
		text-decoration: underline;
		text-underline-offset: 2px;
		cursor: pointer;
	}
	:global(.tiptap-editor a:hover) { opacity: 0.8; }

	/* Link tooltip */
	.link-tooltip {
		position: fixed;
		z-index: 200;
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 3px 6px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		box-shadow: 0 3px 12px rgba(0, 0, 0, 0.12);
		font-size: 0.75rem;
		white-space: nowrap;
		max-width: 380px;
	}

	.lt-url {
		color: var(--muted);
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 200px;
		display: inline-block;
	}

	.lt-sep {
		width: 1px;
		height: 14px;
		background: var(--border);
		flex-shrink: 0;
		margin: 0 3px;
	}

	.link-tooltip button {
		border: none;
		background: none;
		color: var(--text);
		cursor: pointer;
		padding: 2px 5px;
		border-radius: 4px;
		font-size: 0.75rem;
		font-family: inherit;
	}

	.link-tooltip button:hover { background: var(--sidebar-bg); }
	.link-tooltip button.lt-remove { color: #e57373; }
	.link-tooltip button.lt-remove:hover { background: color-mix(in srgb, #e57373 10%, transparent); }

	/* Link prompt */
	.link-backdrop {
		position: fixed;
		inset: 0;
		z-index: 299;
	}

	.link-prompt {
		position: fixed;
		z-index: 300;
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 0.5rem;
		box-shadow: 0 4px 20px rgba(0,0,0,0.14);
		min-width: 280px;
	}

	.link-prompt input {
		background: var(--sidebar-bg);
		border: 1px solid var(--border);
		border-radius: 5px;
		padding: 0.28rem 0.55rem;
		font-size: 0.83rem;
		color: var(--text);
		font-family: inherit;
		outline: none;
		width: 100%;
	}

	.link-prompt input:focus { border-color: var(--accent); }

	.lp-actions {
		display: flex;
		gap: 0.3rem;
	}

	.lp-apply, .lp-remove {
		border-radius: 5px;
		padding: 0.22rem 0.7rem;
		font-size: 0.78rem;
		font-family: inherit;
		cursor: pointer;
		white-space: nowrap;
		border: 1px solid var(--border);
	}

	.lp-apply {
		background: var(--accent);
		border-color: var(--accent);
		color: #fff;
	}

	.lp-apply:hover { opacity: 0.88; }

	.lp-remove {
		background: none;
		color: #e57373;
	}

	.lp-remove:hover { background: color-mix(in srgb, #e57373 10%, transparent); }
	:global(.tiptap-editor p.is-editor-empty:first-child::before) {
		content: attr(data-placeholder);
		float: left;
		color: var(--muted);
		pointer-events: none;
		height: 0;
	}

	/* Lists */
	:global(.tiptap-editor ul) { list-style-type: disc; padding-left: 1.5rem; margin: 0.4rem 0 0.8rem; }
	:global(.tiptap-editor ol) { list-style-type: decimal; padding-left: 1.5rem; margin: 0.4rem 0 0.8rem; }
	:global(.tiptap-editor ul ul) { list-style-type: circle; }
	:global(.tiptap-editor ul ul ul) { list-style-type: square; }
	:global(.tiptap-editor li) { margin: 0.2rem 0; }

	/* Code inline */
	:global(.tiptap-editor code) {
		font-family: 'JetBrains Mono', 'Fira Code', monospace;
		font-size: 0.88em;
		background: var(--border);
		padding: 0.15em 0.4em;
		border-radius: 3px;
	}

	/* Code block wrapper */
	:global(.tiptap-editor .code-block-wrap) {
		border-left: 3px solid var(--border);
		border-radius: 0 4px 4px 0;
		margin: 0.8rem 0;
		background: var(--code-bg, color-mix(in srgb, var(--sidebar-bg) 70%, transparent));
		position: relative;
	}

	/* Header spans full width: lang left, copy right */
	:global(.tiptap-editor .code-block-header) {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.3rem 0.5rem;
		pointer-events: none;
		z-index: 2;
	}

	:global(.tiptap-editor .code-lang) {
		pointer-events: auto;
		font-size: 0.67rem;
		font-family: 'JetBrains Mono', monospace;
		color: var(--muted);
		text-transform: lowercase;
		letter-spacing: 0.03em;
		cursor: pointer;
		padding: 0.1rem 0.35rem;
		border-radius: 3px;
		background: var(--sidebar-bg);
		border: 1px solid var(--border);
		transition: background 80ms, color 80ms;
	}
	:global(.tiptap-editor .code-lang:hover) {
		background: var(--border);
		color: var(--text);
	}

	:global(.tiptap-editor .code-lang-input) {
		pointer-events: auto;
		font-size: 0.67rem;
		font-family: 'JetBrains Mono', monospace;
		color: var(--text);
		background: var(--bg);
		border: 1px solid var(--accent);
		border-radius: 3px;
		outline: none;
		padding: 0.1rem 0.3rem;
		width: 7rem;
	}

	/* Copy: hidden by default, revealed on hover */
	:global(.tiptap-editor .code-copy-btn) {
		font-size: 0.65rem;
		font-family: inherit;
		background: var(--sidebar-bg);
		border: 1px solid var(--border);
		border-radius: 3px;
		padding: 0.1rem 0.4rem;
		color: var(--muted);
		cursor: pointer;
		opacity: 0;
		pointer-events: none;
		transition: opacity 120ms, background 80ms, color 80ms;
	}
	:global(.tiptap-editor .code-block-wrap:hover .code-copy-btn),
	:global(.tiptap-editor .code-block-wrap:focus-within .code-copy-btn) {
		opacity: 1;
		pointer-events: auto;
	}
	:global(.tiptap-editor .code-copy-btn:hover) {
		background: var(--border);
		color: var(--text);
	}

	/* Extra top padding so first code line clears the header badge */
	:global(.tiptap-editor .code-block-wrap pre) {
		margin: 0;
		padding: 2.1rem 1rem 0.85rem;
		overflow-x: auto;
		background: none;
		border: none;
		border-radius: 0;
	}

	:global(.tiptap-editor .code-block-wrap pre code) {
		font-family: 'JetBrains Mono', 'Fira Code', monospace;
		font-size: 0.88rem;
		line-height: 1.6;
		background: none;
		padding: 0;
		border-radius: 0;
		color: var(--text);
	}

	/* ── Syntax highlighting tokens (light) ─────────────── */
	:global(.hljs-comment), :global(.hljs-quote)                    { color: #6a737d; font-style: italic; }
	:global(.hljs-keyword), :global(.hljs-selector-tag)             { color: #d73a49; }
	:global(.hljs-string),  :global(.hljs-regexp), :global(.hljs-addition) { color: #032f62; }
	:global(.hljs-number),  :global(.hljs-literal)                  { color: #005cc5; }
	:global(.hljs-type),    :global(.hljs-params)                   { color: #6f42c1; }
	:global(.hljs-title),   :global(.hljs-section), :global(.hljs-built_in) { color: #6f42c1; font-weight: 600; }
	:global(.hljs-attr),    :global(.hljs-attribute), :global(.hljs-name)   { color: #005cc5; }
	:global(.hljs-variable),:global(.hljs-template-variable)        { color: #e36209; }
	:global(.hljs-meta)                                              { color: #6a737d; }
	:global(.hljs-deletion)                                          { color: #b31d28; }

	/* ── Dark mode overrides ─────────────────────────────── */
	@media (prefers-color-scheme: dark) {
		:global(.hljs-comment), :global(.hljs-quote)                { color: #8b949e; }
		:global(.hljs-keyword), :global(.hljs-selector-tag)         { color: #ff7b72; }
		:global(.hljs-string),  :global(.hljs-regexp), :global(.hljs-addition) { color: #a5d6ff; }
		:global(.hljs-number),  :global(.hljs-literal)              { color: #79c0ff; }
		:global(.hljs-type),    :global(.hljs-params)               { color: #ffa657; }
		:global(.hljs-title),   :global(.hljs-section), :global(.hljs-built_in) { color: #d2a8ff; }
		:global(.hljs-attr),    :global(.hljs-attribute), :global(.hljs-name)   { color: #79c0ff; }
		:global(.hljs-variable),:global(.hljs-template-variable)    { color: #ffa657; }
		:global(.hljs-meta)                                          { color: #8b949e; }
	}

	/* GitHub Dark theme overrides */
	:global([data-theme="github-dark"] .hljs-comment),
	:global([data-theme="github-dark"] .hljs-quote)                 { color: #8b949e; }
	:global([data-theme="github-dark"] .hljs-keyword),
	:global([data-theme="github-dark"] .hljs-selector-tag)          { color: #ff7b72; }
	:global([data-theme="github-dark"] .hljs-string),
	:global([data-theme="github-dark"] .hljs-addition)              { color: #a5d6ff; }
	:global([data-theme="github-dark"] .hljs-number),
	:global([data-theme="github-dark"] .hljs-literal)               { color: #79c0ff; }
	:global([data-theme="github-dark"] .hljs-type),
	:global([data-theme="github-dark"] .hljs-params)                { color: #ffa657; }
	:global([data-theme="github-dark"] .hljs-title),
	:global([data-theme="github-dark"] .hljs-built_in)              { color: #d2a8ff; }
	:global([data-theme="github-dark"] .hljs-attr),
	:global([data-theme="github-dark"] .hljs-name)                  { color: #79c0ff; }
	:global([data-theme="github-dark"] .hljs-variable)              { color: #ffa657; }
	:global([data-theme="github-dark"] .hljs-meta)                  { color: #8b949e; }

	/* Blockquote */
	:global(.tiptap-editor blockquote) {
		border-left: 4px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 14%, var(--bg));
		color: var(--blockquote-text);
		margin: 0.6rem 0;
		padding: 0.4em 0.9em;
		border-radius: 0 4px 4px 0;
	}
	:global(.tiptap-editor blockquote p) {
		font-style: italic;
		font-weight: 500;
		color: inherit;
	}

	/* Table */
	:global(.tiptap-editor table) { border-collapse: collapse; width: 100%; margin: 1rem 0; font-size: 0.95rem; }
	:global(.tiptap-editor th, .tiptap-editor td) { border: 1px solid var(--border); padding: 0.5rem 0.75rem; text-align: left; vertical-align: top; }
	:global(.tiptap-editor th) { background: var(--sidebar-bg); font-weight: 600; }
	:global(.tiptap-editor .selectedCell) { background: color-mix(in srgb, var(--accent) 8%, transparent); }

	/* Images */
	:global(.image-wrapper) {
		display: inline-block;
		position: relative;
		line-height: 0;
		max-width: 100%;
		vertical-align: bottom;
	}
	:global(.image-wrapper img) {
		display: block;
		max-width: 100%;
		height: auto;
		border-radius: 6px;
	}
	:global(.image-resize-handle) {
		position: absolute;
		right: -5px;
		bottom: -5px;
		width: 12px;
		height: 12px;
		background: var(--accent);
		border: 2px solid var(--bg);
		border-radius: 50%;
		cursor: se-resize;
		opacity: 0;
		transition: opacity 120ms;
	}
	:global(.image-wrapper:hover .image-resize-handle) { opacity: 1; }

	/* Horizontal rule */
	:global(.tiptap-editor hr) { border: none; border-top: 1px solid var(--border); margin: 1.5rem 0; }

	/* Table toolbar */
	:global(.table-toolbar) {
		position: fixed;
		z-index: 50;
		display: flex;
		align-items: center;
		gap: 2px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 3px 5px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.12);
		font-size: 0.72rem;
		white-space: nowrap;
		overflow-x: auto;
	}

	.table-toolbar button {
		display: flex;
		align-items: center;
		gap: 3px;
		background: none;
		border: none;
		cursor: pointer;
		color: var(--muted);
		font-size: 0.72rem;
		font-family: inherit;
		padding: 3px 6px;
		border-radius: 5px;
		transition: background 80ms, color 80ms;
		white-space: nowrap;
	}

	.table-toolbar button svg {
		width: 12px;
		height: 12px;
		flex-shrink: 0;
	}

	.table-toolbar button:hover {
		background: var(--border);
		color: var(--text);
	}

	.table-toolbar button.danger:hover {
		background: rgba(229, 115, 115, 0.15);
		color: #e57373;
	}

	.tb-sep {
		width: 1px;
		height: 16px;
		background: var(--border);
		flex-shrink: 0;
		margin: 0 2px;
	}

	.tb-group {
		display: flex;
		align-items: center;
		gap: 1px;
	}

	/* Slash & wiki-link menus */
	:global(.slash-menu) {
		background: var(--bg, #fff);
		border: 1px solid var(--border, #e5e5e5);
		border-radius: 10px;
		box-shadow: 0 8px 30px rgba(0,0,0,0.14);
		padding: 6px;
		min-width: 240px;
		max-height: 340px;
		overflow-y: auto;
	}
	:global(.slash-menu-item) {
		display: flex;
		align-items: center;
		gap: 0.65rem;
		width: 100%;
		text-align: left;
		background: none;
		border: none;
		padding: 0.45rem 0.6rem;
		cursor: pointer;
		border-radius: 6px;
		color: var(--text, #1a1a1a);
	}
	:global(.slash-menu-item.selected), :global(.slash-menu-item:hover) { background: var(--border, #e5e5e5); }
	:global(.slash-menu-icon) {
		width: 32px; height: 32px;
		display: flex; align-items: center; justify-content: center;
		background: var(--sidebar-bg, #f7f7f7);
		border: 1px solid var(--border, #e5e5e5);
		border-radius: 6px;
		font-size: 0.72rem; font-weight: 700;
		font-family: 'JetBrains Mono', monospace;
		flex-shrink: 0;
		color: var(--text, #1a1a1a);
	}
	:global(.slash-menu-text) { display: flex; flex-direction: column; }
	:global(.slash-menu-title) { font-size: 0.88rem; font-weight: 500; line-height: 1.3; }
	:global(.slash-menu-desc)  { font-size: 0.76rem; color: var(--muted, #6b7280); line-height: 1.3; }
	:global(.slash-menu-empty) { padding: 0.75rem 1rem; font-size: 0.85rem; color: var(--muted, #6b7280); }
</style>
