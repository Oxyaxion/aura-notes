<script lang="ts">
	export interface Heading {
		level: number;
		text: string;
	}

	interface Props {
		headings: Heading[];
		onDisable: () => void;
	}

	let { headings, onDisable }: Props = $props();

	let open = $state(false);

	const minLevel = $derived(Math.min(...headings.map((h) => h.level)));

	function scrollTo(text: string) {
		const els = document.querySelectorAll(
			'.tiptap-editor h1, .tiptap-editor h2, .tiptap-editor h3, .tiptap-editor h4, .tiptap-editor h5, .tiptap-editor h6'
		);
		const target = Array.from(els).find((h) => h.textContent?.trim() === text);
		if (target) {
			target.scrollIntoView({ behavior: 'smooth', block: 'start' });
			open = false;
		}
	}
</script>

<div class="toc-bar">
	<div class="toc-toggle-row">
		<button class="toc-toggle" onclick={() => (open = !open)} aria-expanded={open}>
			<svg class="toc-icon" viewBox="0 0 16 16" fill="none" aria-hidden="true">
				<path d="M2 4h12M2 8h8M2 12h10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
			</svg>
			<span class="toc-label">Contents</span>
			<span class="toc-count">{headings.length} sections</span>
			<svg class="toc-chevron" class:rotated={open} viewBox="0 0 16 16" fill="none" aria-hidden="true">
				<path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</button>
		<button class="toc-disable" onclick={onDisable} title="Disable for this note">
			Disable
		</button>
	</div>

	{#if open}
		<ol class="toc-list">
			{#each headings as h}
				<li style="padding-left: {(h.level - minLevel) * 1.1}rem">
					<button class="toc-item" onclick={() => scrollTo(h.text)}>
						{h.text}
					</button>
				</li>
			{/each}
		</ol>
	{/if}
</div>

<style>
	.toc-bar {
		border-bottom: 1px solid var(--border);
		background: var(--sidebar-bg);
		flex-shrink: 0;
	}

	.toc-toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 1rem;
	}

	.toc-toggle {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		background: none;
		border: none;
		cursor: pointer;
		color: var(--muted);
		font-size: 0.8rem;
		font-family: inherit;
		padding: 0.4rem 0;
		flex: 1;
		text-align: left;
	}

	.toc-toggle:hover {
		color: var(--text);
	}

	.toc-icon {
		width: 14px;
		height: 14px;
		flex-shrink: 0;
	}

	.toc-label {
		font-weight: 500;
	}

	.toc-count {
		font-size: 0.72rem;
		opacity: 0.7;
	}

	.toc-chevron {
		width: 13px;
		height: 13px;
		transition: transform 150ms ease;
		flex-shrink: 0;
	}

	.toc-chevron.rotated {
		transform: rotate(180deg);
	}

	.toc-disable {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 0.72rem;
		font-family: inherit;
		color: var(--muted);
		padding: 0.2rem 0.3rem;
		border-radius: 4px;
		opacity: 0.6;
	}

	.toc-disable:hover {
		opacity: 1;
		background: var(--border);
	}

	.toc-list {
		list-style: none;
		margin: 0;
		padding: 0.35rem 1rem 0.6rem;
		display: flex;
		flex-direction: column;
		gap: 0.05rem;
	}

	.toc-list li {
		margin: 0;
	}

	.toc-item {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 0.84rem;
		font-family: inherit;
		color: var(--text);
		padding: 0.18rem 0;
		text-align: left;
		opacity: 0.75;
		transition: opacity 100ms;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 100%;
	}

	.toc-item:hover {
		opacity: 1;
		text-decoration: underline;
	}
</style>
