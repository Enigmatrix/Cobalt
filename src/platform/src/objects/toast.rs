use util::error::Result;
use windows::Data::Xml::Dom::{XmlDocument, XmlElement};
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
use windows::core::Interface;

// Once we get registration settled, we can move away from a singleton pattern.

/// [Progress](https://learn.microsoft.com/en-us/windows/apps/design/shell/tiles-and-notifications/adaptive-interactive-toasts?tabs=xml#progress-bar)
/// for Toast notifications.
pub struct Progress {
    /// Title
    pub title: Option<String>,
    /// Progress value
    pub value: f64,
    /// Progress value text
    pub value_string_override: Option<String>,
    /// Status
    pub status: String,
}

const AUMID: &str = "me.enigmatrix.cobalt";

/// Toast notification manager using the WinRT APIs.
pub struct ToastManager;

impl ToastManager {
    // for icon
    /*let toastVisualElements = content.GetElementsByTagName(&"binding".into())?;
    toastVisualElements.GetAt(0)?.cast::<XmlElement>()?.SetAttribute(&"template".into(), &"ToastGeneric".into())?;

    // to add an icon you need app logo overlay
    let appLogoOverlay = content.CreateElement(&"image".into())?;
    // here you set the source of the overlay
    appLogoOverlay.SetAttribute(&"src".into(), &r#"C:\Users\enigm\OneDrive\Pictures\94f4045d-76a9-4691-84ad-a40f56a98de1.jpg"#.into())?;
    // here you can add style attributes
    appLogoOverlay.SetAttribute(&"hint-crop".into(), &"circle".into())?;
    appLogoOverlay.SetAttribute(&"placement".into(), &"appLogoOverride".into())?;
    toastVisualElements.GetAt(0)?.AppendChild(&appLogoOverlay)?;*/

    /// Show a basic toast notification with title and text with an alarm scenario.
    pub fn show_alert(title: &str, message: &str) -> Result<()> {
        let mgr = ToastNotificationManager::CreateToastNotifierWithId(&AUMID.into())?;

        let content = XmlDocument::new()?;
        content.LoadXml(
            &r#"
            <toast activationType="protocol" scenario="alarm">
                <visual>
                    <binding template="ToastGeneric">
                        <text id='1'></text>
                        <text id='2'></text>
                    </binding>
                </visual>
                <actions>
                    <action content='Dismiss' arguments='action=dismiss'/>
                </actions>
            </toast>"#
                .into(),
        )?;

        content
            .SelectSingleNode(&"//text[@id='1']".into())?
            .SetInnerText(&title.into())?;
        content
            .SelectSingleNode(&"//text[@id='2']".into())?
            .SetInnerText(&message.into())?;

        let toast = ToastNotification::CreateToastNotification(&content)?;
        mgr.Show(&toast)?;
        Ok(())
    }

    /// Show a basic toast notification with title, text and a progress bar.
    pub fn show_reminder(title: &str, message: &str, progress: Progress) -> Result<()> {
        let mgr = ToastNotificationManager::CreateToastNotifierWithId(&AUMID.into())?;

        let content = XmlDocument::new()?;
        content.LoadXml(
            &r#"
            <toast activationType="protocol">
                <visual>
                    <binding template="ToastGeneric">
                        <text id='1'></text>
                        <text id='2'></text>
                        <progress id='3'></progress>
                    </binding>
                </visual>
            </toast>"#
                .into(),
        )?;

        content
            .SelectSingleNode(&"//text[@id='1']".into())?
            .SetInnerText(&title.into())?;
        content
            .SelectSingleNode(&"//text[@id='2']".into())?
            .SetInnerText(&message.into())?;
        let progress_xml = content
            .SelectSingleNode(&"//progress[@id='3']".into())?
            .cast::<XmlElement>()?;
        if let Some(title) = progress.title {
            progress_xml.SetAttribute(&"title".into(), &title.into())?;
        }
        progress_xml.SetAttribute(&"value".into(), &progress.value.to_string().into())?;
        progress_xml.SetAttribute(&"status".into(), &progress.status.into())?;
        if let Some(value_string_override) = progress.value_string_override {
            progress_xml
                .SetAttribute(&"valueStringOverride".into(), &value_string_override.into())?;
        }

        let toast = ToastNotification::CreateToastNotification(&content)?;
        mgr.Show(&toast)?;
        Ok(())
    }
}

// #[test]
// fn feature() -> Result<()> {
//     // ToastManager::show_basic(
//     //     "League of Legends",
//     //     "You've been playing too much! Go study!",
//     // )?;

//     ToastManager::show_with_progress(
//         "League of Legends",
//         "You've been playing too much! Go study!",
//         Progress {
//             title: String::new(),
//             value: 0.6,
//             value_string_override: "".to_string(),
//             //status: "1h/1h 40m".to_string(),
//             status: "".to_string(),
//         },
//     )?;
//     Ok(())
// }
