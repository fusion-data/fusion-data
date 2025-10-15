"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var index_js_1 = require("./dist/index.js");
// Test that types are working
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
console.log('Imports successful!', testPage, testFilter, testNumberFilter);
console.log('SDK available:', typeof index_js_1.HetumindSDK);
