use crate::crawler::browser;
use crate::models::{NewReview, User};
use crate::utility::debug_webhook;
use anyhow::Result;
use headless_chrome::Tab;
use headless_chrome::protocol::cdp::Page;
use std::thread::sleep;
use std::time::Duration;

struct ReviewText {
    text: String,
    original_text: Option<String>,
}

/// The review that was opened from the contribution list, kept so the page that opens can be
/// checked against it before anything is read from it.
struct ClickedReview {
    /// Google's id for the review, when the markup carries one.
    review_id: Option<String>,
    /// The review's text as the contribution list showed it. List entries are truncated.
    text: String,
}

/// The page that was opened for the clicked review, and the XPath of the review on it.
///
/// Every read of the review is scoped to that XPath. Reading whichever element the page happens to
/// have first instead would report another review's text, stars or photos as this contributor's.
struct OpenedReview {
    url: String,
    /// XPath of the verified review, or `None` when the page carries no review id to scope on.
    scope: Option<String>,
    /// What the contribution list showed for the review that was opened. Kept for logging, and to
    /// tell a review that has no text apart from a text read that found nothing.
    clicked: ClickedReview,
}

static GMAPS_REVIEW_URL: &str = "https://www.google.com/maps/contrib/{}/reviews?hl=en";

/// Google tags every review card with the id of the review it holds, both in the contribution list
/// and in the single review panel. That makes the id usable to pick a review and to recognise the
/// same review again after the page has changed.
const REVIEW_CARD_XPATH: &str = r#"//div[@data-review-id]"#;
/// The contribution list is ordered newest first, so the newest review is the first card.
const NEWEST_REVIEW_XPATH: &str = r#"(//div[@data-review-id])[1]//div[contains(@lang, "en")]"#;
const NEWEST_REVIEW_TEXT_XPATH: &str =
    r#"(//div[@data-review-id])[1]//div[contains(@lang, "en")]/span"#;
/// Any review text on the page. Used when the markup carries no review ids to select on.
const REVIEW_TEXT_XPATH: &str = r#"//div[contains(@lang, "en")]"#;
const REVIEW_TEXT_SPAN_XPATH: &str = r#"//div[contains(@lang, "en")]/span"#;
/// How many characters of the clicked review must reappear on the opened page before the two are
/// accepted as the same review. Only the start of the text is compared: a list entry is truncated
/// and the tail is what truncation cuts off.
const MIN_REVIEW_MATCH_CHARS: usize = 25;
/// How long the contribution list gets to render its review cards before the id-less fallback is
/// used instead. Rendering is asynchronous, so querying once reads a half-built page.
const REVIEW_CARD_TIMEOUT_MS: u64 = 10000;
/// How long the opened review gets to put its review id in the page. The id can arrive after the
/// DOM reports itself ready, so checking once would fail on a page that was still settling.
const REVIEW_VERIFY_TIMEOUT_MS: u64 = 2000;
/// How long the opened review gets to render its text. The review's parts are hydrated separately,
/// so the text can still be missing when the review itself is already there.
const REVIEW_TEXT_TIMEOUT_MS: u64 = 2000;
/// Stored for a review that is a rating without words. Sent as the review's text, as it always was.
const NO_REVIEW_TEXT: &str = "Review doesn't contain text";

