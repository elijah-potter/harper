import {
	EditorView,
	ViewPlugin,
	Decoration,
	DecorationSet,
	WidgetType,
	ViewUpdate,
	Command,
	logException,
	KeyBinding,
	hoverTooltip,
	Tooltip,
	showTooltip,
	gutter,
	GutterMarker,
	PanelConstructor,
	Panel,
	showPanel,
	getPanel
} from '@codemirror/view';
import {
	Text,
	StateEffect,
	StateField,
	Extension,
	TransactionSpec,
	Transaction,
	EditorState,
	Facet,
	combineConfig,
	RangeSet,
	Range
} from '@codemirror/state';
import elt from 'crelt';

type Severity = 'hint' | 'info' | 'warning' | 'error';

/// Describes a problem or hint for a piece of code.
export interface Diagnostic {
	/// The start position of the relevant text.
	from: number;
	/// The end position. May be equal to `from`, though actually
	/// covering text is preferable.
	to: number;
	/// The severity of the problem. This will influence how it is
	/// displayed.
	severity: Severity;
	/// When given, add an extra CSS class to parts of the code that
	/// this diagnostic applies to.
	markClass?: string;
	/// An optional source string indicating where the diagnostic is
	/// coming from. You can put the name of your linter here, if
	/// applicable.
	source?: string;
	title?: string;
	/// The message associated with this diagnostic.
	message: string;
	/// An optional custom rendering function that displays the message
	/// as a DOM node.
	renderMessage?: (view: EditorView) => Node;
	/// An optional array of actions that can be taken on this
	/// diagnostic.
	actions?: readonly Action[];
}

/// An action associated with a diagnostic.
export interface Action {
	/// The label to show to the user. Should be relatively short.
	name: string;
	/// The function to call when the user activates this action. Is
	/// given the diagnostic's _current_ position, which may have
	/// changed since the creation of the diagnostic, due to editing.
	apply: (view: EditorView, from: number, to: number) => void;
}

type DiagnosticFilter = (diagnostics: readonly Diagnostic[], state: EditorState) => Diagnostic[];

interface LintConfig {
	/// Time to wait (in milliseconds) after a change before running
	/// the linter. Defaults to 750ms.
	delay?: number;
	/// Optional predicate that can be used to indicate when diagnostics
	/// need to be recomputed. Linting is always re-done on document
	/// changes.
	needsRefresh?: null | ((update: ViewUpdate) => boolean);
	/// Optional filter to determine which diagnostics produce markers
	/// in the content.
	markerFilter?: null | DiagnosticFilter;
	/// Filter applied to a set of diagnostics shown in a tooltip. No
	/// tooltip will appear if the empty set is returned.
	tooltipFilter?: null | DiagnosticFilter;
	/// Can be used to control what kind of transactions cause lint
	/// hover tooltips associated with the given document range to be
	/// hidden. By default any transactions that changes the line
	/// around the range will hide it. Returning null falls back to this
	/// behavior.
	hideOn?: (tr: Transaction, from: number, to: number) => boolean | null;
	/// When enabled (defaults to off), this will cause the lint panel
	/// to automatically open when diagnostics are found, and close when
	/// all diagnostics are resolved or removed.
	autoPanel?: boolean;
}

interface LintGutterConfig {
	/// The delay before showing a tooltip when hovering over a lint gutter marker.
	hoverTime?: number;
	/// Optional filter determining which diagnostics show a marker in
	/// the gutter.
	markerFilter?: null | DiagnosticFilter;
	/// Optional filter for diagnostics displayed in a tooltip, which
	/// can also be used to prevent a tooltip appearing.
	tooltipFilter?: null | DiagnosticFilter;
}

class SelectedDiagnostic {
	constructor(
		readonly from: number,
		readonly to: number,
		readonly diagnostic: Diagnostic
	) {}
}

class LintState {
	constructor(
		readonly diagnostics: DecorationSet,
		readonly panel: PanelConstructor | null,
		readonly selected: SelectedDiagnostic | null
	) {}

