mod encrypt;
mod secrets;
pub use encrypt::*;
pub use secrets::*;

use crate::{TauriState, Result};

#[tauri::command]
pub fn is_authenticated(state: TauriState) -> Result<bool> {
    println!("Checking if authenticated");
    let state = state.lock()?;
    if state.is_authenticated() {
        println!("====> true");
        Ok(true)
    } else {
        println!("====> false");
        Ok(false)
    }
}
#[tauri::command]
pub fn log_out(state: TauriState) -> Result<()> {
    let mut state = state.lock()?;
    state.log_out();
    Ok(())
}