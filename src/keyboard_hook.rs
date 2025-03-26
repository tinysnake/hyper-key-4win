use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

use log::{debug, warn};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
            KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU,
            VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
        },
        WindowsAndMessaging::{
            CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx,
            WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
        },
    },
};

use crate::keyboard_conf::{self, KeyboardHookConf};

static KEYBOARD_HOOK: OnceLock<Arc<KeyboardHook>> = OnceLock::new();

const HYPER_MODE_NONE: u8 = 0;
const HYPER_MODE_INTERUPT: u8 = 1;
const HYPER_MODE_BASIC: u8 = 2;
const HYPER_MODE_HOTKEY: u8 = 3;
const HYPER_MODE_CANCEL: u8 = 4;
const HYPER_MODE_RESET: u8 = 5;

#[derive(Debug)]
pub struct KeyboardHook {
    hook: Arc<Mutex<HHOOK>>,
    data: Arc<Mutex<KeyboardHookData>>,
}

#[derive(Debug)]
struct KeyboardHookData {
    is_hyper_key_down: bool,
    hot_key_sent: bool,
    cancelling: bool,
}

unsafe impl Send for KeyboardHook {}
unsafe impl Sync for KeyboardHook {}

impl KeyboardHook {
    pub fn new() -> Arc<KeyboardHook> {
        let kh = Arc::new(Self {
            hook: Arc::new(Mutex::new(HHOOK::default())),
            data: Arc::new(Mutex::new(KeyboardHookData {
                is_hyper_key_down: false,
                hot_key_sent: false,
                cancelling: false,
            })),
        });

        KEYBOARD_HOOK.set(kh.clone()).unwrap();
        kh
    }

    fn is_extended_key(vk: VIRTUAL_KEY) -> bool {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_HOME, VK_INSERT, VK_LEFT, VK_NEXT,
            VK_NUMLOCK, VK_PRIOR, VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_SNAPSHOT, VK_UP,
        };

