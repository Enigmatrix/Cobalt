use windows::core::Interface;
use windows::{
    Data::Xml::Dom::{XmlDocument, XmlElement},
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

use util::error::Result;

pub struct Progress {
    pub title: String,
    pub value: f64,
    pub value_string_override: String,
    pub status: String,
}

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

    pub fn show_basic(title: &str, message: &str) -> Result<()> {
        let aumid = "Cobalt";
        let mgr = ToastNotificationManager::CreateToastNotifierWithId(&aumid.into())?;

        let content = XmlDocument::new()?;
        content.LoadXml(
            &r#"
            <toast activationType="protocol">
                <visual>
                    <binding template="ToastGeneric">
                        <text id='1'></text>
                        <text id='2'></text>
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

        let toast = ToastNotification::CreateToastNotification(&content)?;
        mgr.Show(&toast)?;
        Ok(())
    }

    pub fn show_with_progress(title: &str, message: &str, progress: Progress) -> Result<()> {
        let aumid = "Cobalt";
        let mgr = ToastNotificationManager::CreateToastNotifierWithId(&aumid.into())?;

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
        progress_xml.SetAttribute(&"title".into(), &progress.title.into())?;
        progress_xml.SetAttribute(&"value".into(), &progress.value.to_string().into())?;
        progress_xml.SetAttribute(&"status".into(), &progress.status.into())?;
        progress_xml.SetAttribute(
            &"valueStringOverride".into(),
            &progress.value_string_override.into(),
        )?;

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
