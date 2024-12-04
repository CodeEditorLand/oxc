/* auto-generated by NAPI-RS */
/* eslint-disable */
export interface ArrowFunctionsOptions {
	/**
	 * This option enables the following:
	 * * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
	 * * Add a runtime check to ensure the functions are not instantiated.
	 * * Add names to arrow functions.
	 *
	 * @default false
	 */
	spec?: boolean;
}

export interface CompilerAssumptions {
  ignoreFunctionLength?: boolean
  noDocumentAll?: boolean
  objectRestNoSymbols?: boolean
  pureGetters?: boolean
  setPublicClassFields?: boolean
}

export interface Es2015Options {
	/** Transform arrow functions into function expressions. */
	arrowFunction?: ArrowFunctionsOptions;
}

/** TypeScript Isolated Declarations for Standalone DTS Emit */
export declare function isolatedDeclaration(
	filename: string,
	sourceText: string,
	options?: IsolatedDeclarationsOptions | undefined | null,
): IsolatedDeclarationsResult;

export interface IsolatedDeclarationsOptions {
	/**
	 * Do not emit declarations for code that has an @internal annotation in its JSDoc comment.
	 * This is an internal compiler option; use at your own risk, because the compiler does not check that the result is valid.
	 *
	 * Default: `false`
	 *
	 * See <https://www.typescriptlang.org/tsconfig/#stripInternal>
	 */
	stripInternal?: boolean;

	sourcemap?: boolean;
}

export interface IsolatedDeclarationsResult {
	code: string;

	map?: SourceMap;

	errors: Array<string>;
}

/**
 * Configure how TSX and JSX are transformed.
 *
 * @see {@link https://babeljs.io/docs/babel-plugin-transform-react-jsx#options}
 */
export interface JsxOptions {
	/**
	 * Decides which runtime to use.
	 *
	 * - 'automatic' - auto-import the correct JSX factories
	 * - 'classic' - no auto-import
	 *
	 * @default 'automatic'
	 */
	runtime?: "classic" | "automatic";
	/**
	 * Emit development-specific information, such as `__source` and `__self`.
	 *
	 * @default false
	 *
	 * @see {@link https://babeljs.io/docs/babel-plugin-transform-react-jsx-development}
	 */
	development?: boolean;
	/**
	 * Toggles whether or not to throw an error if an XML namespaced tag name
	 * is used.
	 *
	 * Though the JSX spec allows this, it is disabled by default since React's
	 * JSX does not currently have support for it.
	 *
	 * @default true
	 */
	throwIfNamespace?: boolean;
	/**
	 * Enables `@babel/plugin-transform-react-pure-annotations`.
	 *
	 * It will mark top-level React method calls as pure for tree shaking.
	 *
	 * @see {@link https://babeljs.io/docs/en/babel-plugin-transform-react-pure-annotations}
	 *
	 * @default true
	 */
	pure?: boolean;
	/**
	 * Replaces the import source when importing functions.
	 *
	 * @default 'react'
	 */
	importSource?: string;
	/**
	 * Replace the function used when compiling JSX expressions. It should be a
	 * qualified name (e.g. `React.createElement`) or an identifier (e.g.
	 * `createElement`).
	 *
	 * Only used for `classic` {@link runtime}.
	 *
	 * @default 'React.createElement'
	 */
	pragma?: string;
	/**
	 * Replace the component used when compiling JSX fragments. It should be a
	 * valid JSX tag name.
	 *
	 * Only used for `classic` {@link runtime}.
	 *
	 * @default 'React.Fragment'
	 */
	pragmaFrag?: string;
	/**
	 * When spreading props, use `Object.assign` directly instead of an extend helper.
	 *
	 * Only used for `classic` {@link runtime}.
	 *
	 * @default false
	 */
	useBuiltIns?: boolean;
	/**
	 * When spreading props, use inline object with spread elements directly
	 * instead of an extend helper or Object.assign.
	 *
	 * Only used for `classic` {@link runtime}.
	 *
	 * @default false
	 */
	useSpread?: boolean;
	/**
	 * Enable React Fast Refresh .
	 *
	 * Conforms to the implementation in {@link https://github.com/facebook/react/tree/v18.3.1/packages/react-refresh}
	 *
	 * @default false
	 */
	refresh?: boolean | ReactRefreshOptions;
}

export interface ReactRefreshOptions {
	/**
	 * Specify the identifier of the refresh registration variable.
	 *
	 * @default `$RefreshReg$`.
	 */
	refreshReg?: string;
	/**
	 * Specify the identifier of the refresh signature variable.
	 *
	 * @default `$RefreshSig$`.
	 */
	refreshSig?: string;

	emitFullSignatures?: boolean;
}