        match vk {
            // Navigation keys should be injected with the extended flag to distinguish
            // them from the Numpad navigation keys. Otherwise, input Shift+<Navigation key>
            // may not have the expected result and depends on whether NUMLOCK is enabled/disabled.
            // A list of the extended keys can be found here:
            // https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#extended-key-flag
            // TODO: The keys "BREAK (CTRL+PAUSE) key" and "ENTER key in the numeric keypad" are
            // missing
            VK_RMENU | VK_RCONTROL | VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT | VK_INSERT
            | VK_DELETE | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_NUMLOCK | VK_SNAPSHOT
            | VK_DIVIDE => true,
            _ => false,
        }
    }

    fn is_modifier_key(vk: VIRTUAL_KEY) -> bool {
        match vk {
            VK_LCONTROL | VK_RCONTROL | VK_CONTROL | VK_SHIFT | VK_LSHIFT | VK_RSHIFT | VK_MENU
            | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN => true,
            _ => false,
        }
    }

    pub fn start(&self) {
        *self.hook.lock().unwrap() =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0) }
                .unwrap();
    }

    pub fn stop(&self) {
        unsafe { UnhookWindowsHookEx(*self.hook.lock().unwrap()) }.unwrap();
    }

    pub fn handle_key_event(&self, key_action: u32, input: KBDLLHOOKSTRUCT) -> bool {
        let conf_result = keyboard_conf::CONF.lock();
        if conf_result.is_err() {
            return false;
        }
        let conf = { conf_result.unwrap().clone() };
        let key = VIRTUAL_KEY(input.vkCode as u16);
        let is_key_down = key_action == WM_KEYDOWN || key_action == WM_SYSKEYDOWN;
        let hyper_mode = {
            let mut data_guard = self.data.lock().unwrap();
            let data: &mut KeyboardHookData = data_guard.deref_mut();
            self.process_hyper_key(key, is_key_down, data, &conf)
        };
        match hyper_mode {
            HYPER_MODE_BASIC => {
                debug!(
                    "send_hyper_key: {}",
                    if is_key_down { "DOWN" } else { "UP" }
                );
                self.send_hyper_key(conf.use_meh_key, is_key_down);
            }
            HYPER_MODE_HOTKEY => {
                debug!(
                    "send_hyper_hotkey: {}",
                    if is_key_down { "DOWN" } else { "UP" }
                );
                self.send_hyper_key(conf.use_meh_key, is_key_down);
                // the actual hotkey stroke is current one, no need to send again.
            }
            HYPER_MODE_CANCEL => {
                debug!("cancel_hyper_key");
                self.send_input(&[
                    self.get_key_input(VIRTUAL_KEY(conf.the_key), true),
                    // key up stroke is current one, no need to send again.
                ]);
            }
            HYPER_MODE_RESET => {
                warn!("reset states");
                self.send_hyper_key(conf.use_meh_key, false);
            }
            HYPER_MODE_INTERUPT => {
                return true;
            }
            _ => {}
        }

        debug!(
            "handle_key_event: key_action: {}, key_code: {:}",
            key_action, input.vkCode
        );
        false
    }

    fn process_hyper_key(
        &self,
        key: VIRTUAL_KEY,
        is_key_down: bool,
        data: &mut KeyboardHookData,
        conf: &KeyboardHookConf,
    ) -> u8 {
        if key == VIRTUAL_KEY(conf.the_key) {
            if conf.hyper_mode == keyboard_conf::HyperMode::Override {
                return HYPER_MODE_BASIC;
            } else {
                if data.cancelling {
                    if is_key_down {
                        data.is_hyper_key_down = false;
                        data.cancelling = false;
                    }
                    return HYPER_MODE_NONE;
                } else if !data.is_hyper_key_down {
                    if is_key_down {
                        data.is_hyper_key_down = true;
                        debug!("hyper_key_down:");
                    }
                } else if !is_key_down {
                    data.is_hyper_key_down = false;
                    if data.hot_key_sent {
                        data.hot_key_sent = false;
                        debug!("set variable: hot_key_sent false");
                    } else {
                        data.cancelling = true;
                        return HYPER_MODE_CANCEL;
                    }
                }
                return HYPER_MODE_INTERUPT;
            }
        } else if key == VK_ESCAPE {
            if data.cancelling || data.is_hyper_key_down || data.hot_key_sent {
                data.cancelling = false;
                data.hot_key_sent = false;
                data.is_hyper_key_down = false;
                return HYPER_MODE_RESET;
            }
        }
        else if !Self::is_modifier_key(key) {
            if data.is_hyper_key_down {
                if is_key_down {
                    data.hot_key_sent = true;
                    debug!("set variable: hot_key_sent true");
                }
                return HYPER_MODE_HOTKEY;
            }
        }
        return HYPER_MODE_NONE;
    }

    fn send_input(&self, inputs: &[INPUT]) {
        //let len = inputs.len() as u32;
        let size = size_of::<INPUT>() as i32;
        let _ = unsafe { SendInput(inputs, size) };
    }

    fn send_hyper_key(&self, use_meh_key: bool, key_down: bool) {
        if use_meh_key {
            self.send_input(&[
                self.get_key_input(VK_LCONTROL, key_down),
                self.get_key_input(VK_LMENU, key_down),
                self.get_key_input(VK_LSHIFT, key_down),
            ]);
        } else {
            self.send_input(&[
                self.get_key_input(VK_LCONTROL, key_down),
                self.get_key_input(VK_LWIN, key_down),
                self.get_key_input(VK_LMENU, key_down),
                self.get_key_input(VK_LSHIFT, key_down),
            ]);
        }
    }

    fn get_key_input(&self, vk: VIRTUAL_KEY, key_down: bool) -> INPUT {
        let flag = if Self::is_extended_key(vk) {
            KEYEVENTF_EXTENDEDKEY
        } else {
            KEYBD_EVENT_FLAGS(0)
        };
        let flag = flag
            | if key_down {
                KEYBD_EVENT_FLAGS(0)
            } else {
                KEYEVENTF_KEYUP
            };
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code != 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let input = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

    if let Some(kh) = KEYBOARD_HOOK.get() {
        let block = kh.handle_key_event(wparam.0 as u32, input);
        if block {
            return LRESULT(1);
        }
    };
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