	static init(
		diagnostics: readonly Diagnostic[],
		panel: PanelConstructor | null,
		state: EditorState
	) {
		// Filter the list of diagnostics for which to create markers
		let markedDiagnostics = diagnostics;
		const diagnosticFilter = state.facet(lintConfig).markerFilter;
		if (diagnosticFilter) markedDiagnostics = diagnosticFilter(markedDiagnostics, state);

		const ranges = Decoration.set(
			markedDiagnostics.map((d: Diagnostic) => {
				// For zero-length ranges or ranges covering only a line break, create a widget
				return d.from == d.to || (d.from == d.to - 1 && state.doc.lineAt(d.from).to == d.from)
					? Decoration.widget({
							widget: new DiagnosticWidget(d),
							diagnostic: d
						}).range(d.from)
					: Decoration.mark({
							attributes: {
								class:
									'cm-lintRange cm-lintRange-' + d.severity + (d.markClass ? ' ' + d.markClass : '')
							},
							diagnostic: d
						}).range(d.from, d.to);
			}),
			true
		);
		return new LintState(ranges, panel, findDiagnostic(ranges));
	}
}

function findDiagnostic(
	diagnostics: DecorationSet,
	diagnostic: Diagnostic | null = null,
	after = 0
): SelectedDiagnostic | null {
	let found: SelectedDiagnostic | null = null;
	diagnostics.between(after, 1e9, (from, to, { spec }) => {
		if (diagnostic && spec.diagnostic != diagnostic) return;
		found = new SelectedDiagnostic(from, to, spec.diagnostic);
		return false;
	});
	return found;
}

function hideTooltip(tr: Transaction, tooltip: Tooltip) {
	const from = tooltip.pos,
		to = tooltip.end || from;
	const result = tr.state.facet(lintConfig).hideOn(tr, from, to);
	if (result != null) return result;
	const line = tr.startState.doc.lineAt(tooltip.pos);
	return !!(
		tr.effects.some((e) => e.is(setDiagnosticsEffect)) ||
		tr.changes.touchesRange(line.from, Math.max(line.to, to))
	);
}

function maybeEnableLint(state: EditorState, effects: readonly StateEffect<unknown>[]) {
	return state.field(lintState, false)
		? effects
		: effects.concat(StateEffect.appendConfig.of(lintExtensions));
}

/// Returns a transaction spec which updates the current set of
/// diagnostics, and enables the lint extension if if wasn't already
/// active.
export function setDiagnostics(
	state: EditorState,
	diagnostics: readonly Diagnostic[]
): TransactionSpec {
	return {
		effects: maybeEnableLint(state, [setDiagnosticsEffect.of(diagnostics)])
	};
}

/// The state effect that updates the set of active diagnostics. Can
/// be useful when writing an extension that needs to track these.
export const setDiagnosticsEffect = StateEffect.define<readonly Diagnostic[]>();

const togglePanel = StateEffect.define<boolean>();

const movePanelSelection = StateEffect.define<SelectedDiagnostic>();

const lintState = StateField.define<LintState>({
	create() {
		return new LintState(Decoration.none, null, null);
	},
	update(value, tr) {
		if (tr.docChanged && value.diagnostics.size) {
			const mapped = value.diagnostics.map(tr.changes);
			let selected = null;
			const panel = value.panel;
			if (value.selected) {
				const selPos = tr.changes.mapPos(value.selected.from, 1);
				selected =
					findDiagnostic(mapped, value.selected.diagnostic, selPos) ||
					findDiagnostic(mapped, null, selPos);
			}
			if (!mapped.size && panel && tr.state.facet(lintConfig).autoPanel) panel = null;
			value = new LintState(mapped, panel, selected);
		}

		for (const effect of tr.effects) {
			if (effect.is(setDiagnosticsEffect)) {
				const panel = !tr.state.facet(lintConfig).autoPanel
					? value.panel
					: effect.value.length
						? LintPanel.open
						: null;
				value = LintState.init(effect.value, panel, tr.state);
			} else if (effect.is(togglePanel)) {
				value = new LintState(
					value.diagnostics,
					effect.value ? LintPanel.open : null,
					value.selected
				);
			} else if (effect.is(movePanelSelection)) {
				value = new LintState(value.diagnostics, value.panel, effect.value);
			}
		}

		return value;
	},
	provide: (f) => [
		showPanel.from(f, (val) => val.panel),
		EditorView.decorations.from(f, (s) => s.diagnostics)
	]
});

/// Returns the number of active lint diagnostics in the given state.
export function diagnosticCount(state: EditorState) {
	const lint = state.field(lintState, false);
	return lint ? lint.diagnostics.size : 0;
}

const activeMark = Decoration.mark({ class: 'cm-lintRange cm-lintRange-active' });

