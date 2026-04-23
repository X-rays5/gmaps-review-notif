use crate::crawler::browser;
use crate::models::{NewReview, User};
use anyhow::Result;
use headless_chrome::Tab;
use std::thread::sleep;
use std::time::Duration;

struct ReviewText {
    text: String,
    original_text: Option<String>,
}

static GMAPS_REVIEW_URL: &str = "https://www.google.com/maps/contrib/{}/reviews?hl=en";

pub fn get_latest_review_for_user(gmaps_user: &User) -> Result<NewReview> {
    let browser = browser::get(true)?;
    let tab = browser::new_tab(&browser)?;

    let review_url = match open_review_page(&tab, gmaps_user) {
        Ok(val) => val,
        Err(err) => return Err(anyhow::anyhow!("Failed to open review page for user {}: {}", gmaps_user.gmaps_id.as_str(), err)),
    };

    let ReviewText {
        text: review_text,
        original_text: original_review_text,
    } = retrieve_review_text(&tab);
    tracing::debug!("Retrieved review text: '{}'", review_text);

    let star_count = retrieve_star_count(&tab)?;
    tracing::debug!("Retrieved star rating: {}", star_count);

    let pictures = retrieve_pictures(&tab, 1)?;
    let pictures_json = serde_json::to_value(&pictures)?;
    tracing::debug!("Retrieved pictures: {:?}", pictures);

    let place_name = get_place_name(&tab, gmaps_user)?;
    tracing::debug!("Retrieved place name: {}", place_name);

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

fn open_review_page(tab: &Tab, gmaps_user: &User) -> Result<String> {
    load_review_url(tab, gmaps_user)?;

    match tab
        .wait_for_elements_by_xpath(r#"//div[contains(@lang, "en")]"#)?
        .first()
    {
        Some(review_element) => {
            review_element.click()?;
            sleep(Duration::from_secs(1));
        }
        None => {
            return Err(anyhow::anyhow!(
                "No reviews found for user {}",
                gmaps_user.gmaps_id.as_str()
            ));
        }
    }

    load_single_review_page(tab)
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

fn retrieve_review_text(tab: &Tab) -> ReviewText {
    tracing::debug!("Retrieving review text from page");

    let review_text = match tab.find_element_by_xpath(r#"//div[contains(@lang, "en")]/span"#) {
        Ok(elem) => elem
            .get_inner_text()
            .unwrap_or_else(|_| "Review doesn't contain text".to_string()),
        Err(_) => "Review doesn't contain text".to_string(),
    };
    tracing::debug!("Retrieved review text element");

    let show_original_button = tab.find_element_by_xpath(
        r#"//button[contains(@role, "switch")]/span[contains(text(), "original")]"#,
    );
    let original_review_text = match show_original_button {
        Ok(button) => {
            tracing::debug!("Found 'Show original' button, clicking to reveal original text");
            match button.click() {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to click 'Show original' button: {}", e);
                    return ReviewText {
                        text: review_text,
                        original_text: None,
                    };
                }
            }
            sleep(Duration::from_secs(1));
            let original_review_text = match tab.find_element_by_xpath(r#"//button[contains(@role, "switch")]/span[contains(text(), "translation")]/../../..//div[@lang]/span"#) {
                Ok(elem) => elem.get_inner_text().unwrap_or_else(|_| "Review doesn't contain text".to_string()),
                Err(_) => "Review doesn't contain text".to_string(),
            };
            Some(original_review_text)
        }
        Err(_) => None,
    };

    ReviewText {
        text: review_text,
        original_text: original_review_text,
    }
}

fn retrieve_star_count(tab: &Tab) -> Result<i32> {
    let Ok(stars_span) = tab.find_elements_by_xpath(
        r#"//span[contains(@aria-label, " star")]/span[contains(@class, "google-symbols")]"#,
    ) else {
        return Err(anyhow::anyhow!(
            "Failed to find star rating elements for review"
        ));
    };
    if stars_span.is_empty() {
        return Err(anyhow::anyhow!("Failed to find star rating element"));
    }

    let Ok(Some(valid_star_classes)) = stars_span[0].get_attribute_value("class") else {
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

fn retrieve_pictures(tab: &Tab, depth: i32) -> Result<Vec<String>> {
    if depth > 10 {
        return Err(anyhow::anyhow!("Exceeded maximum depth while retrieving pictures, possible infinite loop"));
    }

    tracing::debug!("Retrieving pictures");
    let picture_elements = match tab.find_elements_by_xpath(r"//div/button[@data-photo-index]") {
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
                tracing::error!("Picture element does not have an aria-label attribute");
                continue;
            },
            Err(err) => {
                tracing::error!("Failed to get aria-label for picture element: {}", err);
                continue;
            }
        };

        // Check if there are more images to be found
        if aria_label.starts_with('+') {
            picture_element.click().ok();
            sleep(Duration::from_millis(500));
            return retrieve_pictures(tab, depth + 1);
        }
    }

    let re = regex::Regex::new(r#"background-image:\s*url\((?:&quot;|")?(https?://[^"]+)(?:&quot;|")?\)"#)?;
    let mut pictures = vec![];
    for picture_element in &picture_elements {
        let style = match picture_element.get_attribute_value("style") {
            Ok(style) => if let Some(s) = style { s } else {
                tracing::error!("Picture element does not have a style attribute");
                continue;
            },
            Err(err) => {
                tracing::error!("Failed to get style attribute for picture element: {}", err);
                continue;
            }
        };

        if let Some(caps) = re.captures(&style) {
            if let Some(url) = caps.get(1) {
                let clean_url = if let Some(idx) = url.as_str().rfind('=') {
                    &url.as_str()[..idx] // Remove everything from '=' onwards
                } else {
                    url.as_str() // If no '=' is found, use the full URL
                };
                pictures.push(clean_url.to_string());
            } else {
                tracing::error!("Failed to extract URL from style attribute: {}", style);
            }
        } else {
            tracing::error!("Style attribute does not match expected format: {}", style);
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
    use super::get_place_name_from_url;

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
}

