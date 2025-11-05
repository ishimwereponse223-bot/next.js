import { tryGetBindingsSync, getBindingsSync } from '../../../build/swc'
import type {
  NapiCodeFrameLocation,
  NapiCodeFrameOptions,
} from '../../../build/swc/generated-native'

/**
 * Renders a code frame showing the location of an error in source code
 *
 * Performs best effort syntax highlighting using ANSI codes and
 *
 * Uses the native Rust implementation for:
 * - Better performance on large files
 * - Proper handling of long lines
 * - Memory efficiency
 * - Accurate syntax highlighting using SWC lexer
 *
 * @param file - The source code to render
 * @param location - The location to highlight (line and column numbers are 1-indexed)
 * @param options - Optional configuration
 * @returns The formatted code frame string
 * @throws if the native bindings have not been installed
 */
export function renderCodeFrame(
  file: string,
  location: NapiCodeFrameLocation,
  options?: NapiCodeFrameOptions
): string {
  return getBindingsSync().codeFrameColumns(
    file,
    location,
    defaultOptions(options)
  )
}
/** Same as {@code codeFrame} but returns {@code undefined} if the native bindings have not been loaded yet. */
export function renderCodeFrameIfNativeBindingsAvailable(
  file: string,
  location: NapiCodeFrameLocation,
  options?: NapiCodeFrameOptions
): string | undefined {
  return tryGetBindingsSync()?.codeFrameColumns(
    file,
    location,
    defaultOptions(options)
  )
}

function defaultOptions(
  options: NapiCodeFrameOptions = {}
): NapiCodeFrameOptions {
  // default to the terminal width.
  if (options.maxWidth === undefined) {
    options.maxWidth = process.stdout.columns
  }
  return options
}
