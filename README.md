# Cobalt

View usage statistics of all your apps. Track and determine where you are wasting time, and get your procrastination under control.

<p align="center">
  <img src="./assets/screenshots/01.home.png" alt="Cobalt home dashboard" width="720" />
</p>

<details>
  <summary><strong>Show more screenshots</strong></summary>

  <br />

  <p align="center">
    <img src="./assets/screenshots/01.home.light.png" alt="Home screen in light theme" width="720" />
    <br />
    Home Page (light mode)
  </p>

  <p align="center">
    <img src="./assets/screenshots/02.streaks.png" alt="App usage streaks view" width="720" />
    <br />
    Home Page (Streaks)
  </p>

  <p align="center">
    <img src="./assets/screenshots/03.apps.png" alt="Apps overview screen" width="720" />
    <br />
    Apps Page
  </p>

  <p align="center">
    <img src="./assets/screenshots/04.tag.png" alt="Tag details screen" width="720" />
    <br />
    Tag Page
  </p>

  <p align="center">
    <img src="./assets/screenshots/04.tag.histogram.png" alt="Tag histogram screen" width="720" />
    <br />
    Tag Page (Histogram)
  </p>

  <p align="center">
    <img src="./assets/screenshots/05.alerts.png" alt="Alerts list screen" width="720" />
    <br />
    Alerts Page
  </p>

  <p align="center">
    <img src="./assets/screenshots/06.alert.png" alt="Alert details screen" width="720" />
    <br />
    Alert Page
  </p>

  <p align="center">
    <img src="./assets/screenshots/07.edit_alert.usage.png" alt="Edit alert usage screen" width="720" />
    <br />
    Edit Alert Page (Usage Preview)
  </p>

  <p align="center">
    <img src="./assets/screenshots/07.edit_alert.trigger_action.png" alt="Edit alert trigger action screen" width="720" />
    <br />
    Edit Alert Page (Action Preview)
  </p>

  <p align="center">
    <img src="./assets/screenshots/07.edit_alert.reminders.png" alt="Edit alert reminders screen" width="720" />
    <br />
    Edit Alert Page (Reminder Preview)
  </p>

  <p align="center">
    <img src="./assets/screenshots/08.history.png" alt="Usage history screen" width="720" />
    <br />
    History
  </p>

</details>

> [!WARNING]
> This project is mostly production-ready, but sometimes Chrome might
> slow down by 2-5%. If this becomes noticible, please report it so that I can trace the issue!

## Docs
See the [ARCHITECTURE.md](./docs/ARCHITECTURE.md) for an overview of the project.

More detailed developer guides:
- General: [DEV_GUIDE.md](./docs/DEV_GUIDE.md)
- Engine: [DEV_GUIDE_Engine.md](./docs/DEV_GUIDE_Engine.md)
- UI: [DEV_GUIDE_UI.md](./docs/DEV_GUIDE_UI.md)

## Running

```bash
# Install UI dependencies
bun i
# Build all
cargo build

# Run the UI
bun dev
# Run the Engine
cargo run --bin engine

# Test
cargo test
```