"use strict";

function findMenuItem(menu, text) {
  for (const row of menu.querySelectorAll(".contextmenu-row")) {
    if (row.textContent.includes(text)) {
      return row;
    }
  }

  return null;
}

add_task(async function test_toml() {
  await TestUtils.loadPath("/tests/source/tests/mochitest.toml");

  {
    const call = frame.contentDocument.querySelector("#line-3 .syn_string");
    TestUtils.click(call);

    const menu = frame.contentDocument.querySelector("#context-menu");
    await waitForShown(menu, "Context menu is shown for symbol click");

    const row = findMenuItem(menu, "Go to definition of tests/support.html");
    ok(row, "The menu item exists");
  }

  {
    const call = frame.contentDocument.querySelector("#line-5 .syn_string");
    TestUtils.click(call);

    const menu = frame.contentDocument.querySelector("#context-menu");
    await waitForShown(menu, "Context menu is shown for symbol click");

    const row = findMenuItem(menu, "Go to definition of tests/support.txt");
    ok(row, "The menu item exists");
  }

  {
    const call = frame.contentDocument.querySelector("#line-7 .syn_string");
    TestUtils.click(call);

    const menu = frame.contentDocument.querySelector("#context-menu");
    await waitForShown(menu, "Context menu is shown for symbol click");

    const row = findMenuItem(menu, "Go to definition of js/export.mjs");
    ok(row, "The menu item exists");
  }

  {
    const call = frame.contentDocument.querySelector("#line-12 .syn_string");
    TestUtils.click(call);

    const menu = frame.contentDocument.querySelector("#context-menu");
    await waitForShown(menu, "Context menu is shown for symbol click");

    const row = findMenuItem(menu, "Go to definition of tests/file_something.html");
    ok(row, "The menu item exists");
  }

  {
    const call = frame.contentDocument.querySelector("#line-25 .syn_string");
    TestUtils.click(call);

    const menu = frame.contentDocument.querySelector("#context-menu");
    await waitForShown(menu, "Context menu is shown for symbol click");

    const row = findMenuItem(menu, "Go to definition of tests/mochitest-common.toml");
    ok(row, "The menu item exists");
  }
});
