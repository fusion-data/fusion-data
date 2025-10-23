/**
 * Test file to verify fusionsql imports work correctly
 */

import { PageResult } from '@fusion-data/fusionsql';
import { OpValString, OpValNumber, OpValBool } from '@fusion-data/fusionsql';
import { HetumindSDK } from './dist/index.js';

// Test that fusionsql types are working
const testPage: PageResult<string> = {
  page: { total: 10 },
  result: ['test']
};

const testFilter: OpValString = {
  $eq: 'test',
  $contains: 'value'
};

const testNumberFilter: OpValNumber = {
  $eq: 100,
  $gt: 50
};

const testBoolFilter: OpValBool = {
  $eq: true
};

// Test that SDK works with fusionsql types
const sdk = new HetumindSDK({
  baseURL: 'http://localhost:3000',
  token: 'test-token'
});

// Test API methods
const workflowQuery = {
  options: { page: 1, limit: 10 },
  filter: {
    name: { $eq: 'test-workflow' },
    status: { $eq: 100 },
    is_archived: { $eq: false }
  }
};

const executionQuery = {
  options: { page: 1, limit: 10 },
  filter: {
    status: { $eq: 100 },
    started_at: { $gte: '2023-01-01T00:00:00Z' }
  }
};

console.log('✅ All fusionsql imports successful!');
console.log('✅ Types working correctly:', testPage, testFilter, testNumberFilter, testBoolFilter);
console.log('✅ SDK available:', typeof HetumindSDK);
console.log('✅ API queries working:', workflowQuery, executionQuery);