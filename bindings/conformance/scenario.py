import asyncio
import json
import sys
import brimp

async def main(url):
    browser = await brimp.launch()
    page = await browser.new_page()
    await page.goto(url)
    result = {
        "title": await page.title(),
        "text": "Hello bindings" in await page.text_content(),
        "value": await page.evaluate("({answer: 42, values: [true, null]})"),
        "png": (await page.screenshot()).startswith(b"\x89PNG\r\n\x1a\n"),
    }
    for name, expression in {"javascript": "throw new Error('boom')", "unsupported": "undefined"}.items():
        try: await page.evaluate(expression)
        except brimp.BrimpError as error: result[name] = error.code
    hanging = await browser.new_page()
    task = asyncio.create_task(hanging.goto(url + "/hang"))
    await asyncio.sleep(0.05); task.cancel()
    try: await task
    except asyncio.CancelledError as error: result["cancelled"] = getattr(error, "code", None) == "cancelled"
    await hanging.close(); await hanging.close()
    await page.close(); await page.close()
    try: await page.title()
    except brimp.BrimpError as error: result["closed"] = error.code
    await browser.close(); await browser.close()
    print(json.dumps(result, sort_keys=True))

asyncio.run(main(sys.argv[1]))
