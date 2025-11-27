use crate::crawler::browser;
use crate::models::NewUser;
use anyhow::Result;

static GMAPS_USER_URL: &str = "https://www.google.com/maps/contrib/{}/reviews?hl=en";

pub fn get_user_from_id(user_id: &str) -> Result<NewUser> {
    let browser = browser::get(true)?;
    let tab = browser::new_tab(&browser)?;

    let user_url = GMAPS_USER_URL.replace("{}", user_id);
    tab.navigate_to(&user_url)?;
    tab.wait_until_navigated()?;
    browser::wait_for_url(&tab, "/reviews/@")?;

    let name_element = tab.find_element(
        r"h1.fontHeadlineLarge[role='button'][tabindex='0'][aria-haspopup='true']",
    )?;
    let name = name_element.get_inner_text()?;
    Ok(NewUser {
        gmaps_id: user_id.to_string(),
        name,
    })
}
