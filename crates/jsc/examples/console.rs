use jsc::JsRuntime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = JsRuntime::new()?;
    runtime.set_console_callback(|message| println!("JavaScript said: {message}"))?;
    runtime.eval("console.log('hello')")?;
    Ok(())
}
