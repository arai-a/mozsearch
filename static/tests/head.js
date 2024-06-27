window.TEST_LOG = [];

class TestHarness {
  static tests = [];

  static log(type, msg) {
    console.log(`${type} - ${msg}`);
    window.TEST_LOG.push([type, msg]);
  }

  static pass(msg) {
    this.log("PASS", msg);
  }

  static fail(msg) {
    this.log("FAIL", msg);
  }

  static info(msg) {
    this.log("INFO", msg);
  }

  static addTest(func) {
    this.tests.push(func);
    return func;
  }

  static async runTests() {
    try {
      for (const func of this.tests) {
        this.info(`Entering test ${func.name}`);
        await func();
        this.info(`Leaving test ${func.name}`);
      }
    } catch (e) {
      this.fail(e.toString());
      this.log("STACK", e.stack);
    }
  }

  static loadTest(file) {
    this.log("TEST_START", file);

    const script = document.createElement("script");
    script.src = `/static/tests/${file}?${Date.now()}`;
    script.addEventListener("load", async () => {
      await this.runTests();
        this.log("TEST_END", file);
    });
    document.body.append(script);
  }
}
window.TestHarness = TestHarness;

class TestUtils {
  static setText(elem, text) {
    elem.value = text;
    const ev = new InputEvent("input", { bubbles: true });
    elem.dispatchEvent(ev);
  }

  static clickCheckbox(elem) {
    elem.checked = !elem.checked;
    const ev = new Event("change", { bubbles: true });
    elem.dispatchEvent(ev);
  }

  static sleep(timeout) {
    return new Promise(resolve => setTimeout(resolve, timeout));
  }

  static async waitForCondition(condition, msg, interval=100, maxTries=50) {
    for (let i = 0; i < maxTries; i ++) {
      if (condition()) {
        return;
      }

      await this.sleep(interval);
    }

    TestHarness.fail(`${msg} - timed out after ${maxTries} tries.`);
  }

  static ok(condition, msg) {
    if (condition) {
      TestHarness.pass(msg);
    } else {
      TestHarness.fail(msg);
    }
  }

  static is(a, b, msg) {
    if (Object.is(a, b)) {
      TestHarness.pass(msg);
    } else {
      TestHarness.fail(`${msg} - Got ${a}, expected ${b}`);
    }
  }

  static isnot(a, b, msg) {
    if (Object.is(a, b)) {
      TestHarness.fail(`${msg} - Didn't expect ${a}, but got it`);
    } else {
      TestHarness.pass(msg);
    }
  }

  static info(msg) {
    TestHarness.info(msg);
  }
}
window.TestUtils = TestUtils;

for (const name of [
  "waitForCondition",
  "ok",
  "is",
  "isnot",
  "info",
]) {
  window[name] = TestUtils[name].bind(TestUtils);
}

function add_task(func) {
  TestHarness.addTest(func);
}
