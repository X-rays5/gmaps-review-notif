use crate::crawler::browser;
use crate::models::NewUser;
use anyhow::Result;

static GMAPS_USER_URL: &str = "https://www.google.com/maps/contrib/{}/reviews?hl=en";

pub fn get_user_from_id(user_id: &str) -> Result<NewUser> {
    let browser = browser::get(true)?;
    let tab = browser::new_tab(&browser)?;

    open_user_page(&tab, user_id)?;

    let name_element = tab
        .find_element(r"h1.fontHeadlineLarge[role='button'][tabindex='0'][aria-haspopup='true']")?;
    let name = name_element.get_inner_text()?;
    Ok(NewUser {
        gmaps_id: user_id.to_string(),
        name,
    })
}

fn open_user_page(tab: &headless_chrome::Tab, user_id: &str) -> Result<()> {
    let user_url = GMAPS_USER_URL.replace("{}", user_id);
    match tab.navigate_to(&user_url) {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to navigate to user page {user_url}: {e}"
            ));
        }
    }

    match tab.wait_until_navigated() {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to wait for navigation to complete for user page {user_url}: {e}"
            ));
        }
    }

    match browser::wait_for_url(tab, "/reviews/@", 15000) {
        Ok(()) => (),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "User page did not load correctly for {user_url}: {e}"
            ));
        }
    }
    Ok(())
}
