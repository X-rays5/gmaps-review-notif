use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

pub struct AutoClosableTab {
    tab: Arc<headless_chrome::Tab>,
}

impl Drop for AutoClosableTab {
    fn drop(&mut self) {
        let _ = self.tab.close(false);
    }
}

impl Deref for AutoClosableTab {
    type Target = Arc<headless_chrome::Tab>;
    fn deref(&self) -> &Self::Target {
        &self.tab
    }
}

pub fn get(accept_terms: bool) -> Result<Browser> {
    let browser = Browser::new(LaunchOptions {
        headless: false,
        window_size: Some((1920, 1080)),
        args: vec![
            "--headless=new".as_ref(),
            "--no-sandbox".as_ref(),
            "--lang=en-US".as_ref(),
        ],
        ..Default::default()
    });

    match browser {
        Ok(b) => {
            if accept_terms {
                accept_gmaps_terms(&b)?;
            }
            Ok(b)
        }
        Err(e) => Err(anyhow::anyhow!("Failed to launch browser: {e}")),
    }
}

pub fn new_tab(browser: &Browser) -> Result<AutoClosableTab> {
    let tab = browser.new_tab()?;
    match tab.enable_stealth_mode() {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to enable stealth mode: {e}"));
        }
    }
    tab.set_default_timeout(Duration::from_secs(10));
    Ok(AutoClosableTab { tab })
}

pub fn accept_gmaps_terms(browser: &Browser) -> Result<()> {
    let tab = new_tab(browser)?;

    tab.navigate_to("https://www.google.com/maps?hl=en")?;
    tab.wait_until_navigated()?;

    while tab.get_url().contains("consent.google.com") {
        let accept_button = tab.wait_for_xpath(r#"//form[contains(@action, "consent.google.com")]//button[contains(@aria-label, "Accept all")]"#);
        match accept_button {
            Ok(button) => {
                button.click()?;
                tab.wait_until_navigated()?;
                wait_dom_ready(&tab, 10000)?;
            }
            Err(e) => {
                if tab.get_url().contains("consent.google.com") {
                    return Err(anyhow::anyhow!(
                        "Accept button not found on terms page: {e}"
                    ));
                }
                break;
            }
        }
    }

    Ok(())
}

pub fn wait_for_url(
    tab: &headless_chrome::Tab,
    url_substring: &str,
    timeout_ms: u64,
) -> Result<()> {
    tracing::debug!("Waiting for URL: {}", url_substring);
    let start = std::time::Instant::now();
    while !tab.get_url().contains(url_substring) {
        if start.elapsed().as_millis() > u128::from(timeout_ms) {
            tracing::debug!("Current URL: {}", tab.get_url());
            return Err(anyhow::anyhow!(
                "Timeout waiting for URL containing: {url_substring}"
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    tracing::debug!("URL contains: {}", url_substring);
    Ok(())
}

pub fn wait_for_url_regex(
    tab: &headless_chrome::Tab,
    url_pattern: &regex::Regex,
    timeout_ms: u64,
) -> Result<()> {
    tracing::debug!("Waiting for URL matching pattern: {}", url_pattern.as_str());
    let start = std::time::Instant::now();
    while !url_pattern.is_match(&tab.get_url()) {
        if start.elapsed().as_millis() > u128::from(timeout_ms) {
            tracing::debug!("Current URL: {}", tab.get_url());
            return Err(anyhow::anyhow!(
                "Timeout waiting for URL matching pattern: {}",
                url_pattern.as_str()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    tracing::debug!("URL matches pattern: {}", url_pattern.as_str());
    Ok(())
}

pub fn wait_dom_ready(tab: &headless_chrome::Tab, timeout_ms: u64) -> Result<()> {
    tracing::debug!("Waiting for DOM ready");
    let start = std::time::Instant::now();
    let mut ready_state = document_ready_state(tab)?;
    while ready_state != "complete" {
        if start.elapsed().as_millis() > u128::from(timeout_ms) {
            tracing::debug!(ready_state = %ready_state, "Timed out waiting for the DOM to be ready");
            return Err(anyhow::anyhow!("Timeout waiting for DOM ready"));
        }
        std::thread::sleep(Duration::from_millis(100));
        ready_state = document_ready_state(tab)?;
    }

    tracing::debug!("DOM is ready");
    Ok(())
}

/// Reads `document.readyState`. A missing or non-string result reads as "not ready" so the caller
/// keeps waiting and eventually reports a timeout instead of failing on a surprise page value.
fn document_ready_state(tab: &headless_chrome::Tab) -> Result<String> {
    let evaluated = tab.evaluate("document.readyState", false)?;
    let state = evaluated
        .value
        .as_ref()
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    Ok(state.to_string())
}