pub fn get_latest_review_for_user(gmaps_user: &User) -> Result<NewReview> {
    tracing::debug!(gmaps_id = %gmaps_user.gmaps_id, "Crawling latest review from Google Maps");

    let browser = browser::get(true)?;
    let tab = browser::new_tab(&browser)?;

    let OpenedReview {
        url: review_url,
        scope,
        clicked,
    } = match open_review_page(&tab, gmaps_user) {
        Ok(val) => val,
        Err(err) => return Err(anyhow::anyhow!("Failed to open review page for user {}: {}", gmaps_user.gmaps_id.as_str(), err)),
    };
    let scope = scope.as_deref();

    let ReviewText {
        text: review_text,
        original_text: original_review_text,
    } = retrieve_review_text(&tab, scope, &clicked.text)?;
    tracing::debug!("Retrieved review text: '{}'", review_text);

    let star_count = retrieve_star_count(&tab, scope)?;
    tracing::debug!("Retrieved star rating: {}", star_count);

    let pictures = retrieve_pictures(&tab, scope, 1)?;
    let pictures_json = serde_json::to_value(&pictures)?;
    tracing::debug!("Retrieved pictures: {:?}", pictures);

    // Taken while the review is still on screen: `get_place_name` navigates away from it.
    let screenshot = capture_debug_screenshot(&tab, gmaps_user);

    let place_name = get_place_name(&tab, gmaps_user)?;
    tracing::debug!("Retrieved place name: {}", place_name);

    tracing::debug!(
        gmaps_id = %gmaps_user.gmaps_id,
        place_name = %place_name,
        stars = star_count,
        text_length = review_text.len(),
        picture_count = pictures.len(),
        // Whether the review was verified and read by its id, or by text alone. A run of `false`
        // here is what a layout change would look like before it turns into failures.
        review_id = ?clicked.review_id,
        scoped = scope.is_some(),
        "Crawled review"
    );

    send_debug_screenshot(
        gmaps_user,
        format!(
            "{} — {}\nReview of {} ({} stars)\n{}",
            gmaps_user.name,
            gmaps_user.gmaps_id.as_str(),
            place_name,
            star_count,
            review_url
        ),
        screenshot,
    );

    Ok(NewReview {
        place_name,
        text: review_text,
        original_text: original_review_text,
        stars: star_count,
        user_id: gmaps_user.id,
        link_en: review_url,
        pictures: pictures_json,
    })
}

fn open_review_page(tab: &Tab, gmaps_user: &User) -> Result<OpenedReview> {
    load_review_url(tab, gmaps_user)?;

    let clicked_review = click_newest_review(tab, gmaps_user)?;

    let review_url = load_single_review_page(tab)?;

    // Reading whatever review the page ends up showing is what produced notifications for reviews
    // that could not be found again: the notification links to *this contributor's* review of the
    // place, so a review belonging to anyone else renders as a place page with no review in it.
    match verify_opened_review(tab, &clicked_review) {
        Ok(scope) => Ok(OpenedReview {
            url: review_url,
            scope,
            clicked: clicked_review,
        }),
        Err(e) => {
            let screenshot = capture_debug_screenshot(tab, gmaps_user);
            send_debug_screenshot(
                gmaps_user,
                format!(
                    "{} — {}\nOpened review does not match the clicked review: {e}",
                    gmaps_user.name,
                    gmaps_user.gmaps_id.as_str()
                ),
                screenshot,
            );
            Err(e)
        }
    }
}

/// Opens the newest review in the contribution list and reports what was clicked.
///
/// Waiting for the first element that merely carries a `lang` attribute (which is what this used to
/// do) does not say *which* review it is: the list fills in asynchronously, so that element can be
/// a card that is out of order, or markup that is not a review card at all. Anchoring on a review
/// card keeps the choice deterministic.
fn click_newest_review(tab: &Tab, gmaps_user: &User) -> Result<ClickedReview> {
    let user_id = gmaps_user.gmaps_id.as_str();

    if let Ok(cards) = browser::wait_for_elements(tab, REVIEW_CARD_XPATH, REVIEW_CARD_TIMEOUT_MS)
        && let Some(card) = cards.first()
    {
        let review_id = card
            .get_attribute_value("data-review-id")
            .ok()
            .flatten()
            .filter(|id| !id.is_empty());

        let review_element = tab
            .find_element_by_xpath(NEWEST_REVIEW_XPATH)
            .map_err(|e| {
                anyhow::anyhow!("Failed to find the newest review for user {user_id}: {e}")
            })?;
        let text = element_text(tab, NEWEST_REVIEW_TEXT_XPATH);
        review_element.click().map_err(|e| {
            anyhow::anyhow!("Failed to open the newest review for user {user_id}: {e}")
        })?;
        sleep(Duration::from_secs(1));

        return Ok(ClickedReview { review_id, text });
    }

    // Markup without review ids: fall back to the first element that carries a language attribute.
    // There is no id to recognise the review by afterwards, so this path leans on the text
    // comparison in `verify_opened_review` and cannot scope the reads to a review.
    let elements = browser::wait_for_elements(tab, REVIEW_TEXT_XPATH, REVIEW_CARD_TIMEOUT_MS)
        .map_err(|_| anyhow::anyhow!("No reviews found for user {user_id}"))?;
    let Some(review_element) = elements.first() else {
        return Err(anyhow::anyhow!("No reviews found for user {user_id}"));
    };

    // Only worth reporting once this path is actually taken: a user with no reviews at all reaches
    // this point too, and for them it is not a surprise.
    tracing::warn!(
        gmaps_id = %user_id,
        "No review cards carrying a review id were found; falling back to the first element with a language attribute"
    );

    let text = element_text(tab, REVIEW_TEXT_SPAN_XPATH);
    review_element
        .click()
        .map_err(|e| anyhow::anyhow!("Failed to open a review for user {user_id}: {e}"))?;
    sleep(Duration::from_secs(1));

    Ok(ClickedReview {
        review_id: None,
        text,
    })
}

