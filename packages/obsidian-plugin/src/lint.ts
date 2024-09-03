import {
	EditorView,
	ViewPlugin,
	Decoration,
	DecorationSet,
	WidgetType,
	ViewUpdate,
	logException,
	hoverTooltip,
	Tooltip
} from '@codemirror/view';
import {
	StateEffect,
	StateField,
	Extension,
	TransactionSpec,
	Transaction,
	EditorState,
	Facet,
	combineConfig,
	RangeSet
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
		readonly selected: SelectedDiagnostic | null
	) {}

	static init(diagnostics: readonly Diagnostic[], state: EditorState) {
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
		return new LintState(ranges, findDiagnostic(ranges));
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

const movePanelSelection = StateEffect.define<SelectedDiagnostic>();

const lintState = StateField.define<LintState>({
	create() {
		return new LintState(Decoration.none, null);
	},
	update(value, tr) {
		if (tr.docChanged && value.diagnostics.size) {
			const mapped = value.diagnostics.map(tr.changes);
			let selected: SelectedDiagnostic | null = null;
			if (value.selected) {
				const selPos = tr.changes.mapPos(value.selected.from, 1);
				selected =
					findDiagnostic(mapped, value.selected.diagnostic, selPos) ||
					findDiagnostic(mapped, null, selPos);
			}
			value = new LintState(mapped, selected);
		}

		for (const effect of tr.effects) {
			if (effect.is(setDiagnosticsEffect)) {
				value = LintState.init(effect.value, tr.state);
			} else if (effect.is(movePanelSelection)) {
				value = new LintState(value.diagnostics, effect.value);
			}
		}

		return value;
	},
	provide: (f) => [EditorView.decorations.from(f, (s) => s.diagnostics)]
});

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

function svg(content: string, attrs = `viewBox="0 0 40 40"`) {
	return `url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" ${attrs}>${encodeURIComponent(content)}</svg>')`;
}

function underline(color: string) {
	return svg(
		`<path d="m0 2.5 l2 -1.5 l1 0 l2 1.5 l1 0" stroke="${color}" fill="none" stroke-width="1"/>`,
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
