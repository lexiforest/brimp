use jsc::JsRuntime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = JsRuntime::new()?;
    let result = runtime.eval("1 + 2")?;
    println!("1 + 2 = {}", result.to_number()?);
    Ok(())
}
