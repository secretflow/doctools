/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable no-undef */

import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";

export const App = () =>
  /*#__PURE__*/ _jsx(literal_block, {
    ids: [],
    classes: [],
    names: [],
    dupnames: [],
    language: "proto",
    children: `syntax = "proto3";

message SearchRequest {
  string query = 1;
  int32 page_number = 2;
  int32 result_per_page = 3;
}

message SearchResponse {
  repeated Result results = 1;
}

message Result {
  string url = 1;
  string title = 2;
  repeated string snippets = 3;
}

service SearchService {
  rpc Search(SearchRequest) returns (SearchResponse);
}`,
  });