/// Checks that the page that opened is showing the review that was clicked, so that nothing else on
/// the page can be mistaken for this contributor's newest review, and reports the XPath the review
/// was found at so the reads can be limited to it.
///
/// When the clicked review has an id, the id is the whole answer: either it is on the page or the
/// page is not showing that review. The text comparison only covers markup that carries no id.
fn verify_opened_review(tab: &Tab, clicked_review: &ClickedReview) -> Result<Option<String>> {
    if let Some(xpath) = clicked_review
        .review_id
        .as_deref()
        .and_then(review_panel_xpath)
    {
        // Waited for, not queried once: the id is written into the page as the panel renders, so a
        // single check would fail on a page that is still filling in.
        if browser::wait_for_elements(tab, &xpath, REVIEW_VERIFY_TIMEOUT_MS).is_ok() {
            return Ok(Some(xpath));
        }

        let opened_text = element_text(tab, REVIEW_TEXT_SPAN_XPATH);
        return Err(anyhow::anyhow!(
            "The opened page does not contain the clicked review {:?}; it shows {:?}",
            clicked_review.review_id.as_deref().map(truncate_for_log),
            truncate_for_log(&opened_text)
        ));
    }

    if clicked_review.text.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "The review that was clicked could not be read from the contribution list, so the opened page cannot be checked against it"
        ));
    }

    let opened_text = element_text(tab, REVIEW_TEXT_SPAN_XPATH);
    if panel_shows_clicked_review(&clicked_review.text, &opened_text) {
        return Ok(None);
    }

    Err(anyhow::anyhow!(
        "The opened review is a different review than the one that was clicked (clicked {:?}, opened {:?})",
        truncate_for_log(&clicked_review.text),
        truncate_for_log(&opened_text)
    ))
}

/// Builds an XPath that looks inside the verified review when there is one, so that a read cannot
/// pick up a review that happens to be elsewhere on the page.
fn scoped_xpath(scope: Option<&str>, xpath: &str) -> String {
    match scope {
        Some(scope) => format!("{scope}{xpath}"),
        None => xpath.to_string(),
    }
}

