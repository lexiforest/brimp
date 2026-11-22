// Identical GLib embedding workload for GTK and WPE; no Inspector retention.
#ifdef BENCH_WPE
#include <wpe/webkit.h>
#include <wpe/headless/wpe-headless.h>
#else
#include <webkit/webkit.h>
#endif
#include <glib-unix.h>
#include <dlfcn.h>
#include <unistd.h>
#include <nlohmann/json.hpp>
#include <cstdio>
#include <memory>
#include <string>
#include <vector>

using Json = nlohmann::json;
static GMainLoop* loop;
struct Page {
    unsigned index;
    std::string requested;
    WebKitWebView* view;
    unsigned status = 0;
    bool finished = false;
    std::string error;
};
static std::vector<std::unique_ptr<Page>> pages;
static void emit(Json value) { printf("%s\n", value.dump().c_str()); fflush(stdout); }

static void snapshot(Page& page)
{
    static const char* script = R"JS(
        await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        const bounds = document.documentElement.getBoundingClientRect();
        return {
            url: location.href, title: document.title, readyState: document.readyState,
            visibility: document.visibilityState, focused: document.hasFocus(),
            viewport: [innerWidth, innerHeight, devicePixelRatio],
            screen: [screen.width, screen.height, screen.colorDepth],
            bounds: [bounds.width, bounds.height],
            nodes: document.getElementsByTagName('*').length,
            images: document.images.length,
            decodedImages: Array.from(document.images).filter(i => i.complete && i.naturalWidth).length,
            resources: performance.getEntriesByType('resource').length,
            text: (document.body?.innerText || '').slice(0, 2000),
            userAgent: navigator.userAgent,
            webglAPI: typeof WebGLRenderingContext !== 'undefined'
        };
    )JS";
    webkit_web_view_call_async_javascript_function(page.view, script, -1, nullptr,
        nullptr, nullptr, nullptr, +[](GObject* object, GAsyncResult* result, gpointer data) {
            auto& page = *static_cast<Page*>(data);
            GError* error = nullptr;
            auto* value = webkit_web_view_call_async_javascript_function_finish(WEBKIT_WEB_VIEW(object), result, &error);
            Json record = {{"event", "snapshot"}, {"index", page.index}, {"requested", page.requested},
                {"status", page.status}, {"loadFinished", page.finished}, {"navigationError", page.error}};
            if (error) {
                record["error"] = error->message;
                g_error_free(error);
            } else {
                auto* serialized = jsc_value_to_json(value, 0);
                record["document"] = Json::parse(serialized);
                g_free(serialized);
                g_object_unref(value);
            }
            emit(std::move(record));
        }, &page);
}

int main(int argc, char** argv)
{
    if (argc < 2) return 2;
    g_unsetenv("WEBKIT_INSPECTOR_SERVER");
    g_unsetenv("WEBKIT_INSPECTOR_HTTP_SERVER");
#ifdef BENCH_WPE
    auto* display = wpe_display_headless_new();
    GError* error = nullptr;
    if (!wpe_display_connect(display, &error)) {
        fprintf(stderr, "Headless display: %s\n", error->message);
        return 2;
    }
#else
    gtk_init();
#endif
    Dl_info library { };
    dladdr(reinterpret_cast<void*>(webkit_web_view_get_type), &library);
    auto* context = webkit_web_context_new();
    auto* network = webkit_network_session_new_ephemeral();
    auto* settings = webkit_settings_new();
    // Both ports see the same desktop UA. No identity/runtime override scripts.
    webkit_settings_set_user_agent(settings, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15");
    emit({{"event", "started"}, {"pid", getpid()}, {"library", library.dli_fname ? library.dli_fname : ""},
        {"cacheModel", webkit_web_context_get_cache_model(context)},
        {"webgl", bool(webkit_settings_get_enable_webgl(settings))},
        {"pageCache", bool(webkit_settings_get_enable_page_cache(settings))},
        {"defaultFont", webkit_settings_get_default_font_family(settings)}});
    for (int i = 1; i < argc; ++i) {
        auto* view = WEBKIT_WEB_VIEW(g_object_new(WEBKIT_TYPE_WEB_VIEW,
            "web-context", context, "network-session", network, "settings", settings,
#ifdef BENCH_WPE
            "display", display,
#endif
            nullptr));
        auto page = std::make_unique<Page>(Page { static_cast<unsigned>(i - 1), argv[i], view, 0, false, {} });
        auto* p = page.get();
        pages.push_back(std::move(page));
        g_signal_connect(view, "load-changed", G_CALLBACK((+[](WebKitWebView* view, WebKitLoadEvent state, Page* p) {
            if (state == WEBKIT_LOAD_STARTED) p->finished = false;
            if (state == WEBKIT_LOAD_COMMITTED || state == WEBKIT_LOAD_FINISHED) {
                auto* resource = webkit_web_view_get_main_resource(view);
                auto* response = resource ? webkit_web_resource_get_response(resource) : nullptr;
                if (response) p->status = webkit_uri_response_get_status_code(response);
            }
            if (state == WEBKIT_LOAD_FINISHED) {
                p->finished = true;
                emit({{"event", "loaded"}, {"index", p->index}, {"status", p->status},
                    {"url", webkit_web_view_get_uri(view)}, {"requested", p->requested}});
            }
        })), p);
        g_signal_connect(view, "load-failed", G_CALLBACK((+[](WebKitWebView*, WebKitLoadEvent, const char*, GError* error, Page* p) -> gboolean {
            p->error = error->message;
            emit({{"event", "failed"}, {"index", p->index}, {"error", p->error}});
            return TRUE;
        })), p);
        g_signal_connect(view, "web-process-terminated", G_CALLBACK((+[](WebKitWebView*, WebKitWebProcessTerminationReason reason, Page* p) {
            p->error = "Content process terminated: " + std::to_string(reason);
            emit({{"event", "crashed"}, {"index", p->index}, {"error", p->error}});
        })), p);
#ifdef BENCH_WPE
        auto* native = webkit_web_view_get_wpe_view(view);
        auto* toplevel = wpe_view_get_toplevel(native);
        wpe_toplevel_resize(toplevel, 1280, 720);
        // Match GTK: every view is visible, with keyboard focus on the last.
        wpe_toplevel_state_changed(toplevel, i == argc - 1 ? WPE_TOPLEVEL_STATE_ACTIVE : static_cast<WPEToplevelState>(0));
        if (i == argc - 1) wpe_view_focus_in(native);
        wpe_view_set_visible(native, TRUE);
#else
        auto* window = GTK_WINDOW(gtk_window_new());
        gtk_window_set_default_size(window, 1280, 720);
        gtk_window_set_child(window, GTK_WIDGET(view));
        gtk_window_present(window);
#endif
        webkit_web_view_load_uri(view, argv[i]);
    }
    g_unix_fd_add(STDIN_FILENO, static_cast<GIOCondition>(G_IO_IN | G_IO_HUP), +[](gint fd, GIOCondition condition, gpointer) -> gboolean {
        char command[128];
        auto count = read(fd, command, sizeof(command));
        if (count <= 0 || condition & G_IO_HUP) { g_main_loop_quit(loop); return G_SOURCE_REMOVE; }
        for (ssize_t i = 0; i < count; ++i) {
            if (command[i] == 's') for (auto& page : pages) snapshot(*page);
            if (command[i] == 'q') g_main_loop_quit(loop);
        }
        return G_SOURCE_CONTINUE;
    }, nullptr);
    loop = g_main_loop_new(nullptr, FALSE);
    g_main_loop_run(loop);
    return 0;
}
