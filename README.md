# PlistOxide ![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/NootInc/PlistOxide/main.yml?branch=master&logo=github&style=for-the-badge)

<p align="center"><img width="256" src="src/app_icon/icon512x512@2x.png"></p>

Cross-platform Property List (plist) editor written in Rust.

Currently does not support manual arrangement of entries, only sorting using the right click context menu option. Also, due to a technical limitation `⌘+Q` on macOS will close the application even if there are unsaved changes.

Support for flushing (snapshotting) OpenCore EFI folder data to config.plist coming soon.

The PlistOxide project is licensed under the `Thou Shalt Not Profit License version 1.5`. See [`LICENSE`](https://github.com/ChefKissInc/PlistOxide/blob/master/LICENSE).