/// XPath matching the review with `review_id`, or `None` when the id cannot be embedded in one.
fn review_panel_xpath(review_id: &str) -> Option<String> {
    if review_id.is_empty() || review_id.contains('"') {
        return None;
    }

    Some(format!(r#"//div[@data-review-id="{review_id}"]"#))
}

/// Reads the text of the first element matching `xpath`, or an empty string when there is none.
fn element_text(tab: &Tab, xpath: &str) -> String {
    match tab.find_element_by_xpath(xpath) {
        Ok(element) => element.get_inner_text().unwrap_or_default(),
        Err(_) => String::new(),
    }
}

/// Whether the review on the opened page is the one that was clicked.
///
/// A list entry is truncated ("… More") and may repeat the place name above the review, while the
/// panel shows the review in full and without that heading, so only the start of the review text
/// can be compared.
fn panel_shows_clicked_review(clicked_text: &str, opened_text: &str) -> bool {
    let clicked = collapse_whitespace(review_body(strip_truncation_marker(clicked_text)));
    if clicked.is_empty() {
        return false;
    }

    let needle: String = clicked.chars().take(MIN_REVIEW_MATCH_CHARS).collect();
    collapse_whitespace(opened_text).contains(&needle)
}

/// Drops the heading a contribution list entry puts above the review, which the panel does not
/// repeat. Text without a line break is returned as it is.
fn review_body(text: &str) -> &str {
    match text.split_once('\n') {
        Some((_heading, body)) => body,
        None => text,
    }
}

/// Removes the marker the contribution list appends to a review it truncated.
fn strip_truncation_marker(text: &str) -> &str {
    let without_more = text
        .trim_end()
        .strip_suffix("More")
        .map(str::trim_end)
        .unwrap_or_else(|| text.trim_end());

    without_more
        .strip_suffix('…')
        .map(str::trim_end)
        .unwrap_or(without_more)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_for_log(text: &str) -> String {
    text.chars().take(80).collect()
}

/// Screenshots the page for the debug webhook, if one is configured. A debug aid must never fail a
/// crawl, so a missing screenshot is only logged.
fn capture_debug_screenshot(tab: &Tab, gmaps_user: &User) -> Option<Vec<u8>> {
    if !debug_webhook::is_enabled() {
        return None;
    }

    match tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true) {
        Ok(screenshot) => Some(screenshot),
        Err(e) => {
            tracing::warn!(gmaps_id = %gmaps_user.gmaps_id, error = %e, "Failed to capture a debug screenshot");
            None
        }
    }
}

fn send_debug_screenshot(gmaps_user: &User, context: String, screenshot: Option<Vec<u8>>) {
    let Some(screenshot) = screenshot else {
        return;
    };

    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        tracing::warn!(gmaps_id = %gmaps_user.gmaps_id, "No async runtime available; skipping the debug screenshot");
        return;
    };

    // Sent in the background: a debug aid must not add the webhook's response time (or a timeout)
    // to a crawl. The bot's runtime outlives the task, so it does not need to be awaited.
    handle.spawn(async move {
        if let Err(e) = debug_webhook::send_screenshot(&context, screenshot).await {
            tracing::warn!(error = %e, "Failed to send the debug screenshot");
        }
    });
}

fn load_review_url(tab: &Tab, gmaps_user: &User) -> Result<()> {
    let review_url = GMAPS_REVIEW_URL.replace("{}", gmaps_user.gmaps_id.as_ref());
    match tab.navigate_to(review_url.as_str()) {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to navigate to review page for user {}: {}",
                gmaps_user.gmaps_id.as_str(),
                e
            ));
        }
    }

    match browser::wait_for_url(tab, "reviews/@", 10000) {
        Ok(()) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to load review page for user {}: {}",
                gmaps_user.gmaps_id.as_str(),
                e
            ));
        }
    }

    match browser::wait_dom_ready(tab, 10000) {
        Ok(()) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "DOM not ready on review page for user {}: {}",
                gmaps_user.gmaps_id.as_str(),
                e
            ));
        }
    }

    tracing::debug!("Loaded review page: {}", tab.get_url());
    Ok(())
}

fn load_single_review_page(tab: &Tab) -> Result<String> {
    tracing::debug!("Loading single review page: {}", tab.get_url());

    match browser::wait_for_url_regex(
        tab,
        &regex::Regex::new(r"/place/[a-zA-Z0-9-_]+/@.*")?,
        10000,
    ) {
        Ok(()) => (),
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to load single review page: {e}"));
        }
    }

    match browser::wait_dom_ready(tab, 10000) {
        Ok(()) => (),
        Err(e) => {
            return Err(anyhow::anyhow!("DOM not ready on single review page: {e}"));
        }
    }

    tracing::debug!("Loaded single review page: {}", tab.get_url());
    Ok(tab.get_url())
}

