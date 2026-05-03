<script lang="ts">
	import { getBacklinks } from './api';

	interface Props {
		note: string;
		onNavigate: (name: string) => void;
	}

	let { note, onNavigate }: Props = $props();

	let backlinks = $state<string[]>([]);

	$effect(() => {
		const n = note;
		let cancelled = false;
		getBacklinks(n)
			.then((r) => { if (!cancelled) backlinks = r.backlinks; })
			.catch(() =>  { if (!cancelled) backlinks = []; });
		return () => { cancelled = true; };
	});
</script>

{#if backlinks.length > 0}
	<aside class="backlinks">
		<header>
			<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
				<path d="M6 4l-4 4 4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
				<path d="M4 8h8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
				<path d="M10 4l4 4-4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
			<span>{backlinks.length} backlink{backlinks.length > 1 ? 's' : ''}</span>
		</header>
		<ul>
			{#each backlinks as bl}
				<li>
					<button onclick={() => onNavigate(bl)}>
						<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
							<path d="M3 2h7l3 3v9H3V2z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
							<path d="M10 2v3h3" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
						</svg>
						{bl}
					</button>
				</li>
			{/each}
		</ul>
	</aside>
{/if}

<style>
	.backlinks {
		flex-shrink: 0;
		border-top: 1px solid var(--border);
		background: var(--sidebar-bg);
		padding: 0.75rem 3rem;
	}

	header {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.78rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--muted);
		margin-bottom: 0.5rem;
	}

	header svg {
		width: 14px;
		height: 14px;
	}

	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem;
	}

	button {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 0.25rem 0.6rem;
		font-size: 0.85rem;
		font-family: inherit;
		color: var(--text);
		cursor: pointer;
		transition: background 80ms, border-color 80ms;
	}

	button:hover {
		background: var(--border);
	}

	button svg {
		width: 13px;
		height: 13px;
		color: var(--muted);
		flex-shrink: 0;
	}
</style>
