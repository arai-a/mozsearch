use std::path::Path;
use std::time::Instant;

use async_trait::async_trait;
use clap::Args;
use stringmatch::Needle;
use thirtyfour::extensions::query::ElementWaiter;
use thirtyfour::prelude::*;
use thirtyfour::stringmatch::StringMatchable;

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

/// The condition to wait for query parameter contents.
trait HasQueryParam {
    async fn has_query_param<N>(self, text: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static;
}
impl HasQueryParam for ElementWaiter {
    async fn has_query_param<N>(self, text: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static {
        self.condition(Box::new(move |elem| {
            let text = text.clone();
            Box::pin(async move {
                let url = elem.handle.current_url().await?;
                Ok(match url.query() {
                    Some(query) => text.is_match(query),
                    None => false,
                })
            })
        })).await?;
        Ok(())
    }
}

// Various macros to make it easier to write tests.

/// Run tests method on self with logging.
#[macro_export]
macro_rules! call_test {
    ( $self:ident, $count:ident, $driver:ident, $name:ident ) => {
        {
            eprint!("{} ... ", stringify!($name));
            $self.$name(&$driver).await?;
            eprintln!("ok");
            $count += 1;
        }
    };
}

/// Await on expr.
/// When it fails,  print details and also save the screenshot of the browser.
#[macro_export]
macro_rules! check {
    ( $driver:ident, $expr:expr ) => {
        {
            let result = $expr.await;
            match &result {
                Err(e) => {
                    eprintln!(" error");
                    eprintln!("  Error: {:?}", e);
                    eprintln!("   Expr: {}", stringify!($expr));
                    eprintln!("     At: {}:{}", file!(), line!());

                    eprintln!("  Saving screenshot to /tmp/screen.png");
                    $driver.screenshot(Path::new("/tmp/screen.png")).await?;
                }
                _ => {}
            }
            result?
        }
    };
}

// Wrappers for check! macro.

/// Navigate to the URL.
#[macro_export]
macro_rules! goto {
    ( $driver:ident, $url:expr ) => {
        {
            check!($driver, $driver.goto($url));
        }
    };
}

/// Returns the element with given ID.
#[macro_export]
macro_rules! find_by_id {
    ( $driver:ident, $id:expr ) => {
        {
            check!($driver, $driver.find(By::Id($id)))
        }
    };
}

/// Wait until the given text is found in given element.
#[macro_export]
macro_rules! wait_text {
    ( $driver:ident, $elem:ident, $text:expr ) => {
        {
            check!($driver, $elem.wait_until().has_text($text.match_partial()));
        }
    };
}

/// Wait until the given text is found in the query parameter.
#[macro_export]
macro_rules! wait_query_param {
    ( $driver:ident, $elem:ident, $text:expr ) => {
        {
            check!($driver, $elem.wait_until().has_query_param($text.match_partial()));
        }
    };
}

impl WebTestCommand {
    async fn setup_webdriver_and_run_tests(&self) -> WebDriverResult<()> {
        let mut caps = DesiredCapabilities::firefox();
        caps.set_headless()?;
        let driver = WebDriver::new("http://localhost:4444", caps).await?;

        let result = self.run_tests(&driver).await;
        // quit should be performed even for error case.
        driver.quit().await?;
        result?;

        Ok(())
    }

    async fn run_tests(&self, driver: &WebDriver) -> WebDriverResult<()> {
        let start = Instant::now();
        let mut count = 0;

        call_test!(self, count, driver, test_simple_search);
        call_test!(self, count, driver, test_case_sensitiveness);
        call_test!(self, count, driver, test_regexp);
        call_test!(self, count, driver, test_path_filter);

        let elapsed_time = start.elapsed();
        eprintln!("");
        eprintln!("----------------------------------------------------------------------");
        eprintln!("Run {} tests in {:.3}s.", count, elapsed_time.as_millis() as f64 / 1000.0);
        eprintln!("");
        eprintln!("OK");

        Ok(())
    }

    async fn test_simple_search(&self, driver: &WebDriver) -> WebDriverResult<()> {
        goto!(driver, "http://localhost/");

        let query = find_by_id!(driver, "query");
        check!(driver, query.send_keys("SimpleSearch"));

        let content = check!(driver, driver.find(By::Id("content")));

        wait_text!(driver, content, "Core code (1 lines");
        wait_text!(driver, content, "class SimpleSearch");

        wait_query_param!(driver, content, "SimpleSearch");

        Ok(())
    }

    async fn test_case_sensitiveness(&self, driver: &WebDriver) -> WebDriverResult<()> {
        goto!(driver, "http://localhost/");

        let query = find_by_id!(driver, "query");
        check!(driver, query.send_keys("CaseSensitiveness"));

        let content = find_by_id!(driver, "content");

        wait_text!(driver, content, "Core code (2 lines");
        wait_text!(driver, content, "class CaseSensitiveness1");
        wait_text!(driver, content, "class casesensitiveness2");

        wait_query_param!(driver, content, "CaseSensitiveness");
        wait_query_param!(driver, content, "case=false");

        let case_checkbox = find_by_id!(driver, "case");
        check!(driver, case_checkbox.click());

        wait_text!(driver, content, "Core code (1 lines");
        wait_text!(driver, content, "class CaseSensitiveness1");

        wait_query_param!(driver, content, "CaseSensitiveness");
        wait_query_param!(driver, content, "case=true");

        Ok(())
    }

    async fn test_regexp(&self, driver: &WebDriver) -> WebDriverResult<()> {
        goto!(driver, "http://localhost/");

        let query = find_by_id!(driver, "query");
        check!(driver, query.send_keys("Simpl.Search"));

        let content = find_by_id!(driver, "content");

        wait_text!(driver, content, "No results for current query");

        wait_query_param!(driver, content, "Simpl.Search");
        wait_query_param!(driver, content, "regexp=false");

        let regexp_checkbox = find_by_id!(driver, "regexp");
        check!(driver, regexp_checkbox.click());

        wait_text!(driver, content, "Core code (1 lines");
        wait_text!(driver, content, "class SimpleSearch");

        wait_query_param!(driver, content, "Simpl.Search");
        wait_query_param!(driver, content, "regexp=true");

        Ok(())
    }

    async fn test_path_filter(&self, driver: &WebDriver) -> WebDriverResult<()> {
        goto!(driver, "http://localhost/");

        let query = find_by_id!(driver, "query");
        check!(driver, query.send_keys("PathFilter"));

        let content = find_by_id!(driver, "content");

        wait_text!(driver, content, "Core code (2 lines");
        wait_text!(driver, content, "class PathFilter");
        wait_text!(driver, content, "WebTest.cpp");
        wait_text!(driver, content, "WebTestPathFilter.cpp");

        wait_query_param!(driver, content, "PathFilter");
        wait_query_param!(driver, content, "path=&");

        let query = find_by_id!(driver, "path");
        check!(driver, query.send_keys("Filter.cpp"));

        wait_text!(driver, content, "Core code (1 lines");
        wait_text!(driver, content, "class PathFilter");
        wait_text!(driver, content, "WebTestPathFilter.cpp");

        wait_query_param!(driver, content, "PathFilter");
        wait_query_param!(driver, content, "path=Filter.cpp&");

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
                message: format!("{}", e),
            })
        })?;

        Ok(PipelineValues::Void)
    }
}
