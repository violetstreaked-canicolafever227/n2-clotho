/* tslint:disable */
/* eslint-disable */

/**
 * Extract blacklist patterns from .n2 source — returns JSON array
 */
export function extract_blacklist_wasm(source: string): string;

/**
 * Get compiler version info
 */
export function n2c_version(): string;

/**
 * Parse .n2 source and return AST as JSON string
 */
export function parse_n2_wasm(source: string): string;

/**
 * Query .n2 source with SQL — returns formatted table string
 */
export function query_n2_wasm(source: string, sql: string): string;

/**
 * Validate .n2 source — returns JSON with errors/warnings
 */
export function validate_n2_wasm(source: string): string;
