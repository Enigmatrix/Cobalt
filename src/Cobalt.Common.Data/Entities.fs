namespace Cobalt.Common.Data.Entites

open System
open System.IO

type AppIdentity =
    | Win32 of Path: string
    | UWP of AUMID: string
    // | Java of MainJar: string

type Ref<'a> =
    | Loaded of Value: 'a
    | Unloaded of Id: int64

[<CLIMutable>]
type App = {
    Id: int64;
    Name: string voption;
    Description: string voption;
    Color: string voption;
    Identity: AppIdentity;
    Icon: Ref<Stream>
}
and [<CLIMutable>] Tag = {
    Id: int64;
    Name: string;
    Description: string;
    Color: string;
}

[<CLIMutable>]
type Session = {
    Id: int64;
    Title: string;
    Arguments: string voption;
    App: Ref<App>;
}

[<CLIMutable>]
type Usage = {
    Id: int64;
    Start: DateTime;
    End: DateTime;
    DuringIdle: bool;
    Session: Ref<Session>;
}

(*type SystemEventKind = Logon = 0L | Logoff = 1L | Active = 2L | Idle = 3L

[<CLIMutable>]
type SystemEvent = {
    Id: int64;
    Timestamp: DateTime;
    Kind: SystemEventKind;
}

type Target = App of App: App | Tag of App: Tag

type TimeRange =
    | Once of Start: DateTime * End: DateTime
    | Repeated of Type: RepeatType * StartOfDay: TimeSpan * EndOfDay: TimeSpan
and RepeatType = Daily = 0L | Weekly = 1L | Monthly = 2L

type Reaction = 
    | Kill
    | Message of Message: string

[<CLIMutable>]
type Alert = {
    Id: int64;
    Target: Target;
    TimeRange: TimeRange;
    UsageLimit: TimeSpan;
    ExceededReaction: Reaction;
}
*)
