use std::fs;
use std::thread;
use std::time::{ Instant, Duration };
use std::io::Write;

use async_trait::async_trait;
use clap::Args;
use fantoccini::{Client, ClientBuilder};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::interface::{
    PipelineCommand, PipelineValues,
};

use crate::abstract_server::{AbstractServer, ErrorDetails, ErrorLayer, Result, ServerError};

/// Runs the specified
#[derive(Debug, Args)]
pub struct WebTest {
}

#[derive(Debug)]
pub struct WebTestCommand {
    pub args: WebTest,
}

fn print_log(ty: &str, msg: String) {
    let mut stderr = StandardStream::stderr(ColorChoice::Always);

    let color = match ty {
        "INFO" => Color::Blue,
        "PASS" => Color::Green,
        "FAIL" => Color::Red,
        "STACK" => Color::Red,
        "TEST_START" => Color::Yellow,
        "TEST_END" => Color::Yellow,
        _ => Color::Cyan,
    };

    stderr.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
    write!(&mut stderr, "{}", ty).unwrap();

    stderr.reset().unwrap();
    writeln!(&mut stderr, " - {}", msg).unwrap();
}

type TestResult<T> = std::result::Result<T, String>;

impl WebTestCommand {
    async fn setup_webdriver_and_run_tests(&self) -> TestResult<()> {
        let mut caps = serde_json::map::Map::new();
        let opts = serde_json::json!({ "args": ["--headless"] });
        caps.insert("moz:firefoxOptions".to_string(), opts);
        let client = ClientBuilder::native()
            .capabilities(caps)
            .connect("http://localhost:4444").await.map_err(|e| format!("{:?}", e))?;

        let result = self.run_tests(&client).await;

        client.close().await.map_err(|e| format!("{:?}", e))?;

        result.map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    async fn run_tests(&self, client: &Client) -> std::result::Result<(), fantoccini::error::CmdError> {
        let entire_start = Instant::now();
        let mut count = 0;

        let files = fs::read_dir("static/tests/").unwrap();
        for file in files {
            if file.is_err() {
                continue;
            }

            let file = file.unwrap();

            let name = file.file_name().clone().into_string().unwrap();
            if !name.starts_with("test_") {
                continue;
            }
            if !name.ends_with(".js") {
                continue;
            }

            let mut url_path = None;
            let mut timeout: u64 = 30 * 1000;

            let text = match fs::read_to_string(file.path().clone()) {
                Ok(text) => text,
                _ => continue
            };
            for line in text.lines() {
                if line.starts_with("// @@PATH: ") {
                    url_path = line.strip_prefix("// @@PATH: ");
                }
                if line.starts_with("// @@TIMEOUT: ") {
                    timeout = line.strip_prefix("// @@TIMEOUT: ").unwrap().parse().unwrap();
                }
            }

            let url = match url_path {
                Some(path) => format!("http://localhost{}", path),
                None => "http://localhost/".to_string(),
            };

            print_log("INFO", format!("Starting {}", name));

            print_log("INFO", format!("Navigate to {}", url));
            client.goto(url.as_str()).await?;

            let script = r#"
const [name] = arguments;
const s = document.createElement("script");
s.src = "/static/tests/head.js?" + Date.now();
s.addEventListener("load", () => {
  window.TestHarness.loadTest(name);
});
document.body.append(s);
"#;

            let args = vec![
                serde_json::json!(name)
            ];

            client.execute(script, args).await?;

            let script = r#"
if (!("LAST_TEST_LOG_INDEX" in window)) {
  window.LAST_TEST_LOG_INDEX = 0;
}

if (!("TEST_LOG" in window)) {
  return [];
}

const result = window.TEST_LOG.slice(window.LAST_TEST_LOG_INDEX);
window.LAST_TEST_LOG_INDEX = window.TEST_LOG.length;

return result;
"#;

            let start = Instant::now();
            let mut failed = false;

            'test_loop: loop {
                let log_value = client.execute(script, vec![]).await?;
                let log: Vec<(String, String)> = serde_json::value::from_value(log_value)?;
                for (ty, msg) in log {
                    print_log(ty.as_str(), msg);
                    if ty == "FAIL" {
                        failed = true;
                    }
                    if ty == "TEST_END" {
                        break 'test_loop;
                    }
                }
                let elapsed_time = start.elapsed();
                if elapsed_time > Duration::from_millis(timeout) {
                    failed = true;
                    print_log("FAIL", format!("{} | Test timed out", name));
                    break 'test_loop;
                }

                thread::sleep(Duration::from_millis(100));
            }

            if failed {
                let filename = "/tmp/screen.png";
                print_log("INFO", format!("Saving screenshot to {}", filename));
                let data = client.screenshot().await?;
                fs::write(filename, data)?;
            }

            count += 1;
        }

        let elapsed_time = entire_start.elapsed();
        eprintln!("");
        eprintln!("----------------------------------------------------------------------");
        eprintln!("Run {} tests in {:.3}s.", count, elapsed_time.as_millis() as f64 / 1000.0);
        eprintln!("");
        eprintln!("OK");

        Ok(())
    }
}

#[async_trait]
impl PipelineCommand for WebTestCommand {
    async fn execute(
        &self,
        _server: &Box<dyn AbstractServer + Send + Sync>,
        _input: PipelineValues,
    ) -> Result<PipelineValues> {
        self.setup_webdriver_and_run_tests().await.map_err(|e| {
            ServerError::StickyProblem(ErrorDetails {
                layer: ErrorLayer::ConfigLayer,
                message: e,
            })
        })?;

        Ok(PipelineValues::Void)
    }
}
