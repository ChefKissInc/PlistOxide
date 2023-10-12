# PlistOxide ![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/NootInc/PlistOxide/main.yml?branch=master&logo=github&style=for-the-badge)

<p align="center"><img width="256" src="src/app_icon/icon512x512@2x.png"></p>

Cross-platform Property List (plist) editor written in Rust.

Currently does not support manually rearranging the order of entries, only sorting with the option in the right click menu.

On macOS, there are no menu bar options yet. Use `⌘+O` to open, `⌘+S` to save. In addition, due to a technical limitation `⌘+Q` will not be prevented from closing the application when there are unsaved changes.

OpenCore configuration snapshot support coming soon, after the addition of menu bar options on macOS.

The Source Code of this Original Work is licensed under the `Thou Shalt Not Profit License version 1.5`. See [`LICENSE`](LICENSE).