fn retrieve_review_text(tab: &Tab, scope: Option<&str>, clicked_text: &str) -> Result<ReviewText> {
    tracing::debug!("Retrieving review text from page");

    // The panel is waited on for its review id, not for everything inside it, so the text node can
    // still be on its way while this runs. Waiting here as well keeps a slow render from reading as
    // a review without text.
    let text_nodes = match scope {
        Some(_) => browser::wait_for_elements(
            tab,
            &scoped_xpath(scope, REVIEW_TEXT_XPATH),
            REVIEW_TEXT_TIMEOUT_MS,
        ),
        None => tab.find_elements_by_xpath(&scoped_xpath(scope, REVIEW_TEXT_XPATH)),
    }
    .unwrap_or_default();

    if text_nodes.is_empty() {
        // A review can be a rating without words, and then it has no text node at all. If the
        // contribution list did show text for it, that is not what this is: the page is not shaped
        // the way this code expects, and the notification will carry the placeholder as the review.
        if !clicked_text.trim().is_empty() {
            tracing::warn!(
                clicked = %truncate_for_log(clicked_text),
                "The opened review has no text node, but the contribution list showed text for it"
            );
        }

        return Ok(ReviewText {
            text: NO_REVIEW_TEXT.to_string(),
            original_text: None,
        });
    }

    let review_text = tab
        .find_element_by_xpath(&scoped_xpath(scope, REVIEW_TEXT_SPAN_XPATH))
        .and_then(|element| element.get_inner_text())
        .map_err(|e| {
            anyhow::anyhow!("Failed to read the review text from the opened page: {e}")
        })?;
    tracing::debug!("Retrieved review text element");

    // Scoped to the verified review as well: the button belongs to the review it sits in, and a
    // page showing more than one review has one of these per review.
    let show_original_button = tab.find_element_by_xpath(&scoped_xpath(
        scope,
        r#"//button[contains(@role, "switch")]/span[contains(text(), "original")]/.."#,
    ));
    let original_review_text = match show_original_button {
        Ok(button) => {
            tracing::debug!("Found 'Show original' button, clicking to reveal original text");
            match button.click() {
                Ok(_) => (),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to click 'Show original' button; using translated text");
                    return Ok(ReviewText {
                        text: review_text,
                        original_text: None,
                    });
                }
            }
            sleep(Duration::from_secs(1));
            // Read inside the verified review rather than by climbing out of the button: a `..`
            // step leaves the review's subtree, and from there the read can reach another review.
            // The original is by definition not English, and matching on that picks out the review
            // text rather than whatever else in the card carries a language.
            let original_text_xpath =
                scoped_xpath(scope, r#"//div[@lang and not(contains(@lang, "en"))]/span"#);
            // Falling back to the translated text keeps the stored original text readable: it says
            // less than it should, rather than claiming the review has no text at all.
            let original_review_text = match tab.find_element_by_xpath(&original_text_xpath) {
                Ok(elem) => elem
                    .get_inner_text()
                    .unwrap_or_else(|_| review_text.clone()),
                Err(e) => {
                    tracing::debug!(error = %e, "Could not read the original text; using the translation");
                    review_text.clone()
                }
            };
            Some(original_review_text)
        }
        Err(_) => None,
    };

    Ok(ReviewText {
        text: review_text,
        original_text: original_review_text,
    })
}

fn retrieve_star_count(tab: &Tab, scope: Option<&str>) -> Result<i32> {
    let stars_xpath = scoped_xpath(
        scope,
        r#"//span[contains(@aria-label, " star")]/span[contains(@class, "google-symbols")]"#,
    );
    let Ok(stars_span) = tab.find_elements_by_xpath(&stars_xpath) else {
        return Err(anyhow::anyhow!(
            "Failed to find star rating elements for review"
        ));
    };
    let Some(first_star) = stars_span.first() else {
        return Err(anyhow::anyhow!("Failed to find star rating element"));
    };

    let Ok(Some(valid_star_classes)) = first_star.get_attribute_value("class") else {
        return Err(anyhow::anyhow!("Failed to get valid star class"));
    };

    let mut star_count = 0;
    for star in stars_span {
        let class_value = star.get_attribute_value("class").ok().flatten();
        if let Some(class) = class_value
            && class == valid_star_classes
        {
            star_count += 1;
        }
    }

    Ok(star_count)
}

