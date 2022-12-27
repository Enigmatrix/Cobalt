pub type Timestamp = u64;

#[derive(Default, Debug)]
pub struct App {
    id: Ref<Self>,
    name: String,
    description: String,
    company: String,
    color: Option<String>,
    identity: AppIdentity,
}

#[derive(Default, Debug)]
pub struct Session {
    id: Ref<Self>,
    app: Ref<App>,
    title: String,
    cmd_line: Option<String>,
}

#[derive(Default, Debug)]
pub struct Usage {
    id: Ref<Self>,
    session: Ref<Session>,
    start: Timestamp,
    end: Timestamp,
}

#[derive(Default, Debug)]
pub struct InteractionPeriod {
    id: Ref<Self>,
    start: Timestamp,
    end: Timestamp,
    mouseclicks: u64,
    keystrokes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Unique identity of an App, outside of the Database (on the FileSystem/Registry)
pub enum AppIdentity {
    Win32 { path: String },
    UWP { aumid: String },
}

impl Default for AppIdentity {
    fn default() -> Self {
        Self::Win32 {
            path: Default::default(),
        }
    }
}

pub trait Table {
    type Id: Default;

    fn id(&self) -> &Ref<Self>
    where
        Self: Sized;
}

#[derive(Default, Debug, Clone)]
pub struct Ref<T: Table> {
    inner: T::Id,
}

impl<T: Table> Ref<T> {
    fn new(inner: T::Id) -> Self {
        Self { inner }
    }
}

macro_rules! table {
    ($t:ty, $fld:ident : $id:ty) => {
        impl Table for $t {
            type Id = $id;
            fn id(&self) -> &Ref<Self> {
                &self.$fld
            }
        }
    };
}

table!(App, id: u64);
table!(Session, id: u64);
table!(Usage, id: u64);
table!(InteractionPeriod, id: u64);
