# Architecture

High-level architecture design document to show how the components interact with each other. For more
information, see the [Engine Dev Guide](./DEV_GUIDE_Engine.md) and the [Viewer Dev Guide](./DEV_GUIDE_Viewer.md).

## Component Diagram

```mermaid
graph LR

    subgraph Cobalt.Engine
        subgraph Engine
            Sentry
            Resolver
            Cache
        end
        Engine --> Platform
        Engine --> EngineData[Data]
    end

    subgraph Cobalt
        UI <--> Tauri --> ViewerData[Data]
    end

    Db[(Database)]
    Win32

    EngineData <--> Db
    ViewerData <--> Db
    Platform --> Win32
```

The Engine talks to the Win32 platform to fetch usage events, and saves them to the Db.
The Viewer talks to the Db to fetch information and displays them.


## ER Diagram
```mermaid
erDiagram
    apps {
        int             id PK
        tinyint         initialized "true if details are set"
        tinyint         found   "temp col, true if found by upsert"
        text            name
        text            description
        text            company
        text            color
        int_nullable    tag_id FK
        int             identity_is_win32     UK "UNIQUE(is_win32, path_or_aumid)"
        text            identity_path_or_aumid   UK "UNIQUE(is_win32, path_or_aumid)"
        blob_nullable   icon
    }

    tags {
        int             id PK
        text            name UK
        text            color
    }

    sessions {
        int             id PK
        int             app_id FK
        text            title
    }

    usages {
        int             id PK
        int             session_id FK
        int             start
        int             end
    }

    interaction_periods {
        int             id PK
        int             start
        int             end
        int             mouse_clicks
        int             key_strokes
    }

    system_events {
        int             id PK
        int             timestamp
        int             event
    }

    alerts {
        int             id PK
        int_nullable    app_id FK
        int_nullable    tag_id FK
        int             usage_limit
        int             time_frame      "Daily / Weekly / Monthly"
        int_nullable    trigger_action_dim_duration     "Duration of Dim(Duration)"
        text_nullable   trigger_action_message_content     "Content of Message(Content)"
        int             trigger_action_tag      "Kill / Dim / Message"
    }

    alert_events {
        int             id PK
        int             alert_id FK
        int             timestamp
    }

    reminders {
        int             id PK
        int             alert_id FK
        real            threshold
        text            message
        tinyint         active
    }

    reminder_events {
        int             id PK
        int             reminder_id FK
        int             timestamp
    }

    apps ||--o{ sessions : sessions
    sessions ||--o{ usages : usages
    tags ||--o{ apps : "tag's apps"
    apps ||--o{ alerts : "app alerts"
    tags ||--o{ alerts : "tag alerts"
    alerts ||--o{ reminders : reminders
    alerts ||--o{ alert_events : events
    reminders ||--o{ reminder_events : events
```