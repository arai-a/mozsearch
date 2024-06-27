// @@PATH: /

"use strict";

add_task(async function test_SearchPathFilter() {
  const query = document.querySelector("#query");
  TestUtils.setText(query, "PathFilter");

  const content = document.querySelector("#content");

  await waitForCondition(() => content.textContent.includes("Core code (2 lines"));
  await waitForCondition(() => content.textContent.includes("class PathFilter"));
  await waitForCondition(() => content.textContent.includes("WebTest.cpp"));
  await waitForCondition(() => content.textContent.includes("WebTestPathFilter.cpp"));
  ok(true, "2 classes match without path filter");

  await waitForCondition(() => document.location.href.includes("PathFilter"));
  await waitForCondition(() => document.location.href.includes("path=&"));
  ok(true, "URL is updated");

  const pathFilter = document.querySelector("#path");
  TestUtils.setText(pathFilter, "Filter.cpp");

  await waitForCondition(() => content.textContent.includes("Core code (1 lines"));
  await waitForCondition(() => content.textContent.includes("class PathFilter"));
  await waitForCondition(() => content.textContent.includes("WebTestPathFilter.cpp"));
  ok(true, "1 class matches without path filter");

  await waitForCondition(() => document.location.href.includes("PathFilter"));
  await waitForCondition(() => document.location.href.includes("path=Filter.cpp&"));
  ok(true, "URL is updated");
});
