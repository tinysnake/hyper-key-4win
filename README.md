![hyper-key-logo](src/http_server_assets/favicon.ico)

This is a Windows application allows you to convert a single key into a Hyper Key when held.

### What is a Hyper Key?

A Hyper Key is a combination of ctrl + win + alt + shift, it's a super awkward key combination that almost nobody uses (except for MS Office), if you have some shortcut keys that don't want to conflict with other applications, you can use this app to turn this awkward key combination into a single key solution.

The default hyper key is the `Caps Lock` key. This app will start silently and create a tray icon on the task bar.


## FAQ

### Windows Defender or my Antivirus software said that the Hyper Key is a malware?!

No, it's not, They might have detected that this application is monitoring your keyboard activity.
1. Monitoring keyboard activity is necessary to perform the Hyper Key action.
2. It uses the Windows SDK for monitoring ,not some hacky methods.
3. You can read the code and compile it by yourself to prevent risks.

### Can I change the hyper key to a different key?

Sure, click the app icon in the task bar and select `Preferences`, it will open up the configuration page where and you'll find more info there.

### Hyper Key still conflicts with some windows shortcut key.

This happens when you using the `Override` mode in the configuration page, there're some solutions:

1. Simply [turn off the office key](https://www.reddit.com/r/Office365/comments/pjhswo/how_do_i_disable_office_keyboard_shortcutshotkeys/) as it doesn't quite get along with Hyper Key in this mode.

2. Switch to Meh Key (which is: ctrl + alt + shift), by turn on the `Use Meh Key` toggle in the configuration page.

### Hyper Key won't start, there's no tray icon, can't find it in the Task Manager.

Hyper Key will check the port 19456(by default) is in use, if it's in use then the app will sliently quit. You can download [this application](https://learn.microsoft.com/en-us/sysinternals/downloads/tcpview) to check if this port is taken or not.

If the default port 19456 is already taken, you can change it by open/create the following file: `%USERPROFILE%\.config\hyper-key\config.json`, you can enter the `%USERPROFILE%` in the File explorer's address bar, and then create the following file.

Change the port number if the config file is present, or paste in the following into the config file you just created(change the number to what ever you want between 10000-60000):

```json
{
    "port": 34567
}
```

### Why Hyper Key require port listening?
The port hosts a configuration web server for communication between the configuration page and the app, this application doesn't monitor your activity, and doesn't upload data to the internet.

### How can I start Hyper Key on startup?

Create a shortcut of this application and put it into the startup folder.

You can navigate to startup folder by type `shell:startup` in the address bar of the File Explorer.

### AutoHotkey can do this, so why bother?

1. This is a less geek, maybe more elegant way.
2. A great excuse to try rust.


## How to compile?

1. Make sure you're in Windows system.
2. Clone this repository to a folder you want.
3. open up your favourite terminal app and cd into the folder of the repository.
4. run: `cargo build --release` if you have rust installed; if not, check [this](https://www.rust-lang.org/tools/install) out first.