fn retrieve_pictures(tab: &Tab, scope: Option<&str>, depth: i32) -> Result<Vec<String>> {
    if depth > 10 {
        return Err(anyhow::anyhow!("Exceeded maximum depth while retrieving pictures, possible infinite loop"));
    }

    tracing::debug!("Retrieving pictures");
    let pictures_xpath = scoped_xpath(scope, r"//div/button[@data-photo-index]");
    let picture_elements = match tab.find_elements_by_xpath(&pictures_xpath) {
        Ok(elements) => elements,
        Err(e) => {
            tracing::debug!("No picture elements found for review: {e}");
            return Ok(vec![]);
        }
    };

    if picture_elements.is_empty() {
        return Ok(vec![]);
    }
    for picture_element in &picture_elements {
        let aria_label = match picture_element.get_attribute_value("aria-label") {
            Ok(aria_label) => if let Some(aria_label) = aria_label { aria_label } else {
                tracing::warn!("Picture element does not have an aria-label attribute");
                continue;
            },
            Err(err) => {
                tracing::warn!(error = %err, "Failed to get aria-label for picture element");
                continue;
            }
        };

        // Check if there are more images to be found
        if aria_label.starts_with('+') {
            picture_element.click().ok();
            sleep(Duration::from_millis(500));
            return retrieve_pictures(tab, scope, depth + 1);
        }
    }

    let re = regex::Regex::new(r#"background-image:\s*url\((?:&quot;|")?(https?://[^"]+)(?:&quot;|")?\)"#)?;
    let mut pictures = vec![];
    for picture_element in &picture_elements {
        let style = match picture_element.get_attribute_value("style") {
            Ok(style) => if let Some(s) = style { s } else {
                tracing::warn!("Picture element does not have a style attribute");
                continue;
            },
            Err(err) => {
                tracing::warn!(error = %err, "Failed to get style attribute for picture element");
                continue;
            }
        };

        if let Some(caps) = re.captures(&style) {
            if let Some(url) = caps.get(1) {
                // Strip the size suffix from the last '=' onwards; keep the URL when there is none.
                let url_str = url.as_str();
                let clean_url = url_str.rsplit_once('=').map_or(url_str, |(base, _)| base);
                pictures.push(clean_url.to_string());
            } else {
                tracing::warn!(style = %style, "Failed to extract URL from style attribute");
            }
        } else {
            tracing::warn!(style = %style, "Style attribute does not match expected format");
        }
    }

    Ok(pictures)
}

fn get_place_name(tab: &Tab, gmaps_user: &User) -> Result<String> {
    let place_details_button =
        match tab.find_element_by_xpath(r#"//div[contains(@jsaction, "placeNameHeader")]"#) {
            Ok(button) => button,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to find place details button for user {}: {}",
                    gmaps_user.gmaps_id.as_str(),
                    e
                ));
            }
        };

    match place_details_button.click() {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to click place details button for user {}: {}",
                gmaps_user.gmaps_id.as_str(),
                e
            ));
        }
    }
    browser::wait_for_url_regex(tab, &regex::Regex::new(r"maps/place/.+/@.*")?, 10000)?;
    browser::wait_dom_ready(tab, 10000)?;
    let place_name =
        get_place_name_from_url(&tab.get_url()).unwrap_or_else(|| "Unknown Place".to_string());

    match tab.evaluate("window.history.back();", false) {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(
                "Failed to navigate back to review page for user {}: {}",
                gmaps_user.gmaps_id.as_str(),
                e
            );
        }
    }
    Ok(place_name)
}

