namespace Cobalt.Common.Data.Entities

open System;

type Id = int64

type AppIdentity =
    | Win32 of Path: string
    | UWP of AUMID: string

[<CLIMutable>]
type App = {
    mutable Id: Id;
    Name: string voption;
    Description: string voption;
    Color: string voption;
    Identity: AppIdentity;
}

and [<CLIMutable>] Tag = {
    mutable Id: Id;
    Name: string;
    Description: string;
    Color: string;
}

[<CLIMutable>]
type Session = {
    mutable Id: Id;
    Title: string;
    Arguments: string voption;
    AppId: Id;
}

[<CLIMutable>]
type Usage = {
    mutable Id: Id;
    Start: DateTime;
    End: DateTime;
    DuringIdle: bool;
    SessionId: Id;
}

type Target = App of AppId: Id | Tag of TagId: Id

type TimeFrame =
    | Once of Start: DateTime * End: DateTime
    | Repeated of Type: RepeatType * StartOfDay: TimeSpan * EndOfDay: TimeSpan
and RepeatType = Daiy = 0 | Weeky = 1 | Monthy = 2

type Reaction =
    | Kill
    | Message of Message: string

[<CLIMutable>]
type Alert = {
    mutable Id: Id;
    Target: Target;
    TimeFrame: TimeFrame;
    UsageLimit: TimeSpan;
    ExceededReaction: Reaction
}
