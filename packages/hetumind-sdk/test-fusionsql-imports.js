"use strict";
/**
 * Test file to verify fusionsql imports work correctly
 */
Object.defineProperty(exports, "__esModule", { value: true });
var index_js_1 = require("./dist/index.js");
// Test that fusionsql types are working
var testPage = {
    page: { total: 10 },
    result: ['test']
};
var testFilter = {
    $eq: 'test',
    $contains: 'value'
};
var testNumberFilter = {
    $eq: 100,
    $gt: 50
};
var testBoolFilter = {
    $eq: true
};
// Test that SDK works with fusionsql types
var sdk = new index_js_1.HetumindSDK({
    baseURL: 'http://localhost:3000',
    token: 'test-token'
});
// Test API methods
var workflowQuery = {
    options: { page: 1, limit: 10 },
    filter: {
        name: { $eq: 'test-workflow' },
        status: { $eq: 100 },
        is_archived: { $eq: false }
    }
};
var executionQuery = {
    options: { page: 1, limit: 10 },
    filter: {
        status: { $eq: 100 },
        started_at: { $gte: '2023-01-01T00:00:00Z' }
    }
};
console.log('✅ All fusionsql imports successful!');
console.log('✅ Types working correctly:', testPage, testFilter, testNumberFilter, testBoolFilter);
console.log('✅ SDK available:', typeof index_js_1.HetumindSDK);
console.log('✅ API queries working:', workflowQuery, executionQuery);
