#[derive(Clone, Debug, PartialEq, Eq)]
/// Unique identity of an App, outside of the Database (on the FileSystem/Registry)
pub enum AppIdentity {
    Win32 { path: String },
    UWP { aumid: String },
}

pub struct App {
    id: Ref<Self>,
    name: String,
    description: String,
    company: String
}

pub struct Session {
    id: Ref<Self>,
    app: Ref<App>
}

pub struct Usage {
    id: Ref<Self>,
    session: Ref<Session>
}

pub trait Table {
    type Id: Sized;

    fn id(&self) -> &Ref<Self> where Self: Sized;
}

#[derive(Default)]
pub struct Ref<T: Table> {
    inner: Option<T::Id>
}

impl<T: Table> Ref<T> {
    fn new(id: T::Id) -> Self {
        Self { inner: Some(id) }
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