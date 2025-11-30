add_task(async function test_DiagramFlat() {
  await TestUtils.loadPath("/tests/query/default?q=class-diagram%3A%27diagram_badges%3A%3AC2%27+depth%3A4+hier%3Aflat");

  const texts = [...frame.contentDocument.querySelectorAll(`svg g > text`)];

  const c1 = texts.find(t => t.textContent.includes("C1"));
  ok(c1, "C1 node exists");

  is(c1.parentNode.textContent.trim(), "diagram_badges::C1",
     "flat pretty name is shown");

  const ptr = texts.find(t => t.textContent.includes("mPtr"));
  ok(ptr, "mPtr node exists");

  is(ptr.parentNode.textContent.trim(), "diagram_badges::C2::mPtr",
     "flat pretty name is shown");
});

add_task(async function test_DiagramFlatBadges() {
  await TestUtils.loadPath("/tests/query/default?q=class-diagram%3A%27diagram_badges%3A%3AC2%27+depth%3A4+hier%3Aflatbadges");

  const texts = [...frame.contentDocument.querySelectorAll(`svg g > text`)];

  const c1 = texts.find(t => t.textContent.includes("C1"));
  ok(c1, "C1 node exists");

  ok(c1.parentNode.textContent.trim().startsWith("diagram_badges::C1"),
     "flat pretty name is included");
  ok(c1.parentNode.textContent.trim().includes("\u{1f9ee}"),
     "Ref Counted badge is included");

  const ptr = texts.find(t => t.textContent.includes("mPtr"));
  ok(ptr, "mPtr node exists");

  ok(ptr.parentNode.textContent.trim().startsWith("diagram_badges::C2::mPtr"),
     "flat pretty name is included");
  ok(ptr.parentNode.textContent.trim().includes("\u{1f4aa}"),
     "Strong Ref badge is included");
});
