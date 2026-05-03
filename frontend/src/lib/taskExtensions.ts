/**
 * TaskList and TaskItem extensions that combine:
 * - the interactive NodeView (checkbox) from @tiptap/extension-task-list/item
 * - the Markdown serialization storage expected by tiptap-markdown
 *
 * tiptap-markdown registers its own skeletal TaskList/TaskItem (no NodeView).
 * These replacements keep the full NodeView and add the markdown storage so
 * tiptap-markdown can serialize/deserialize task lists correctly.
 */
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
// eslint-disable-next-line @typescript-eslint/no-explicit-any
import taskListPlugin from 'markdown-it-task-lists';

export const TaskListMd = TaskList.extend({
	addStorage() {
		return {
			markdown: {
				// Renders as a bullet list; each TaskItem adds its own "[x] " prefix.
				serialize(state: any, node: any) {
					const marker = (this as any).editor?.storage?.markdown?.options?.bulletListMarker ?? '-';
					return state.renderList(node, '  ', () => `${marker} `);
				},
				parse: {
					setup(markdownit: any) {
						markdownit.use(taskListPlugin);
					},
					updateDOM(element: Element) {
						element.querySelectorAll('.contains-task-list').forEach((list) => {
							list.setAttribute('data-type', 'taskList');
						});
					},
				},
			},
		};
	},
});

export const TaskItemMd = TaskItem.configure({ nested: true }).extend({
	addStorage() {
		return {
			markdown: {
				serialize(state: any, node: any) {
					state.write(node.attrs.checked ? '[x] ' : '[ ] ');
					state.renderContent(node);
				},
				parse: {
					updateDOM(element: Element) {
						element.querySelectorAll('.task-list-item').forEach((item) => {
							const input = item.querySelector('input');
							item.setAttribute('data-type', 'taskItem');
							if (input) {
								item.setAttribute('data-checked', String((input as HTMLInputElement).checked));
								input.remove();
							}
						});
					},
				},
			},
		};
	},
});
