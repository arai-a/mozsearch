// @@PATH: /

"use strict";

add_task(async function test_SearchRegExp() {
  const query = document.querySelector("#query");
  TestUtils.setText(query, "Simpl.Search");

  const content = document.querySelector("#content");

  await waitForCondition(() => content.textContent.includes("No results for current query"));
  ok(true, "Nothing matches with regexp==false");

  await waitForCondition(() => document.location.href.includes("Simpl.Search"));
  await waitForCondition(() => document.location.href.includes("regexp=false"));
  ok(true, "URL is updated");

  const regExpCheckbox = document.querySelector("#regexp");
  TestUtils.clickCheckbox(regExpCheckbox);

  await waitForCondition(() => content.textContent.includes("Core code (1 lines"));
  await waitForCondition(() => content.textContent.includes("class SimpleSearch"));
  ok(true, "1 class matches with regexp==true");

  await waitForCondition(() => document.location.href.includes("Simpl.Search"));
  await waitForCondition(() => document.location.href.includes("regexp=true"));
  ok(true, "URL is updated");
});
