use anyhow::Result;
use crate::crawler::browser;

static GMAPS_USER_URL: &str = "https://www.google.com/maps/contrib/{}/reviews?hl=en";

pub struct GmapsUser {
    pub id: String,
    pub name: String,
}

pub fn get_user_from_id(user_id: &str) -> Result<GmapsUser> {
    let browser = browser::get(true)?;
    let tab = browser::new_tab(&browser)?;

    let user_url = GMAPS_USER_URL.replace("{}", user_id);
    tab.navigate_to(&user_url)?;
    tab.wait_until_navigated()?;
    browser::wait_for_url(&tab, "/reviews/@", 15000)?;

    let name_element = tab.find_element(r#"h1.fontHeadlineLarge[role='button'][tabindex='0'][aria-haspopup='true']"#)?;
    let user_name = name_element.get_inner_text();
    match user_name {
        Ok(name) => Ok(GmapsUser {
            id: user_id.to_string(),
            name,
        }),
        Err(e) => Err(anyhow::anyhow!("Failed to get user name: {}", e)),
    }
}