use windows::{Data::Xml::Dom::XmlElement, UI::Notifications::{ToastNotification, ToastNotificationManager, ToastTemplateType}};

use util::error::Result;

pub struct Toast {

}

impl Toast {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&self, title: &str, message: &str) -> Result<()> {
        use windows::core::Interface;
        // let aumid = r#"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe"#;
        let aumid = r#"Cobalt"#;
        let mgr = ToastNotificationManager::CreateToastNotifierWithId(&aumid.into())?;
        let content = ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)?;
        content.SelectSingleNode(&"//text[@id = '1']".into())?.SetInnerText(&title.into())?;
        content.SelectSingleNode(&"//text[@id = '2']".into())?.SetInnerText(&message.into())?;
        
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

        let xml = content.GetXml()?;
        print!("{}", xml);


        let toast = ToastNotification::CreateToastNotification(&content)?;
        mgr.Show(&toast)?;
        Ok(())
    }
}

#[test]
fn feature() -> Result<()> {
    let toast = Toast::new();
    toast.show("League of Legends", "You've been playing too much! Go study!")?;
    Ok(())
}