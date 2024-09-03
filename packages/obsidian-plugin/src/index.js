import logoSvg from '../logo.svg';
import { linter } from './lint';
import { Plugin, addIcon, Menu } from 'obsidian';
import wasm from 'wasm/harper_wasm_bg.wasm';
import init, { lint, get_lint_config_as_object, set_lint_config_from_object } from 'wasm';
import { HarperSettingTab } from './HarperSettingTab';

function suggestionToLabel(sug) {
	if (sug.kind() == 'Remove') {
		return 'Remove';
	} else {
		return `Replace with "${sug.get_replacement_text()}"`;
	}
}

const harperLinter = (plugin) =>
	linter(
		async (view) => {
			if (!plugin.shouldLint()) {
				return [];
			}

			const text = view.state.doc.sliceString(-1);

			await init(await wasm());

			const lints = lint(text);

			return lints.map((lint) => {
				let span = lint.span();

				return {
					from: span.start,
					to: span.end,
					severity: 'error',
					title: lint.lint_kind(),
					message: lint.message(),
					actions: lint.suggestions().map((sug) => {
						return {
							name: suggestionToLabel(sug),
							apply: (view) => {
								if (sug === 'Remove') {
									view.dispatch({
										changes: {
											from: span.start,
											to: span.end,
											insert: ''
										}
									});
								} else {
									view.dispatch({
										changes: {
											from: span.start,
											to: span.end,
											insert: sug.get_replacement_text()
										}
									});
								}
							}
						};
					})
				};
			});
		},
		{
			delay: -1,
			needsRefresh: () => {
				let temp = plugin.lintSettingModified;
				plugin.lintSettingModified = false;
				return temp;
			}
		}
	);

export default class HarperPlugin extends Plugin {
	/** @private */
	shouldAutoLint = true;
	/** @public */
	lintSettingModified = false;

	/** @public
	 * @returns {Promise<Record<string, any>>} */
	async getSettings() {
		await init(await wasm());
		this.lintSettingChanged();

		let lintSettings = await get_lint_config_as_object();

		return { lintSettings };
	}

	/** @public
	 * @param {Record<string, any>} settings
	 * @returns {Promise<void>} */
	async setSettings(settings) {
		await init(await wasm());

		if (settings == null) {
			settings = {};
		}

		if (settings.lintSettings == undefined) {
			settings.lintSettings = {};
		}

		if (settings.lintSettings.spell_check == undefined) {
			settings.lintSettings.spell_check = false;
		}

		set_lint_config_from_object(settings.lintSettings);
		this.lintSettingChanged();
		this.saveData(settings);
	}

	async onload() {
		console.log(harperLinter(this));

		this.registerEditorExtension([harperLinter(this)]);
		this.app.workspace.updateOptions();

		addIcon('harper-logo', logoSvg);

		this.setupCommands();
		this.setupStatusBar();

		this.addSettingTab(new HarperSettingTab(this.app, this));
		await this.setSettings(await this.loadData());
	}

	setupCommands() {
		this.addCommand({
			id: 'harper-toggle-auto-lint',
			name: 'Toggle automatic grammar checking',
			callback: () => this.toggleAutoLint()
		});
	}

	setupStatusBar() {
		/** @type HTMLElement */
		let statusBarItem = this.addStatusBarItem();
		statusBarItem.className += ' mod-clickable';

		let button = document.createElement('span');
		button.style = 'width:24px';
		button.innerHTML = logoSvg;

		button.addEventListener('click', (event) => {
			const menu = new Menu();

			menu.addItem((item) =>
				item
					.setTitle(`${this.shouldAutoLint ? 'Disable' : 'Enable'} automatic checking`)
					.setIcon('documents')
					.onClick(() => {
						this.toggleAutoLint();
					})
			);

			menu.showAtMouseEvent(event);
		});

		statusBarItem.appendChild(button);
	}

	shouldLint() {
		return this.shouldAutoLint;
	}

	/** @param {boolean} shouldAutoLint  */
	setAutoLint(shouldAutoLint) {
		this.shouldAutoLint = shouldAutoLint;
		this.lintSettingChanged();
	}

	toggleAutoLint() {
		this.shouldAutoLint = !this.shouldAutoLint;
		this.lintSettingChanged();
	}

	lintSettingChanged() {
		this.lintSettingModified = true;
		this.app.workspace.updateOptions();
	}
}