export interface SourceMap {
	file?: string;

	mappings: string;

	names: Array<string>;

	sourceRoot?: string;

	sources: Array<string>;

	sourcesContent?: Array<string>;

	version: number;

	x_google_ignoreList?: Array<number>;
}

/**
 * Transpile a JavaScript or TypeScript into a target ECMAScript version.
 *
 * @param filename The name of the file being transformed. If this is a
 * relative path, consider setting the {@link TransformOptions#cwd} option..
 * @param sourceText the source code itself
 * @param options The options for the transformation. See {@link
 * TransformOptions} for more information.
 *
 * @returns an object containing the transformed code, source maps, and any
 * errors that occurred during parsing or transformation.
 */
export declare function transform(
	filename: string,
	sourceText: string,
	options?: TransformOptions | undefined | null,
): TransformResult;

/**
 * Options for transforming a JavaScript or TypeScript file.
 *
 * @see {@link transform}
 */
export interface TransformOptions {
  sourceType?: 'script' | 'module' | 'unambiguous' | undefined
  /** Treat the source text as `js`, `jsx`, `ts`, or `tsx`. */
  lang?: 'js' | 'jsx' | 'ts' | 'tsx'
  /**
   * The current working directory. Used to resolve relative paths in other
   * options.
   */
  cwd?: string
  /**
   * Enable source map generation.
   *
   * When `true`, the `sourceMap` field of transform result objects will be populated.
   *
   * @default false
   *
   * @see {@link SourceMap}
   */
  sourcemap?: boolean
  /** Set assumptions in order to produce smaller output. */
  assumptions?: CompilerAssumptions
  /** Configure how TypeScript is transformed. */
  typescript?: TypeScriptOptions
  /** Configure how TSX and JSX are transformed. */
  jsx?: JsxOptions
  /**
   * Sets the target environment for the generated JavaScript.
   *
   * The lowest target is `es2015`.
   *
   * Example:
   *
   * * 'es2015'
   * * ['es2020', 'chrome58', 'edge16', 'firefox57', 'node12', 'safari11']
   *
   * @default `esnext` (No transformation)
   *
   * @see [esbuild#target](https://esbuild.github.io/api/#target)
   */
  target?: string | Array<string>
  /** Define Plugin */
  define?: Record<string, string>
  /** Inject Plugin */
  inject?: Record<string, string | [string, string]>
}

export interface TransformResult {
	/**
	 * The transformed code.
	 *
	 * If parsing failed, this will be an empty string.
	 */
	code: string;
	/**
	 * The source map for the transformed code.
	 *
	 * This will be set if {@link TransformOptions#sourcemap} is `true`.
	 */
	map?: SourceMap;
	/**
	 * The `.d.ts` declaration file for the transformed code. Declarations are
	 * only generated if `declaration` is set to `true` and a TypeScript file
	 * is provided.
	 *
	 * If parsing failed and `declaration` is set, this will be an empty string.
	 *
	 * @see {@link TypeScriptOptions#declaration}
	 * @see [declaration tsconfig option](https://www.typescriptlang.org/tsconfig/#declaration)
	 */
	declaration?: string;
	/**
	 * Declaration source map. Only generated if both
	 * {@link TypeScriptOptions#declaration declaration} and
	 * {@link TransformOptions#sourcemap sourcemap} are set to `true`.
	 */
	declarationMap?: SourceMap;
	/**
	 * Parse and transformation errors.
	 *
	 * Oxc's parser recovers from common syntax errors, meaning that
	 * transformed code may still be available even if there are errors in this
	 * list.
	 */
	errors: Array<string>;
}

export interface TypeScriptOptions {
	jsxPragma?: string;

	jsxPragmaFrag?: string;

	onlyRemoveTypeImports?: boolean;

	allowNamespaces?: boolean;

	allowDeclareFields?: boolean;
	/**
	 * Also generate a `.d.ts` declaration file for TypeScript files.
	 *
	 * The source file must be compliant with all
	 * [`isolatedDeclarations`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html#isolated-declarations)
	 * requirements.
	 *
	 * @default false
	 */
	declaration?: IsolatedDeclarationsOptions;
	/**
	 * Rewrite or remove TypeScript import/export declaration extensions.
	 *
	 * - When set to `rewrite`, it will change `.ts`, `.mts`, `.cts` extensions to `.js`, `.mjs`, `.cjs` respectively.
	 * - When set to `remove`, it will remove `.ts`/`.mts`/`.cts`/`.tsx` extension entirely.
	 * - When set to `true`, it's equivalent to `rewrite`.
	 * - When set to `false` or omitted, no changes will be made to the extensions.
	 *
	 * @default false
	 */
	rewriteImportExtensions?: "rewrite" | "remove" | boolean;
}