function lintTooltip(view: EditorView, pos: number, side: -1 | 1) {
	const { diagnostics } = view.state.field(lintState);
	let found: Diagnostic[] = [],
		stackStart = 2e8,
		stackEnd = 0;
	diagnostics.between(pos - (side < 0 ? 1 : 0), pos + (side > 0 ? 1 : 0), (from, to, { spec }) => {
		if (
			pos >= from &&
			pos <= to &&
			(from == to || ((pos > from || side > 0) && (pos < to || side < 0)))
		) {
			found.push(spec.diagnostic);
			stackStart = Math.min(from, stackStart);
			stackEnd = Math.max(to, stackEnd);
		}
	});

	const diagnosticFilter = view.state.facet(lintConfig).tooltipFilter;
	if (diagnosticFilter) found = diagnosticFilter(found, view.state);

	if (!found.length) return null;

	return {
		pos: stackStart,
		end: stackEnd,
		above: view.state.doc.lineAt(stackStart).to < stackEnd,
		create() {
			return { dom: diagnosticsTooltip(view, found) };
		}
	};
}

function diagnosticsTooltip(view: EditorView, diagnostics: readonly Diagnostic[]) {
	return elt(
		'ul',
		{ class: 'cm-tooltip-lint' },
		diagnostics.map((d) => renderDiagnostic(view, d, false))
	);
}

/// Command to open and focus the lint panel.
export const openLintPanel: Command = (view: EditorView) => {
	const field = view.state.field(lintState, false);
	if (!field || !field.panel)
		view.dispatch({ effects: maybeEnableLint(view.state, [togglePanel.of(true)]) });
	const panel = getPanel(view, LintPanel.open);
	if (panel) (panel.dom.querySelector('.cm-panel-lint ul') as HTMLElement).focus();
	return true;
};

/// Command to close the lint panel, when open.
export const closeLintPanel: Command = (view: EditorView) => {
	const field = view.state.field(lintState, false);
	if (!field || !field.panel) return false;
	view.dispatch({ effects: togglePanel.of(false) });
	return true;
};

/// Move the selection to the next diagnostic.
export const nextDiagnostic: Command = (view: EditorView) => {
	const field = view.state.field(lintState, false);
	if (!field) return false;
	const sel = view.state.selection.main;
	let next = field.diagnostics.iter(sel.to + 1);
	if (!next.value) {
		next = field.diagnostics.iter(0);
		if (!next.value || (next.from == sel.from && next.to == sel.to)) return false;
	}
	view.dispatch({ selection: { anchor: next.from, head: next.to }, scrollIntoView: true });
	return true;
};

/// Move the selection to the previous diagnostic.
export const previousDiagnostic: Command = (view: EditorView) => {
	const { state } = view;
	const field = state.field(lintState, false);
	if (!field) return false;
	const sel = state.selection.main;
	let prevFrom: number | undefined,
		prevTo: number | undefined,
		lastFrom: number | undefined,
		lastTo: number | undefined;
	field.diagnostics.between(0, state.doc.length, (from, to) => {
		if (to < sel.to && (prevFrom == null || prevFrom < from)) {
			prevFrom = from;
			prevTo = to;
		}
		if (lastFrom == null || from > lastFrom) {
			lastFrom = from;
			lastTo = to;
		}
	});
	if (lastFrom == null || (prevFrom == null && lastFrom == sel.from)) return false;
	view.dispatch({
		selection: { anchor: prevFrom ?? lastFrom, head: prevTo ?? lastTo },
		scrollIntoView: true
	});
	return true;
};

/// A set of default key bindings for the lint functionality.
///
/// - Ctrl-Shift-m (Cmd-Shift-m on macOS): [`openLintPanel`](#lint.openLintPanel)
/// - F8: [`nextDiagnostic`](#lint.nextDiagnostic)
export const lintKeymap: readonly KeyBinding[] = [
	{ key: 'Mod-Shift-m', run: openLintPanel, preventDefault: true },
	{ key: 'F8', run: nextDiagnostic }
];

/// The type of a function that produces diagnostics.
export type LintSource = (
	view: EditorView
) => readonly Diagnostic[] | Promise<readonly Diagnostic[]>;

