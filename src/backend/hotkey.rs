use global_hotkey::GlobalHotKeyManager;

pub fn register_hotkey() -> anyhow::Result<GlobalHotKeyManager> {
    use global_hotkey::{
        hotkey::{Code, HotKey, Modifiers},
        GlobalHotKeyManager,
    };

    // initialize the hotkeys manager
    let manager = GlobalHotKeyManager::new().unwrap();

    // Ctrl + Left Arrow
    manager.register(HotKey::new(Some(Modifiers::CONTROL), Code::ArrowLeft))?;
    manager.register(HotKey::new(Some(Modifiers::CONTROL), Code::ArrowRight))?;
    manager.register(HotKey::new(Some(Modifiers::CONTROL), Code::Space))?;
    manager.register(HotKey::new(Some(Modifiers::SHIFT), Code::Space))?;

    Ok(manager)
}
