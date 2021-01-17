namespace Cobalt.Common.Data.Entities

open System;

type Id = int64

type AppIdentity =
    | Win32 of Path: string
    | UWP of AUMID: string

[<CLIMutable>]
type App = {
    Id: Id;
    Name: string voption;
    Description: string voption;
    Color: string voption;
    Identity: AppIdentity;
}

and [<CLIMutable>] Tag = {
    Id: Id;
    Name: string;
    Description: string;
    Color: string;
}

[<CLIMutable>]
type Session = {
    Id: Id;
    Title: string;
    Arguments: string voption;
    AppId: Id;
}

[<CLIMutable>]
type Usage = {
    Id: Id;
    Start: DateTime;
    End: DateTime;
    DuringIdle: bool;
    SessionId: Id;
}

type Target = App of App: App | Tag of Tag: Tag

type TimeFrame =
    | Once of Start: DateTime * End: DateTime
    | Repeated of Type: RepeatType * StartOfDay: TimeSpan * EndOfDay: TimeSpan
and RepeatType = Daily | Weekly | Monthly

type Reaction =
    | Kill
    | Message of Message: string

[<CLIMutable>]
type Alert = {
    Id: Id;
    Target: Target;
    TimeFrame: TimeFrame;
    UsageLimit: TimeSpan;
    ExceededReaction: Reaction
}
