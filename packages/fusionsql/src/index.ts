/**
 * FusionSQL - Shared utilities for database operations
 */

// Export all page-related types and utilities
export * from './page.js';

// Export all operation types and filters
export * from './op.js';

// Re-export commonly used types for convenience
export type {
  Page,
  PageResult,
  Paged,
  OrderDirection,
} from './page.js';

export type {
  OpValString,
  OpValBool,
  OpValNumber,
  OpValDate,
  OpValDateTime,
  OpValDayJs,
  OpValUuid,
  OpValValue,
  ArrayOpVal,
  // Type arrays
  OpValsString,
  OpValsBool,
  OpValsNumber,
  OpValsDate,
  OpValsDateTime,
  OpValsDayJs,
  OpValsUuid,
  OpValsValue,
  ArrayOpVals,
} from './op.js';

// Utility functions
export {
  newPageWithLimit,
  newPageWithOffsetLimit,
  getPageOffset,
  createPaged,
  createPageResult,
} from './page.js';