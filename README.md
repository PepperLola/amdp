# Apple Music Discord Presence
> [!IMPORTANT]
> MacOS Only!

## Setup
1. Clone the repository
2. Create a [Discord app](https://discord.com/developers/applications/) named `Apple Music`, copy the client ID into `src/main.rs` and paste it into the `CLIENT_ID` constant
3. Install the dependencies, then run `cargo build --path .` in the project directory
4. Create a new file in `~/Library/LaunchAgents/` called `com.github.pepperlola.amdp.plist` and add the following
    - Be sure to replace `PATH` with the path to the executable that was installed, most likely `/Users/<username>/.cargo/bin/amdp`.
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple/DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.github.pepperlola.amdp</string>
    <key>ProgramArguments</key>
    <array>
        <string>PATH</string> <!-- REPLACE PATH HERE -->
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```
5. Run `launchctl load ~/Library/LaunchAgents/com.github.pepperlola.amdp.plist`