const lintPlugin = ViewPlugin.fromClass(
	class {
		lintTime: number;
		timeout = -1;
		set = true;

		constructor(readonly view: EditorView) {
			const { delay } = view.state.facet(lintConfig);
			this.lintTime = Date.now() + delay;
			this.run = this.run.bind(this);
			this.timeout = setTimeout(this.run, delay);
		}

		run() {
			clearTimeout(this.timeout);
			const now = Date.now();
			if (now < this.lintTime - 10) {
				this.timeout = setTimeout(this.run, this.lintTime - now);
			} else {
				this.set = false;
				const { state } = this.view,
					{ sources } = state.facet(lintConfig);
				if (sources.length)
					Promise.all(sources.map((source) => Promise.resolve(source(this.view)))).then(
						(annotations) => {
							const all = annotations.reduce((a, b) => a.concat(b));
							if (this.view.state.doc == state.doc)
								this.view.dispatch(setDiagnostics(this.view.state, all));
						},
						(error) => {
							logException(this.view.state, error);
						}
					);
			}
		}

		update(update: ViewUpdate) {
			const config = update.state.facet(lintConfig);
			if (
				update.docChanged ||
				config != update.startState.facet(lintConfig) ||
				(config.needsRefresh && config.needsRefresh(update))
			) {
				this.lintTime = Date.now() + config.delay;
				if (!this.set) {
					this.set = true;
					this.timeout = setTimeout(this.run, config.delay);
				}
			}
		}

		force() {
			if (this.set) {
				this.lintTime = Date.now();
				this.run();
			}
		}

		destroy() {
			clearTimeout(this.timeout);
		}
	}
);

const lintConfig = Facet.define<
	{ source: LintSource | null; config: LintConfig },
	Required<LintConfig> & { sources: readonly LintSource[] }
>({
	combine(input) {
		return {
			sources: input.map((i) => i.source).filter((x) => x != null) as readonly LintSource[],
			...combineConfig(
				input.map((i) => i.config),
				{
					delay: 750,
					markerFilter: null,
					tooltipFilter: null,
					needsRefresh: null,
					hideOn: () => null
				},
				{
					needsRefresh: (a, b) => (!a ? b : !b ? a : (u) => a(u) || b(u))
				}
			)
		};
	}
});

/// Given a diagnostic source, this function returns an extension that
/// enables linting with that source. It will be called whenever the
/// editor is idle (after its content changed). If `null` is given as
/// source, this only configures the lint extension.
export function linter(source: LintSource | null, config: LintConfig = {}): Extension {
	return [lintConfig.of({ source, config }), lintPlugin, lintExtensions];
}

/// Forces any linters [configured](#lint.linter) to run when the
/// editor is idle to run right away.
export function forceLinting(view: EditorView) {
	const plugin = view.plugin(lintPlugin);
	if (plugin) plugin.force();
}

function assignKeys(actions: readonly Action[] | undefined) {
	const assigned: string[] = [];
	if (actions)
		actions: for (const { name } of actions) {
			for (let i = 0; i < name.length; i++) {
				const ch = name[i];
				if (/[a-zA-Z]/.test(ch) && !assigned.some((c) => c.toLowerCase() == ch.toLowerCase())) {
					assigned.push(ch);
					continue actions;
				}
			}
			assigned.push('');
		}
	return assigned;
}