fn get_place_name_from_url(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"/place/([^/]+)/@").ok()?;
    let caps = re.captures(url)?;
    match caps.get(1).map(|m| m.as_str().to_string()) {
        Some(mut name) => {
            name = urlencoding::decode(&name)
                .unwrap_or_else(|_| "Unknown Place".into())
                .to_string();
            name = name.replace('+', " ");
            Some(name)
        }
        None => Some("Unknown Place".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_place_name_from_url, panel_shows_clicked_review, review_panel_xpath, scoped_xpath,
    };

    #[test]
    fn get_place_name_from_url_decodes_encoded_characters() {
        let url = "https://www.google.com/maps/place/Caf%C3%A9+de+Flore/@48.854,2.333,17z";
        assert_eq!(get_place_name_from_url(url), Some("Café de Flore".to_string()));
    }

    #[test]
    fn get_place_name_from_url_returns_none_for_non_place_urls() {
        let url = "https://www.google.com/maps/search/coffee/@48.854,2.333,17z";
        assert_eq!(get_place_name_from_url(url), None);
    }

    #[test]
    fn get_place_name_from_url_falls_back_for_invalid_encoding() {
        let url = "https://www.google.com/maps/place/%E0%A4%A/@48.854,2.333,17z";
        assert_eq!(get_place_name_from_url(url), Some("Unknown Place".to_string()));
    }

    #[test]
    fn get_place_name_from_url_decodes_symbols_and_spaces() {
        let url = "https://www.google.com/maps/place/AT%26T+Store/@40.0,-73.0,16z";
        assert_eq!(get_place_name_from_url(url), Some("AT&T Store".to_string()));
    }

    #[test]
    fn get_place_name_from_url_returns_none_when_at_segment_is_missing() {
        let url = "https://www.google.com/maps/place/Cafe+Noir";
        assert_eq!(get_place_name_from_url(url), None);
    }

    #[test]
    fn review_panel_xpath_matches_the_review_with_that_id() {
        assert_eq!(
            review_panel_xpath("Ci9DQUlRQUNvZENodHlj"),
            Some(r#"//div[@data-review-id="Ci9DQUlRQUNvZENodHlj"]"#.to_string())
        );
    }

    #[test]
    fn review_panel_xpath_rejects_ids_it_cannot_embed() {
        assert_eq!(review_panel_xpath(""), None);
        assert_eq!(review_panel_xpath(r#"a"b"#), None);
    }

    #[test]
    fn review_panel_xpath_embeds_ids_that_only_need_the_quotes_around_them() {
        assert_eq!(
            review_panel_xpath("a]b'c\nd"),
            Some("//div[@data-review-id=\"a]b'c\nd\"]".to_string())
        );
    }

    #[test]
    fn scoped_xpath_limits_a_read_to_the_verified_review() {
        assert_eq!(
            scoped_xpath(Some(r#"//div[@data-review-id="abc"]"#), r#"//div[@lang]/span"#),
            r#"//div[@data-review-id="abc"]//div[@lang]/span"#
        );
        assert_eq!(
            scoped_xpath(None, r#"//div[@lang]/span"#),
            r#"//div[@lang]/span"#
        );
    }

    #[test]
    fn panel_shows_clicked_review_accepts_a_truncated_list_entry() {
        let list_entry = "McDonald's\n\nWhat a mess at McDonald's. I have truly rarely seen such a filthy mess. … More";
        let opened = "What a mess at McDonald's. I have truly rarely seen such a filthy mess.\n\nThe food was just cold.";

        assert!(panel_shows_clicked_review(list_entry, opened));
    }

    #[test]
    fn panel_shows_clicked_review_accepts_an_entry_without_a_heading() {
        let list_entry = "The tables are sticky and the floor is sticky. … More";
        let opened =
            "The tables are sticky and the floor is sticky. The whole restaurant looks unclean.";

        assert!(panel_shows_clicked_review(list_entry, opened));
    }

    #[test]
    fn panel_shows_clicked_review_accepts_a_short_entry_whose_whole_text_carries_the_marker() {
        // Short enough that the truncation marker lands inside the compared prefix, so the
        // comparison only works if the marker is stripped off.
        let list_entry = "Cold fries. … More";
        let opened = "Cold fries. The place was packed, so I did not go back.";

        assert!(panel_shows_clicked_review(list_entry, opened));
    }

    #[test]
    fn panel_shows_clicked_review_rejects_a_different_review() {
        let list_entry = "McDonald's\n\nWhat a mess at McDonald's. I have truly rarely seen such a filthy mess. … More";
        let opened = "Great fries, friendly staff, clean restaurant. Would visit again.";

        assert!(!panel_shows_clicked_review(list_entry, opened));
    }

    #[test]
    fn panel_shows_clicked_review_rejects_an_empty_page() {
        assert!(!panel_shows_clicked_review("", ""));
        assert!(!panel_shows_clicked_review("A review", ""));
    }
}
