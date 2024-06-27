// @@PATH: /

"use strict";

add_task(async function test_Search() {
  const query = document.querySelector("#query");
  TestUtils.setText(query, "SimpleSearch");

  const content = document.querySelector("#content");

  await waitForCondition(() => content.textContent.includes("Core code (1 lines"));
  await waitForCondition(() => content.textContent.includes("class SimpleSearch"));
  ok(true, "1 class matches");

  await waitForCondition(() => document.location.href.includes("SimpleSearch"));
  ok(true, "URL is updated");
});
