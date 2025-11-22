use std::thread::sleep;
use std::time::Duration;
use anyhow::Result;
use crate::crawler::browser;
use crate::models::*;

static GMAPS_REVIEW_URL: &str = "https://www.google.com/maps/contrib/{}/reviews?hl=en";

pub fn get_latest_review_for_user(gmaps_user: User) -> Result<NewReview> {
    let browser = browser::get(true)?;
    let tab = browser::new_tab(&browser)?;

    let review_url = GMAPS_REVIEW_URL.replace("{}", gmaps_user.gmaps_id.as_ref());

    tab.navigate_to(review_url.as_str())?;
    browser::wait_for_url(&tab, "reviews/@", 10000)?;
    browser::wait_dom_ready(&tab, 10000)?;

    match tab.wait_for_elements_by_xpath(r#"//div[contains(@lang, "en")]"#)?.first() {
        Some(review_element) => {
            review_element.click()?;
            sleep(Duration::from_secs(1));
        }
        None => {
            return Err(anyhow::anyhow!("No reviews found for user {}", gmaps_user.gmaps_id.as_str()));
        }
    }
    browser::wait_for_url_regex(&tab, &regex::Regex::new(r#"/place/[a-zA-Z0-9-_]+/@.*"#)?, 10000)?;
    browser::wait_dom_ready(&tab, 10000)?;

    tracing::debug!("Got review for {}", review_url);

    let review_text_element = tab.find_element_by_xpath(r#"//div[contains(@lang, "en")]/span"#)?;
    let review_text = review_text_element.get_inner_text()?;
    tracing::debug!("Retrieved review text element");

    let show_original_button = tab.find_element_by_xpath(r#"//button[contains(@role, "switch")]/span[contains(text(), "original")]"#);
    let original_review_text = match show_original_button {
        Ok(button) => {
            tracing::debug!("Found 'Show original' button, clicking to reveal original text");
            button.click()?;
            sleep(Duration::from_secs(1));
            let original_review_text_element = match tab.find_element_by_xpath(r#"//button[contains(@role, "switch")]/span[contains(text(), "translation")]/../../..//div[@lang]/span"#) {
                Ok(elem) => elem,
                Err(e) => return Err(anyhow::anyhow!("Failed to find original review text element: {}", e)),
            };

            let original_review_text = original_review_text_element.get_inner_text()?;
            Some(original_review_text)
        }
        Err(_) => None,
    };

    tracing::debug!("Retrieved review text: '{}'", review_text);

    let stars_span = tab.find_elements_by_xpath(r#"//span[contains(@aria-label, "stars")]/span[contains(@class, "google-symbols")]"#)?;
    if stars_span.is_empty() {
        return Err(anyhow::anyhow!("Failed to find star rating element"));
    }

    let valid_star_classes = match stars_span[0].get_attribute_value("class") {
        Ok(Some(class)) => class,
        _ => return Err(anyhow::anyhow!("Failed to get valid star class")),
    };

    let mut star_count = 0;
    for star in stars_span {
        let class_value = star.get_attribute_value("class")?;
        if let Some(class) = class_value {
            if class == valid_star_classes {
                star_count += 1;
            }
        }
    }

    tracing::debug!("Retrieved star rating: {}", star_count);

    let place_details_button = tab.find_element_by_xpath(r#"//div[contains(@jsaction, "placeNameHeader")]"#)?;
    place_details_button.click()?;
    browser::wait_for_url_regex(&tab, &regex::Regex::new(r#"maps/place/.+/@.*"#)?, 10000)?;
    browser::wait_dom_ready(&tab, 10000)?;
    let place_name = get_place_name_from_url(&tab.get_url()).unwrap_or_else(|| "Unknown Place".to_string());

    tracing::debug!("Retrieved place name: {}", place_name);

    Ok(NewReview {
        place_name,
        review_text,
        review_original_text: original_review_text,
        stars: star_count,
        user_id: gmaps_user.id,
    })
}

fn get_place_name_from_url(url: &str) -> Option<String> {
    let re = regex::Regex::new(r#"/place/([^/]+)/@"#).ok()?;
    let caps = re.captures(url)?;
    match caps.get(1).map(|m| m.as_str().to_string()) {
        Some(mut name) => {
            name = urlencoding::decode(&name).unwrap_or_else(|_| "Unknown Place".into()).to_string();
            name = name.replace('+', " ");
            Some(name)
        },
        None => Some("Unknown Place".to_string()),
    }
}