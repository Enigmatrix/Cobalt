namespace Cobalt.Common.Data.Entities

open System

type Id = int64

type IEntity =
    abstract Id: Id

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
with interface IEntity with
        member x.Id = x.Id

[<CLIMutable>]
type Tag = {
    mutable Id: Id;
    Name: string;
    Description: string;
    Color: string;
}
with interface IEntity with
        member x.Id = x.Id

[<CLIMutable>]
type Session = {
    mutable Id: Id;
    Title: string;
    Arguments: string voption;
    AppId: Id;
}
with interface IEntity with
        member x.Id = x.Id

[<CLIMutable>]
type Usage = {
    mutable Id: Id;
    Start: DateTime;
    End: DateTime;
    DuringIdle: bool;
    SessionId: Id;
}
with interface IEntity with
        member x.Id = x.Id

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
with interface IEntity with
        member x.Id = x.Id
