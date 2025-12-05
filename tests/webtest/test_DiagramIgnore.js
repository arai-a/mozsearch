async function openPanel() {
  const toggle = frame.contentDocument.querySelector("#diagram-panel-toggle");
  TestUtils.click(toggle);

  const panel = frame.contentDocument.querySelector("#diagram-panel");
  await waitForCondition(() => !panel.classList.contains("hidden"),
                         "Panel is shown");

  await waitForCondition(() => panel.querySelector("button"),
                         "Apply button is shown");
  const apply = panel.querySelector("button");

  return { panel, apply };
}

add_task(async function test_DiagramIgnore() {
  await TestUtils.loadPath("/tests/query/default?q=calls-to%3A%27diagram_ignore%3A%3AF1%27+depth%3A10");

  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F1Ev"]`),
     "F1 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F2Ev"]`),
     "F2 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F3Ev"]`),
     "F3 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F4Ev"]`),
     "F4 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F5Ev"]`),
     "F5 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F6Ev"]`),
     "F6 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F7Ev"]`),
     "F7 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F8Ev"]`),
     "F8 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F9Ev"]`),
     "F9 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore3F10Ev"]`),
     "F10 is shown");

  await TestUtils.loadPath("/tests/query/default?q=calls-to%3A%27diagram_ignore%3A%3AF1%27+depth%3A10+ignore-nodes%3A%27diagram_ignore%3A%3AF6%27");

  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F1Ev"]`),
     "F1 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F2Ev"]`),
     "F2 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F3Ev"]`),
     "F3 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F4Ev"]`),
     "F4 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F5Ev"]`),
     "F5 is shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F6Ev"]`),
     "F6 is not shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F7Ev"]`),
     "F7 is not shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F8Ev"]`),
     "F8 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F9Ev"]`),
     "F9 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore3F10Ev"]`),
     "F10 is shown");
});

add_task(async function test_DiagramIgnore_control() {
  await TestUtils.loadPath("/tests/query/default?q=calls-to%3A%27diagram_ignore%3A%3AF1%27+depth%3A10+ignore-nodes%3A%27diagram_ignore%3A%3AF6%27");

  const { panel, apply } = await openPanel();

  const ignoreNodes = panel.querySelector("#diagram-option-ignore-nodes");
  is(ignoreNodes.value, "diagram_ignore::F6", "The specified ignore nodes is shown");

  TestUtils.setText(ignoreNodes, "diagram_ignore::F2");

  const loadPromise = TestUtils.waitForLoad();
  TestUtils.click(apply);
  await loadPromise;

  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F1Ev"]`),
     "F1 is shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F2Ev"]`),
     "F2 is not shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F3Ev"]`),
     "F3 is not shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F4Ev"]`),
     "F4 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F5Ev"]`),
     "F5 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F6Ev"]`),
     "F6 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F7Ev"]`),
     "F7 is shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F8Ev"]`),
     "F8 is not shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F9Ev"]`),
     "F9 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore3F10Ev"]`),
     "F10 is shown");
});

add_task(async function test_DiagramIgnore_menu() {
  await TestUtils.loadPath("/tests/query/default?q=calls-to%3A%27diagram_ignore%3A%3AF1%27+depth%3A10");

  const f5 = frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F5Ev"]`);
  TestUtils.click(f5);

  const menu = frame.contentDocument.querySelector("#context-menu");
  await waitForShown(menu, "Context menu is shown for symbol click");

  const brushRows = menu.querySelectorAll(".icon-brush");
  is(brushRows.length, 3, "3 brush items are visible");

  is(brushRows[2].textContent, "Ignore this node in the diagram",
     "the ignore context menu item is shown");

  const loadPromise = TestUtils.waitForLoad();
  TestUtils.click(brushRows[2]);
  await loadPromise;

  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F1Ev"]`),
     "F1 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F2Ev"]`),
     "F2 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F3Ev"]`),
     "F3 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F4Ev"]`),
     "F4 is shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F5Ev"]`),
     "F5 is not shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F6Ev"]`),
     "F6 is not shown");
  ok(!frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F7Ev"]`),
     "F7 is not shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F8Ev"]`),
     "F8 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore2F9Ev"]`),
     "F9 is shown");
  ok(frame.contentDocument.querySelector(`g[data-symbols="_ZN14diagram_ignore3F10Ev"]`),
     "F10 is shown");
});
