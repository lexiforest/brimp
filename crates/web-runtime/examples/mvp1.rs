use web_runtime::{Browser, PageOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::new()?;
    let mut page = browser.new_page(PageOptions::builder().viewport(1280, 720).build())?;

    page.set_content(
        r##"
        <html>
        <head>
        <style>
        #box { width: 200px; padding: 20px; background: #eee; }
        </style>
        </head>
        <body><div id="box">Hello</div></body>
        </html>
        "##,
    )?;

    page.eval(
        r##"
        const box = document.querySelector("#box");
        box.setAttribute("data-test", "yes");
        box.style.width = "300px";
        console.log(box.getBoundingClientRect().width);
        "##,
    )?;

    page.screenshot("out.png")?;
    Ok(())
}
