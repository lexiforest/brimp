from pathlib import Path
import sys
import brimp


def setup():
    origins = {}

    def partition(context):
        if context.origin is None:
            raise ValueError("Preferences require a non-opaque origin")
        return origins.setdefault(context.origin, {})

    def put(context, key, value):
        partition(context)[key] = value

    return {
        "handlers": {"get": lambda context, key: partition(context).get(key), "set": put},
        "dispose": origins.clear,
    }


plugin = brimp.Plugin.from_file(
    Path(__file__).with_name("preferences.js"), name="preferences", setup=setup
)
url = sys.argv[1] if len(sys.argv) > 1 else "https://example.com"
with brimp.Session(plugins=[plugin]) as session:
    page = session.get(url)
    page.evaluate('preferences.set("theme", "dark")')
    page.get(url)
    print(page.evaluate('preferences.get("theme")'))
