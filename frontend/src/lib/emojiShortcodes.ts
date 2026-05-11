import { Extension, InputRule } from '@tiptap/core';

export const EMOJI_MAP: Record<string, string> = {
	// ── Status ────────────────────────────────────────────
	check: '✅',   done: '✅',
	x: '❌',       no: '❌',       wrong: '❌',
	warning: '⚠️', warn: '⚠️',
	info: 'ℹ️',
	question: '❓',
	exclamation: '❗',
	clock: '⏰',
	hourglass: '⏳', wait: '⏳',
	repeat: '🔄',

	// ── Reactions ─────────────────────────────────────────
	heart: '❤️',
	thumbsup: '👍',   '+1': '👍',
	thumbsdown: '👎', '-1': '👎',
	wave: '👋',
	eyes: '👀',
	brain: '🧠',
	muscle: '💪',

	// ── Notes & documents ─────────────────────────────────
	note: '📝',
	memo: '📋',
	book: '📚',
	books: '📖',
	folder: '📁',
	pin: '📌',
	bookmark: '🔖',
	flag: '🚩',
	calendar: '📅',
	mail: '📧',
	chart: '📊',
	graph: '📈',

	// ── Tools & actions ───────────────────────────────────
	search: '🔍',
	link: '🔗',
	lock: '🔒',
	unlock: '🔓',
	key: '🔑',
	bulb: '💡',   idea: '💡',
	bell: '🔔',
	tools: '🛠️',
	gear: '⚙️',   settings: '⚙️',
	wrench: '🔧',
	money: '💰',

	// ── Tech ──────────────────────────────────────────────
	computer: '💻',
	mobile: '📱',
	bug: '🐛',
	rocket: '🚀',
	zap: '⚡',

	// ── Smileys & visages ────────────────────────────────
	smile: '😊',       happy: '😊',
	grin: '😁',
	laughing: '😆',    lol: '😆',
	joy: '😂',         rofl: '🤣',
	wink: '😉',
	blush: '😊',
	heart_eyes: '😍',  love: '😍',
	starstruck: '🤩',
	thinking: '🤔',
	smirk: '😏',
	neutral: '😐',
	expressionless: '😑',
	unamused: '😒',
	confused: '😕',
	worried: '😟',
	sad: '😢',         cry: '😢',
	sob: '😭',
	angry: '😠',       rage: '😡',
	scream: '😱',      shocked: '😱',
	surprised: '😮',   wow: '😮',
	yum: '😋',
	stuck_out_tongue: '😛',
	sweat_smile: '😅',
	sweat: '😓',
	dizzy_face: '😵',
	sleeping: '😴',
	cool: '😎',        sunglasses: '😎',
	nerd: '🤓',
	monocle: '🧐',
	partying: '🥳',
	pensive: '😔',
	relieved: '😌',
	hugging: '🤗',
	shushing: '🤫',
	lying: '🤥',
	sick: '🤒',
	cold: '🥶',
	hot: '🥵',
	clown: '🤡',
	ghost: '👻',
	skull: '💀',
	poop: '💩',
	alien: '👽',
	robot: '🤖',
	cat: '😺',
	devil: '😈',
	angel: '😇',
	100: '💯',

	// ── Expressions ───────────────────────────────────────
	fire: '🔥',
	star: '⭐',
	sparkles: '✨',
	tada: '🎉',   party: '🎉',
	trophy: '🏆',
	target: '🎯',
	art: '🎨',
	music: '🎵',

	// ── Lieux / nature ────────────────────────────────────
	house: '🏠',
	world: '🌍',
	sun: '☀️',
	moon: '🌙',
	snow: '❄️',

	// ── Flèches ───────────────────────────────────────────
	up: '⬆️',
	down: '⬇️',
	right: '➡️',
	left: '⬅️',
};

// Matches :shortcode: at the current cursor position
const SHORTCODE_RE = /:([a-z0-9_+\-]+):/;

export const EmojiShortcodes = Extension.create({
	name: 'emojiShortcodes',

	addInputRules() {
		return [
			new InputRule({
				find: SHORTCODE_RE,
				handler: ({ chain, range, match }) => {
					const emoji = EMOJI_MAP[match[1]];
					if (!emoji) return;
					chain().deleteRange(range).insertContent(emoji).run();
				},
			}),
		];
	},
});