function renderDiagnostic(view: EditorView, diagnostic: Diagnostic, inPanel: boolean) {
	const keys = inPanel ? assignKeys(diagnostic.actions) : [];
	return elt(
		'li',
		{ class: 'cm-diagnostic cm-diagnostic-' + diagnostic.severity },
		elt('span', { class: 'cm-diagnosticTitle' }, diagnostic.title),
		elt(
			'span',
			{ class: 'cm-diagnosticText' },
			diagnostic.renderMessage ? diagnostic.renderMessage(view) : diagnostic.message
		),
		diagnostic.actions?.map((action, i) => {
			let fired = false;
			const click = (e: Event) => {
				e.preventDefault();
				if (fired) return;
				fired = true;
				const found = findDiagnostic(view.state.field(lintState).diagnostics, diagnostic);
				if (found) action.apply(view, found.from, found.to);
			};
			const { name } = action,
				keyIndex = keys[i] ? name.indexOf(keys[i]) : -1;
			const nameElt =
				keyIndex < 0
					? name
					: [
							name.slice(0, keyIndex),
							elt('u', name.slice(keyIndex, keyIndex + 1)),
							name.slice(keyIndex + 1)
						];
			return elt(
				'button',
				{
					type: 'button',
					class: 'cm-diagnosticAction',
					onclick: click,
					onmousedown: click,
					'aria-label': ` Action: ${name}${keyIndex < 0 ? '' : ` (access key "${keys[i]})"`}.`
				},
				nameElt
			);
		}),
		diagnostic.source && elt('div', { class: 'cm-diagnosticSource' }, diagnostic.source)
	);
}

class DiagnosticWidget extends WidgetType {
	constructor(readonly diagnostic: Diagnostic) {
		super();
	}

	eq(other: DiagnosticWidget) {
		return other.diagnostic == this.diagnostic;
	}

	toDOM() {
		return elt('span', { class: 'cm-lintPoint cm-lintPoint-' + this.diagnostic.severity });
	}
}

class PanelItem {
	id = 'item_' + Math.floor(Math.random() * 0xffffffff).toString(16);
	dom: HTMLElement;

	constructor(
		view: EditorView,
		readonly diagnostic: Diagnostic
	) {
		this.dom = renderDiagnostic(view, diagnostic, true);
		this.dom.id = this.id;
		this.dom.setAttribute('role', 'option');
	}
}

class LintPanel implements Panel {
	items: PanelItem[] = [];
	dom: HTMLElement;
	list: HTMLElement;

	constructor(readonly view: EditorView) {
		const onkeydown = (event: KeyboardEvent) => {
			if (event.keyCode == 27) {
				// Escape
				closeLintPanel(this.view);
				this.view.focus();
			} else if (event.keyCode == 38 || event.keyCode == 33) {
				// ArrowUp, PageUp
				this.moveSelection((this.selectedIndex - 1 + this.items.length) % this.items.length);
			} else if (event.keyCode == 40 || event.keyCode == 34) {
				// ArrowDown, PageDown
				this.moveSelection((this.selectedIndex + 1) % this.items.length);
			} else if (event.keyCode == 36) {
				// Home
				this.moveSelection(0);
			} else if (event.keyCode == 35) {
				// End
				this.moveSelection(this.items.length - 1);
			} else if (event.keyCode == 13) {
				// Enter
				this.view.focus();
			} else if (event.keyCode >= 65 && event.keyCode <= 90 && this.selectedIndex >= 0) {
				// A-Z
				const { diagnostic } = this.items[this.selectedIndex],
					keys = assignKeys(diagnostic.actions);
				for (let i = 0; i < keys.length; i++)
					if (keys[i].toUpperCase().charCodeAt(0) == event.keyCode) {
						const found = findDiagnostic(this.view.state.field(lintState).diagnostics, diagnostic);
						if (found) diagnostic.actions![i].apply(view, found.from, found.to);
					}
			} else {
				return;
			}
			event.preventDefault();
		};
		const onclick = (event: MouseEvent) => {
			for (let i = 0; i < this.items.length; i++) {
				if (this.items[i].dom.contains(event.target as HTMLElement)) this.moveSelection(i);
			}
		};

		this.list = elt('ul', {
			tabIndex: 0,
			role: 'listbox',
			'aria-label': this.view.state.phrase('Diagnostics'),
			onkeydown,
			onclick
		});
		this.dom = elt(
			'div',
			{ class: 'cm-panel-lint' },
			this.list,
			elt(
				'button',
				{
					type: 'button',
					name: 'close',
					'aria-label': this.view.state.phrase('close'),
					onclick: () => closeLintPanel(this.view)
				},
				'×'
			)
		);
		this.update();
	}

	get selectedIndex() {
		const selected = this.view.state.field(lintState).selected;
		if (!selected) return -1;
		for (let i = 0; i < this.items.length; i++)
			if (this.items[i].diagnostic == selected.diagnostic) return i;
		return -1;
	}

	update() {
		const { diagnostics, selected } = this.view.state.field(lintState);
		let i = 0,
			needsSync = false,
			newSelectedItem: PanelItem | null = null;
		diagnostics.between(0, this.view.state.doc.length, (_start, _end, { spec }) => {
			let found = -1,
				item;
			for (let j = i; j < this.items.length; j++)
				if (this.items[j].diagnostic == spec.diagnostic) {
					found = j;
					break;
				}
			if (found < 0) {
				item = new PanelItem(this.view, spec.diagnostic);
				this.items.splice(i, 0, item);
				needsSync = true;
			} else {
				item = this.items[found];
				if (found > i) {
					this.items.splice(i, found - i);
					needsSync = true;
				}
			}
			if (selected && item.diagnostic == selected.diagnostic) {
				if (!item.dom.hasAttribute('aria-selected')) {
					item.dom.setAttribute('aria-selected', 'true');
					newSelectedItem = item;
				}
			} else if (item.dom.hasAttribute('aria-selected')) {
				item.dom.removeAttribute('aria-selected');
			}
			i++;
		});
		while (
			i < this.items.length &&
			!(this.items.length == 1 && this.items[0].diagnostic.from < 0)
		) {
			needsSync = true;
			this.items.pop();
		}
		if (this.items.length == 0) {
			this.items.push(
				new PanelItem(this.view, {
					from: -1,
					to: -1,
					severity: 'info',
					message: this.view.state.phrase('No diagnostics')
				})
			);
			needsSync = true;
		}
		if (newSelectedItem) {
			this.list.setAttribute('aria-activedescendant', newSelectedItem!.id);
			this.view.requestMeasure({
				key: this,
				read: () => ({
					sel: newSelectedItem!.dom.getBoundingClientRect(),
					panel: this.list.getBoundingClientRect()
				}),
				write: ({ sel, panel }) => {
					const scaleY = panel.height / this.list.offsetHeight;
					if (sel.top < panel.top) this.list.scrollTop -= (panel.top - sel.top) / scaleY;
					else if (sel.bottom > panel.bottom)
						this.list.scrollTop += (sel.bottom - panel.bottom) / scaleY;
				}
			});
		} else if (this.selectedIndex < 0) {
			this.list.removeAttribute('aria-activedescendant');
		}
		if (needsSync) this.sync();
	}

	sync() {
		let domPos: ChildNode | null = this.list.firstChild;
		function rm() {
			const prev = domPos!;
			domPos = prev.nextSibling;
			prev.remove();
		}

		for (const item of this.items) {
			if (item.dom.parentNode == this.list) {
				while (domPos != item.dom) rm();
				domPos = item.dom.nextSibling;
			} else {
				this.list.insertBefore(item.dom, domPos);
			}
		}
		while (domPos) rm();
	}

	moveSelection(selectedIndex: number) {
		if (this.selectedIndex < 0) return;
		const field = this.view.state.field(lintState);
		const selection = findDiagnostic(field.diagnostics, this.items[selectedIndex].diagnostic);
		if (!selection) return;
		this.view.dispatch({
			selection: { anchor: selection.from, head: selection.to },
			scrollIntoView: true,
			effects: movePanelSelection.of(selection)
		});
	}

	static open(view: EditorView) {
		return new LintPanel(view);
	}
}

function svg(content: string, attrs = `viewBox="0 0 40 40"`) {
	return `url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" ${attrs}>${encodeURIComponent(content)}</svg>')`;
}

function underline(color: string) {
	return svg(
		`<path d="m0 2.5 l2 -1.5 l1 0 l2 1.5 l1 0" stroke="${color}" fill="none" stroke-width=".7"/>`,
		`width="6" height="3"`
	);
}

const baseTheme = EditorView.baseTheme({
	'.cm-diagnostic': {
		padding: '4px',
		marginLeft: '0px',
		display: 'flex',
		flexDirection: 'column',
		whiteSpace: 'pre-wrap'
	},

	'.cm-diagnosticTitle': {
		boxShadow: 'inset 0 -2px #DB2B39',
		width: 'max-content',
		fontWeight: 'bold'
	},

	'.cm-diagnosticText': {
		marginTop: '8px'
	},

	'.cm-diagnosticAction': {
		font: 'inherit',
		border: 'none',
		marginTop: '8px',
		display: 'flex',
		alignItems: 'center',
		gap: 'var(--size-4-2)',
		padding: 'var(--size-4-1) var(--size-4-2)',
		cursor: 'var(--cursor)',
		fontSize: 'var(--font-ui-small)',
		borderRadius: 'var(--radius-s)',
		whiteSpace: 'nowrap',
		width: '100%'
	},

	'.cm-tooltip': {
		padding: 'var(--size-2-3) !important',
		border: '1px solid var(--background-modifier-border-hover) !important',
		backgroundColor: 'var(--background-secondary) !important',
		borderRadius: 'var(--radius-m) !important',
		boxShadow: 'var(--shadow-s) !important',
		zIndex: 'var(--layer-menu) !important',
		userSelect: 'none !important',
		maxHeight: 'calc(100% - var(--header-height)) !important',
		overflow: 'hidden !important'
	},

	'.cm-diagnosticSource': {
		fontSize: '70%',
		opacity: 0.7
	},

	'.cm-lintRange': {
		backgroundPosition: 'left bottom',
		backgroundRepeat: 'repeat-x',
		paddingBottom: '0.7px'
	},

	'.cm-lintRange-error': { backgroundImage: underline('#d11') },
	'.cm-lintRange-warning': { backgroundImage: underline('orange') },
	'.cm-lintRange-info': { backgroundImage: underline('#999') },
	'.cm-lintRange-hint': { backgroundImage: underline('#66d') },
	'.cm-lintRange-active': { backgroundColor: '#ffdd9980' },

	'.cm-tooltip-lint': {
		padding: 0,
		margin: 0
	},

	'.cm-lintPoint': {
		position: 'relative',

		'&:after': {
			content: '""',
			position: 'absolute',
			bottom: 0,
			left: '-2px',
			borderLeft: '3px solid transparent',
			borderRight: '3px solid transparent',
			borderBottom: '4px solid #d11'
		}
	},

	'.cm-lintPoint-warning': {
		'&:after': { borderBottomColor: 'orange' }
	},
	'.cm-lintPoint-info': {
		'&:after': { borderBottomColor: '#999' }
	},
	'.cm-lintPoint-hint': {
		'&:after': { borderBottomColor: '#66d' }
	},

	'.cm-panel.cm-panel-lint': {
		position: 'relative',
		'& ul': {
			maxHeight: '100px',
			overflowY: 'auto',
			'& [aria-selected]': {
				backgroundColor: '#ddd',
				'& u': { textDecoration: 'underline' }
			},
			'&:focus [aria-selected]': {
				background_fallback: '#bdf',
				backgroundColor: 'Highlight',
				color_fallback: 'white',
				color: 'HighlightText'
			},
			'& u': { textDecoration: 'none' },
			padding: 0,
			margin: 0
		},
		'& [name=close]': {
			position: 'absolute',
			top: '0',
			right: '2px',
			background: 'inherit',
			border: 'none',
			font: 'inherit',
			padding: 0,
			margin: 0
		}
	}
});

function severityWeight(sev: Severity) {
	return sev == 'error' ? 4 : sev == 'warning' ? 3 : sev == 'info' ? 2 : 1;
}

class LintGutterMarker extends GutterMarker {
	severity: Severity;
	constructor(readonly diagnostics: readonly Diagnostic[]) {
		super();
		this.severity = diagnostics.reduce(
			(max, d) => (severityWeight(max) < severityWeight(d.severity) ? d.severity : max),
			'hint' as Severity
		);
	}

	toDOM(view: EditorView) {
		const elt = document.createElement('div');
		elt.className = 'cm-lint-marker cm-lint-marker-' + this.severity;

		let diagnostics = this.diagnostics;
		const diagnosticsFilter = view.state.facet(lintGutterConfig).tooltipFilter;
		if (diagnosticsFilter) diagnostics = diagnosticsFilter(diagnostics, view.state);

		if (diagnostics.length) elt.onmouseover = () => gutterMarkerMouseOver(view, elt, diagnostics);

		return elt;
	}
}

const enum Hover {
	Time = 300,
	Margin = 10
}

function trackHoverOn(view: EditorView, marker: HTMLElement) {
	const mousemove = (event: MouseEvent) => {
		const rect = marker.getBoundingClientRect();
		if (
			event.clientX > rect.left - Hover.Margin &&
			event.clientX < rect.right + Hover.Margin &&
			event.clientY > rect.top - Hover.Margin &&
			event.clientY < rect.bottom + Hover.Margin
		)
			return;
		for (let target = event.target as Node | null; target; target = target.parentNode) {
			if (target.nodeType == 1 && (target as HTMLElement).classList.contains('cm-tooltip-lint'))
				return;
		}
		window.removeEventListener('mousemove', mousemove);
		if (view.state.field(lintGutterTooltip))
			view.dispatch({ effects: setLintGutterTooltip.of(null) });
	};
	window.addEventListener('mousemove', mousemove);
}

function gutterMarkerMouseOver(
	view: EditorView,
	marker: HTMLElement,
	diagnostics: readonly Diagnostic[]
) {
	function hovered() {
		const line = view.elementAtHeight(marker.getBoundingClientRect().top + 5 - view.documentTop);
		const linePos = view.coordsAtPos(line.from);
		if (linePos) {
			view.dispatch({
				effects: setLintGutterTooltip.of({
					pos: line.from,
					above: false,
					create() {
						return {
							dom: diagnosticsTooltip(view, diagnostics),
							getCoords: () => marker.getBoundingClientRect()
						};
					}
				})
			});
		}
		marker.onmouseout = marker.onmousemove = null;
		trackHoverOn(view, marker);
	}

	const { hoverTime } = view.state.facet(lintGutterConfig);

	let hoverTimeout = setTimeout(hovered, hoverTime);
	marker.onmouseout = () => {
		clearTimeout(hoverTimeout);
		marker.onmouseout = marker.onmousemove = null;
	};
	marker.onmousemove = () => {
		clearTimeout(hoverTimeout);
		hoverTimeout = setTimeout(hovered, hoverTime);
	};
}

function markersForDiagnostics(doc: Text, diagnostics: readonly Diagnostic[]) {
	const byLine: { [line: number]: Diagnostic[] } = Object.create(null);
	for (const diagnostic of diagnostics) {
		const line = doc.lineAt(diagnostic.from);
		(byLine[line.from] || (byLine[line.from] = [])).push(diagnostic);
	}
	const markers: Range<GutterMarker>[] = [];
	for (const line in byLine) {
		markers.push(new LintGutterMarker(byLine[line]).range(+line));
	}
	return RangeSet.of(markers, true);
}

const lintGutterExtension = gutter({
	class: 'cm-gutter-lint',
	markers: (view) => view.state.field(lintGutterMarkers)
});

const lintGutterMarkers = StateField.define<RangeSet<GutterMarker>>({
	create() {
		return RangeSet.empty;
	},
	update(markers, tr) {
		markers = markers.map(tr.changes);
		const diagnosticFilter = tr.state.facet(lintGutterConfig).markerFilter;
		for (const effect of tr.effects) {
			if (effect.is(setDiagnosticsEffect)) {
				let diagnostics = effect.value;
				if (diagnosticFilter) diagnostics = diagnosticFilter(diagnostics || [], tr.state);
				markers = markersForDiagnostics(tr.state.doc, diagnostics.slice(0));
			}
		}
		return markers;
	}
});

const setLintGutterTooltip = StateEffect.define<Tooltip | null>();

const lintGutterTooltip = StateField.define<Tooltip | null>({
	create() {
		return null;
	},
	update(tooltip, tr) {
		if (tooltip && tr.docChanged)
			tooltip = hideTooltip(tr, tooltip)
				? null
				: { ...tooltip, pos: tr.changes.mapPos(tooltip.pos) };
		return tr.effects.reduce((t, e) => (e.is(setLintGutterTooltip) ? e.value : t), tooltip);
	},
	provide: (field) => showTooltip.from(field)
});

const lintGutterTheme = EditorView.baseTheme({
	'.cm-gutter-lint': {
		width: '1.4em',
		'& .cm-gutterElement': {
			padding: '.2em'
		}
	},
	'.cm-lint-marker': {
		width: '1em',
		height: '1em'
	},
	'.cm-lint-marker-info': {
		content: svg(
			`<path fill="#aaf" stroke="#77e" stroke-width="6" stroke-linejoin="round" d="M5 5L35 5L35 35L5 35Z"/>`
		)
	},
	'.cm-lint-marker-warning': {
		content: svg(
			`<path fill="#fe8" stroke="#fd7" stroke-width="6" stroke-linejoin="round" d="M20 6L37 35L3 35Z"/>`
		)
	},
	'.cm-lint-marker-error': {
		content: svg(`<circle cx="20" cy="20" r="15" fill="#f87" stroke="#f43" stroke-width="6"/>`)
	}
});

const lintExtensions = [
	lintState,
	EditorView.decorations.compute([lintState], (state) => {
		const { selected, panel } = state.field(lintState);
		return !selected || !panel || selected.from == selected.to
			? Decoration.none
			: Decoration.set([activeMark.range(selected.from, selected.to)]);
	}),
	hoverTooltip(lintTooltip, { hideOn: hideTooltip }),
	baseTheme
];

const lintGutterConfig = Facet.define<LintGutterConfig, Required<LintGutterConfig>>({
	combine(configs) {
		return combineConfig(configs, {
			hoverTime: Hover.Time,
			markerFilter: null,
			tooltipFilter: null
		});
	}
});

/// Returns an extension that installs a gutter showing markers for
/// each line that has diagnostics, which can be hovered over to see
/// the diagnostics.
export function lintGutter(config: LintGutterConfig = {}): Extension {
	return [
		lintGutterConfig.of(config),
		lintGutterMarkers,
		lintGutterExtension,
		lintGutterTheme,
		lintGutterTooltip
	];
}

/// Iterate over the marked diagnostics for the given editor state,
/// calling `f` for each of them. Note that, if the document changed
/// since the diagnostics were created, the `Diagnostic` object will
/// hold the original outdated position, whereas the `to` and `from`
/// arguments hold the diagnostic's current position.
export function forEachDiagnostic(
	state: EditorState,
	f: (d: Diagnostic, from: number, to: number) => void
) {
	const lState = state.field(lintState, false);
	if (lState && lState.diagnostics.size)
		for (let iter = RangeSet.iter([lState.diagnostics]); iter.value; iter.next())
			f(iter.value.spec.diagnostic, iter.from, iter.to);
}
