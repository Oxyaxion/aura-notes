<script lang="ts">
	import { onMount } from 'svelte';
	import {
		listAssets, deleteAsset,
		listDrawings, deleteDrawing,
		getDrawingPreview, getMediaUsage,
		type AssetMeta,
	} from './api';

	const BASE = import.meta.env.VITE_API_BASE ?? '';

	interface Props {
		onClose: () => void;
	}
	let { onClose }: Props = $props();

	type Preview =
		| { kind: 'image'; asset: AssetMeta }
		| { kind: 'drawing'; name: string };

	let assets = $state<AssetMeta[]>([]);
	let drawings = $state<string[]>([]);
	let drawingPreviews = $state<Record<string, string>>({});
	let loading = $state(true);
	let activeTab = $state<'images' | 'drawings'>('images');
	let preview = $state<Preview | null>(null);
	let usedAssets = $state<Set<string> | null>(null);
	let usedDrawings = $state<Set<string> | null>(null);
	let filter = $state<'all' | 'used' | 'orphaned'>('all');

	const filteredAssets = $derived(
		filter === 'all' || !usedAssets ? assets :
		filter === 'used' ? assets.filter(a => usedAssets!.has(a.name)) :
		assets.filter(a => !usedAssets!.has(a.name))
	);

	const filteredDrawings = $derived(
		filter === 'all' || !usedDrawings ? drawings :
		filter === 'used' ? drawings.filter(d => usedDrawings!.has(d)) :
		drawings.filter(d => !usedDrawings!.has(d))
	);

	onMount(async () => {
		await reload();
	});

	async function reload() {
		loading = true;
		try {
			const [assetList, drawingList, usage] = await Promise.all([
				listAssets(),
				listDrawings(),
				getMediaUsage().catch(() => null),
			]);
			assets = assetList;
			drawings = drawingList;
			if (usage) {
				usedAssets = new Set(usage.used_assets);
				usedDrawings = new Set(usage.used_drawings);
			}
			drawingList.forEach(name => {
				getDrawingPreview(name)
					.then(svg => { drawingPreviews = { ...drawingPreviews, [name]: svg }; })
					.catch(() => {});
			});
		} finally {
			loading = false;
		}
	}

	async function removeAsset(name: string) {
		await deleteAsset(name);
		assets = assets.filter(a => a.name !== name);
		if (preview?.kind === 'image' && preview.asset.name === name) preview = null;
	}

	async function removeDrawing(name: string) {
		await deleteDrawing(name);
		drawings = drawings.filter(d => d !== name);
		const p = { ...drawingPreviews };
		delete p[name];
		drawingPreviews = p;
		if (preview?.kind === 'drawing' && preview.name === name) preview = null;
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function onWindowKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (preview) { preview = null; }
			else { onClose(); }
		}
		if (preview && e.key === 'ArrowRight') nextPreview(1);
		if (preview && e.key === 'ArrowLeft')  nextPreview(-1);
	}

	function nextPreview(dir: 1 | -1) {
		if (!preview) return;
		if (preview.kind === 'image') {
			const cur = preview.asset.name;
			const idx = assets.findIndex(a => a.name === cur);
			const next = assets[(idx + dir + assets.length) % assets.length];
			if (next) preview = { kind: 'image', asset: next };
		} else {
			const idx = drawings.indexOf(preview.name);
			const next = drawings[(idx + dir + drawings.length) % drawings.length];
			if (next) preview = { kind: 'drawing', name: next };
		}
	}

	function onBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) preview = null;
	}


