"""Pointer input at coordinates: `page.mouse`."""

from __future__ import annotations

import time

import brimp


def centre(page, selector):
    return page.evaluate(
        "(() => { const box = document.querySelector(%r).getBoundingClientRect();"
        " return [Math.round(box.x + box.width / 2), Math.round(box.y + box.height / 2)]; })()"
        % selector
    )


def events(page):
    return page.evaluate("JSON.stringify(globalThis.log || [])")


def test_mouse_click_targets_the_element_under_the_point():
    with brimp.Session() as session:
        page = session.new_page()
        page.load_html(
            '<div id="a" style="margin:20px;width:200px;height:100px">target</div>'
            '<div id="b" style="margin:20px;width:200px;height:100px"></div>',
            "https://example.test/",
        )
        page.evaluate(
            """
            globalThis.log = [];
            for (const name of ['pointermove','pointerdown','pointerup','click'])
              document.addEventListener(name, (event) => log.push([
                event.type, event.target.id || event.target.localName,
                event.clientX, event.clientY, event.buttons, event.isTrusted,
              ]), true);
            """
        )
        x, y = centre(page, "#a")

        # Hit testing resolves the point to the element the layout places there.
        assert page.evaluate(
            "[document.elementFromPoint(%d, %d).id, document.elementsFromPoint(%d, %d).length]"
            % (x, y, x, y)
        ) == ["a", 3]
        assert page.evaluate("document.elementFromPoint(-5, -5)") is None

        page.mouse.move(x - 60, y - 30, steps=3).click(x, y)
        assert page.mouse.position == (float(x), float(y))
        assert page.evaluate("log.length") > 0
        # The press and the release both land on the element at the point.
        assert ["pointerdown", "a", x, y, 1, True] in page.evaluate("log")
        assert page.evaluate("log.at(-1)[0]") == "click"
        assert page.evaluate("log.at(-1)[1]") == "a"
        assert page.evaluate("log.filter((entry) => entry[0] === 'pointermove').length") >= 4
        assert page.evaluate("log.every((entry) => entry[5])") is True


def test_press_and_hold_reports_a_single_click():
    with brimp.Session() as session:
        page = session.new_page()
        page.load_html('<button id="hold">hold</button>', "https://example.test/")
        page.evaluate(
            """
            globalThis.log = [];
            for (const name of ['pointerdown','pointerup','click'])
              document.addEventListener(name, (event) => log.push([event.type, Date.now()]), true);
            """
        )
        x, y = centre(page, "#hold")
        page.mouse.move(x, y).down()
        pressed = time.monotonic()
        time.sleep(0.2)
        page.mouse.up()
        assert time.monotonic() - pressed >= 0.2
        assert page.evaluate("log.map((entry) => entry[0])") == [
            "pointerdown",
            "pointerup",
            "click",
        ]


def test_wheel_reports_deltas_at_the_point():
    with brimp.Session() as session:
        page = session.new_page()
        page.load_html('<div id="a" style="height:200px">a</div>', "https://example.test/")
        page.evaluate(
            "globalThis.deltas = [];"
            "addEventListener('wheel', (event) => deltas.push([event.deltaX, event.deltaY, event.isTrusted]), true)"
        )
        x, y = centre(page, "#a")
        page.mouse.wheel(0, 120, x=x, y=y)
        assert page.evaluate("JSON.stringify(deltas)") == "[[0,120,true]]"


def test_selector_click_reports_the_element_centre():
    with brimp.Session() as session:
        page = session.new_page()
        page.load_html(
            '<div id="a" style="margin:20px;width:100px;height:40px"></div>',
            "https://example.test/",
        )
        page.evaluate(
            "globalThis.first = null;"
            "addEventListener('pointermove', (event) => { first ??= [event.clientX, event.clientY]; }, true)"
        )
        x, y = centre(page, "#a")
        page.click("#a")
        # `page.evaluate` decodes the array; the first sample is the element centre
        # rather than the (0, 0) a coordinate-less dispatcher would report.
        assert page.evaluate("first") == [x, y]
