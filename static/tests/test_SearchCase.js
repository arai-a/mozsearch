// @@PATH: /

"use strict";

add_task(async function test_SearchCase() {
  const query = document.querySelector("#query");
  TestUtils.setText(query, "CaseSensitiveness");

  const content = document.querySelector("#content");

  await waitForCondition(() => content.textContent.includes("Core code (2 lines"));
  await waitForCondition(() => content.textContent.includes("class CaseSensitiveness1"));
  await waitForCondition(() => content.textContent.includes("class casesensitiveness2"));
  ok(true, "2 classes match with case==false");

  await waitForCondition(() => document.location.href.includes("CaseSensitiveness"));
  await waitForCondition(() => document.location.href.includes("case=false"));
  ok(true, "URL is updated");

  const caseCheckbox = document.querySelector("#case");
  TestUtils.clickCheckbox(caseCheckbox);

  await waitForCondition(() => content.textContent.includes("Core code (1 lines"));
  await waitForCondition(() => content.textContent.includes("class CaseSensitiveness1"));
  ok(true, "1 class matches with case==true");

  await waitForCondition(() => document.location.href.includes("CaseSensitiveness"));
  await waitForCondition(() => document.location.href.includes("case=true"));
  ok(true, "URL is updated");
});
