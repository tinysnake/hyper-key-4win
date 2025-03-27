![hyper-key-logo](src/http_server_assets/favicon.ico)

This is a Windows application makes you convert a single key into a Hyper Key when hold.

### What is a Hyper Key?

A Hyper Key combination of ctrl + win + alt + shift, it's a super awkward key combination that almost nobody uses (except for MS Office), so if you have some shortcut keys that don't want to conflict with other application, you can use this app to turn this awkward key combination into a single key solution.

The default hyper key is the `Caps Lock` key.

## FAQ

### Can I change the hyper key to a different key?

Sure, click the app icon in the task bar and select `Preferences`, it will open up the configuration page, and you'll get more info there.

### Hyper Key still conflicts with some windows shortcut key.

This happens When you chose the `Override` mode in the configuration page, there're some solutions here:

1. Simply [turn off the office key](https://www.reddit.com/r/Office365/comments/pjhswo/how_do_i_disable_office_keyboard_shortcutshotkeys/), which doesn't quite get along with Hyper Key in this mode.

2. You can turn Hyper Key into Meh Key (which is: ctrl + alt + shift), by open up the configuration page and turn on the `Use Meh Key` toggle.

### How can I start Hyper Key on startup?

Create a shortcut-link of this application and put it into the startup folder.

You can navigate to startup folder by type `shell:startup` in the address bar of the File Explorer.

### AutoHotkey can do this, so why bother?

1. This is a less geek, maybe more elegant way.
2. A great excuse to try rust.




