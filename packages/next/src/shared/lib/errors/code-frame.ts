import type {
  NapiCodeFrameLocation,
  NapiCodeFrameOptions,
} from '../../../build/swc/generated-native'

const { codeFrameColumns: nativeCodeFrameColumns } =
  require('../../../build/swc') as typeof import('../../../build/swc')

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
 */
export async function codeFrameColumns(
  file: string,
  location: NapiCodeFrameLocation,
  options?: NapiCodeFrameOptions
): Promise<string> {
  options ??= {}
  if (options.maxWidth === undefined) {
    options.maxWidth = process.stdout.columns
  }
  return nativeCodeFrameColumns(file, location, options)
}