</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="meta-page">
	<!-- ── Header ─────────────────────────────────────────────── -->
	<div class="meta-header">
		<button class="back-btn" onclick={onClose} title="Back to notes">
			<svg viewBox="0 0 16 16" fill="none">
				<path d="M10 3L5 8l5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</button>
		<span class="meta-title">Media library</span>
		<div class="header-right">
			<div class="tabs">
				<button class="tab" class:active={activeTab === 'images'} onclick={() => (activeTab = 'images')}>
					Images <span class="count">{assets.length}</span>
				</button>
				<button class="tab" class:active={activeTab === 'drawings'} onclick={() => (activeTab = 'drawings')}>
					Drawings <span class="count">{drawings.length}</span>
				</button>
			</div>
			{#if usedAssets !== null}
				<div class="filter-pills">
					<button class="pill" class:active={filter === 'all'} onclick={() => (filter = 'all')}>All</button>
					<button class="pill" class:active={filter === 'used'} onclick={() => (filter = 'used')}>Used</button>
					<button class="pill pill-orphaned" class:active={filter === 'orphaned'} onclick={() => (filter = 'orphaned')}>Orphaned</button>
				</div>
			{/if}
		</div>
	</div>

	<!-- ── Grid ───────────────────────────────────────────────── -->
	<div class="meta-body">
		{#if loading}
			<div class="empty-state">Loading…</div>

		{:else if activeTab === 'images'}
			{#if assets.length === 0}
				<div class="empty-state">No images yet. Paste or upload one in a note.</div>
			{:else if filteredAssets.length === 0}
				<div class="empty-state">No {filter} images.</div>
			{:else}
				<div class="grid">
					{#each filteredAssets as asset (asset.name)}
						{@const isOrphaned = usedAssets !== null && !usedAssets.has(asset.name)}
						<div class="card" class:orphaned={isOrphaned}>
							<button class="card-thumb" onclick={() => (preview = { kind: 'image', asset })}>
								<img src="{BASE}/assets/{asset.name}" alt={asset.name} loading="lazy" />
								<div class="thumb-overlay">
									<svg viewBox="0 0 20 20" fill="none"><path d="M8 3H3v5M17 3h-5M3 12v5h5M12 17h5v-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
								</div>
							</button>
							<div class="card-footer">
								<div class="card-info">
									<span class="card-name" title={asset.name}>{asset.name}</span>
									<span class="card-size">{formatSize(asset.size)}</span>
								</div>
								{#if isOrphaned}
									<span class="badge-orphaned">orphaned</span>
								{/if}
								<button class="card-delete" title="Delete" onclick={() => removeAsset(asset.name)}>✕</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}

		{:else if activeTab === 'drawings'}
			{#if drawings.length === 0}
				<div class="empty-state">No drawings yet. Insert one with <kbd>/drawing</kbd>.</div>
			{:else if filteredDrawings.length === 0}
				<div class="empty-state">No {filter} drawings.</div>
			{:else}
				<div class="grid">
					{#each filteredDrawings as name (name)}
						{@const isOrphaned = usedDrawings !== null && !usedDrawings.has(name)}
						<div class="card" class:orphaned={isOrphaned}>
							<button class="card-thumb drawing-thumb" onclick={() => (preview = { kind: 'drawing', name })}>
								{#if drawingPreviews[name]}
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html drawingPreviews[name]}
								{:else}
									<span class="thumb-icon">✎</span>
								{/if}
								<div class="thumb-overlay">
									<svg viewBox="0 0 20 20" fill="none"><path d="M8 3H3v5M17 3h-5M3 12v5h5M12 17h5v-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
								</div>
							</button>
							<div class="card-footer">
								<div class="card-info">
									<span class="card-name" title={name}>{name}</span>
								</div>
								{#if isOrphaned}
									<span class="badge-orphaned">orphaned</span>
								{/if}
								<button class="card-delete" title="Delete" onclick={() => removeDrawing(name)}>✕</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

<!-- ── Preview lightbox ────────────────────────────────────── -->
{#if preview}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="lightbox-backdrop" role="presentation" onmousedown={onBackdropClick}>
		<div class="lightbox">
			<div class="lightbox-toolbar">
				{#if preview.kind === 'image'}
					<span class="lightbox-title">{preview.asset.name}</span>
					<span class="lightbox-meta">{formatSize(preview.asset.size)}</span>
					<button class="lb-delete-btn" onclick={() => removeAsset((preview as { kind: 'image'; asset: AssetMeta }).asset.name)}>
						Delete
					</button>
				{:else}
					<span class="lightbox-title">{preview.name}</span>
					<button class="lb-delete-btn" onclick={() => removeDrawing((preview as { kind: 'drawing'; name: string }).name)}>
						Delete
					</button>
				{/if}
				<button class="lb-close-btn" onclick={() => (preview = null)}>✕</button>
			</div>

			<div class="lightbox-content">
				{#if preview.kind === 'image'}
					<img src="{BASE}/assets/{preview.asset.name}" alt={preview.asset.name} />
				{:else if drawingPreviews[preview.name]}
					<!-- eslint-disable-next-line svelte/no-at-html-tags -->
					<div class="drawing-preview">{@html drawingPreviews[preview.name]}</div>
				{:else}
					<span class="lightbox-placeholder">No preview available</span>
				{/if}
			</div>

			<!-- navigation arrows if more than one item -->
			{#if (preview.kind === 'image' ? assets.length : drawings.length) > 1}
				<button class="lb-nav lb-prev" onclick={() => nextPreview(-1)} title="Previous">‹</button>
				<button class="lb-nav lb-next" onclick={() => nextPreview(1)}  title="Next">›</button>
			{/if}
		</div>
	</div>
{/if}

<style>
	.meta-page {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	/* ── Header ───────────────────────────────────────────────── */
	.meta-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.4rem 1rem;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}

	.back-btn {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--muted);
		padding: 0.25rem;
		border-radius: 5px;
		display: flex;
		align-items: center;
		flex-shrink: 0;
		transition: color 80ms, background 80ms;
	}
	.back-btn svg { width: 16px; height: 16px; }
	.back-btn:hover { color: var(--text); background: var(--border); }

	.meta-title {
		font-size: 1rem;
		font-weight: 600;
		color: var(--text);
		flex-shrink: 0;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-left: auto;
	}

	.tabs {
		display: flex;
		gap: 0.15rem;
	}

	.filter-pills {
		display: flex;
		gap: 0.15rem;
		padding-left: 0.5rem;
		border-left: 1px solid var(--border);
	}

	.pill {
		background: none;
		border: 1px solid transparent;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.78rem;
		font-family: inherit;
		color: var(--muted);
		padding: 0.2rem 0.55rem;
		transition: color 80ms, background 80ms;
	}
	.pill:hover { color: var(--text); background: var(--border); }
	.pill.active { color: var(--text); background: var(--border); }
	.pill-orphaned.active { color: #e09050; background: rgba(224, 144, 80, 0.1); border-color: rgba(224, 144, 80, 0.3); }

	.tab {
		background: none;
		border: 1px solid transparent;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.82rem;
		font-family: inherit;
		color: var(--muted);
		padding: 0.25rem 0.65rem;
		display: flex;
		align-items: center;
		gap: 0.4rem;
		transition: color 80ms, background 80ms;
	}
	.tab:hover { color: var(--text); background: var(--border); }
	.tab.active { color: var(--text); background: var(--border); }

	.count {
		font-size: 0.72rem;
		background: var(--sidebar-bg);
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 0.05rem 0.4rem;
		color: var(--muted);
	}

	/* ── Body ─────────────────────────────────────────────────── */
	.meta-body {
		flex: 1;
		overflow-y: auto;
		padding: 1.5rem 2rem;
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 12rem;
		color: var(--muted);
		font-size: 0.9rem;
		font-style: italic;
	}
	.empty-state kbd {
		font-style: normal;
		font-size: 0.8rem;
		padding: 0.1rem 0.35rem;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--sidebar-bg);
		font-family: inherit;
		color: var(--text);
		margin: 0 0.1rem;
	}

	/* ── Grid ─────────────────────────────────────────────────── */
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
		gap: 1rem;
	}

	.card {
		border: 1px solid var(--border);
		border-radius: 8px;
		overflow: hidden;
		background: var(--sidebar-bg);
		transition: border-color 120ms;
	}
	.card:hover { border-color: var(--muted); }

	.card-thumb {
		width: 100%;
		aspect-ratio: 4 / 3;
		overflow: hidden;
		background: var(--bg);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: zoom-in;
		position: relative;
		border: none;
		padding: 0;
	}
	.card-thumb img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		transition: opacity 120ms;
	}
	.card-thumb:hover img,
	.card-thumb:hover :global(svg:not(.thumb-overlay svg)) { opacity: 0.75; }

	.drawing-thumb :global(svg) {
		width: 100%;
		height: 100%;
		padding: 0.5rem;
	}

	.thumb-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		opacity: 0;
		transition: opacity 120ms;
		pointer-events: none;
	}
	.thumb-overlay svg {
		width: 28px;
		height: 28px;
		color: white;
		filter: drop-shadow(0 1px 3px rgba(0,0,0,0.6));
	}
	.card-thumb:hover .thumb-overlay { opacity: 1; }

	.thumb-icon {
		font-size: 2rem;
		color: var(--muted);
	}

	.card-footer {
		display: flex;
		align-items: center;
		padding: 0.45rem 0.6rem;
		gap: 0.4rem;
		border-top: 1px solid var(--border);
	}

	.card-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
	}

	.card-name {
		font-size: 0.78rem;
		color: var(--text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.card-size {
		font-size: 0.7rem;
		color: var(--muted);
	}

	.badge-orphaned {
		font-size: 0.65rem;
		padding: 0.1rem 0.35rem;
		border-radius: 4px;
		background: rgba(224, 144, 80, 0.1);
		border: 1px solid rgba(224, 144, 80, 0.35);
		color: #e09050;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.card.orphaned {
		border-color: rgba(224, 144, 80, 0.3);
	}

	.card-delete {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--muted);
		font-size: 0.72rem;
		padding: 0.2rem 0.3rem;
		border-radius: 4px;
		flex-shrink: 0;
		line-height: 1;
		transition: color 80ms, background 80ms;
	}
	.card-delete:hover { color: #e57373; background: rgba(229, 115, 115, 0.1); }

	/* ── Lightbox ─────────────────────────────────────────────── */
	.lightbox-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.75);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}

	.lightbox {
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 10px;
		box-shadow: 0 24px 64px rgba(0, 0, 0, 0.4);
		width: min(90vw, 900px);
		max-height: 90vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		position: relative;
	}

	.lightbox-toolbar {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		padding: 0.5rem 0.75rem;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}

	.lightbox-title {
		flex: 1;
		font-size: 0.88rem;
		font-weight: 500;
		color: var(--text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.lightbox-meta {
		font-size: 0.78rem;
		color: var(--muted);
		flex-shrink: 0;
	}

	.lb-delete-btn {
		background: none;
		border: 1px solid var(--border);
		border-radius: 5px;
		cursor: pointer;
		font-size: 0.78rem;
		font-family: inherit;
		color: var(--muted);
		padding: 0.2rem 0.55rem;
		flex-shrink: 0;
		transition: color 80ms, border-color 80ms, background 80ms;
	}
	.lb-delete-btn:hover { color: #e57373; border-color: #e57373; background: rgba(229,115,115,0.08); }

	.lb-close-btn {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--muted);
		font-size: 0.85rem;
		padding: 0.2rem 0.4rem;
		border-radius: 4px;
		line-height: 1;
		flex-shrink: 0;
	}
	.lb-close-btn:hover { color: var(--text); background: var(--border); }

	.lightbox-content {
		flex: 1;
		overflow: auto;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 1.5rem;
		min-height: 0;
	}

	.lightbox-content img {
		max-width: 100%;
		max-height: 100%;
		object-fit: contain;
		border-radius: 4px;
	}

	.drawing-preview {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.drawing-preview :global(svg) {
		max-width: 100%;
		max-height: 100%;
	}

	.lightbox-placeholder {
		color: var(--muted);
		font-style: italic;
		font-size: 0.9rem;
	}

	/* nav arrows */
	.lb-nav {
		position: absolute;
		top: 50%;
		transform: translateY(-50%);
		background: rgba(0,0,0,0.4);
		border: none;
		color: white;
		font-size: 2rem;
		line-height: 1;
		padding: 0.4rem 0.6rem;
		cursor: pointer;
		border-radius: 6px;
		opacity: 0;
		transition: opacity 120ms;
	}
	.lightbox:hover .lb-nav { opacity: 1; }
	.lb-prev { left: 0.5rem; }
	.lb-next { right: 0.5rem; }
	.lb-nav:hover { background: rgba(0,0,0,0.65); }


</style>
