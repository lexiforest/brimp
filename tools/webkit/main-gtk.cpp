// WebKitGTK embedding host. Only the GTK thread touches browser objects.
// The Rust adapter owns browser semantics; this host carries its local protocol.
#include <webkit/webkit.h>
#include <nlohmann/json.hpp>
#include <arpa/inet.h>
#include <dlfcn.h>
#include <unistd.h>
#include <condition_variable>
#include <cerrno>
#include <cstdint>
#include <csignal>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <deque>
#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <set>
#include <string>
#include <stdexcept>
#include <thread>
#include <utility>

using Json = nlohmann::json;
using Id = int64_t;
static constexpr uint32_t maxMessageSize = 64 * 1024 * 1024;

static bool transfer(int fd, void* buffer, size_t size, bool writing)
{
    auto* bytes = static_cast<char*>(buffer);
    while (size) {
        auto count = writing ? write(fd, bytes, size) : read(fd, bytes, size);
        if (count < 0 && errno == EINTR)
            continue;
        if (count <= 0)
            return false;
        bytes += count;
        size -= count;
    }
    return true;
}

// Pipe backpressure must never block GTK or prevent termination of a hung page.
class Output {
    std::mutex mutex;
    std::condition_variable ready;
    std::deque<std::string> queue;
    size_t queuedBytes = 0;
    bool closing = false;
public:
    void start()
    {
        std::thread([this] {
            for (;;) {
                std::unique_lock lock(mutex);
                ready.wait(lock, [this] { return closing || !queue.empty(); });
                if (queue.empty())
                    std::_Exit(0);
                auto data = std::move(queue.front());
                queue.pop_front();
                queuedBytes -= data.size();
                lock.unlock();
                uint32_t size = htonl(data.size());
                if (!transfer(STDOUT_FILENO, &size, sizeof(size), true)
                    || !transfer(STDOUT_FILENO, data.data(), data.size(), true))
                    std::_Exit(74);
            }
        }).detach();
    }
    void send(const Json& message)
    {
        auto data = message.dump();
        std::lock_guard lock(mutex);
        if (data.size() > maxMessageSize || queuedBytes + data.size() > 4 * maxMessageSize)
            std::_Exit(70);
        queuedBytes += data.size();
        queue.push_back(std::move(data));
        ready.notify_one();
    }
    void close()
    {
        std::lock_guard lock(mutex);
        closing = true;
        ready.notify_one();
    }
};

class Host {
    struct Page {
        Host* host;
        std::string handle;
        WebKitWebView* view;
        GtkWindow* window;
        Id navigation = 0;
        bool early = false;
        bool committed = false;
        bool popup = false;
    };
    Output output;
    WebKitWebContext* context;
    WebKitAutomationSession* automation;
    WebKitNetworkSession* network;
    std::map<std::string, std::unique_ptr<Page>> pages;
    std::map<std::string, std::string> targets;
    std::map<Id, Id> inspectorRequests;
    std::set<Id> intermediateRequests;
    std::map<Id, std::string> requestPages;
    Id internalId = -1;
    bool closing = false;
    std::string libraryPath;

    void reply(Id id, Json result = Json::object()) { output.send({{"id", id}, {"result", result}}); }
    void fail(Id id, const char* code, const std::string& message)
    {
        output.send({{"id", id}, {"error", {{"code", code}, {"message", message}}}});
    }
    void event(const char* name, Json params) { output.send({{"method", name}, {"params", params}}); }
    void sendAutomation(const Json& message)
    {
        auto data = message.dump();
        webkit_automation_session_dispatch_message(automation, data.c_str());
    }
    void forgetRequest(Id outer)
    {
        for (auto it = inspectorRequests.begin(); it != inspectorRequests.end();) {
            if (it->second == outer) {
                intermediateRequests.erase(it->first);
                it = inspectorRequests.erase(it);
            } else
                ++it;
        }
        requestPages.erase(outer);
    }
    void inspectorReply(Json message)
    {
        auto found = inspectorRequests.find(message.value("id", Id(0)));
        if (found == inspectorRequests.end())
            return;
        Id inner = found->first;
        Id outer = found->second;
        inspectorRequests.erase(found);
        bool intermediate = intermediateRequests.erase(inner);
        if (intermediate && !message.contains("error"))
            return;
        forgetRequest(outer);
        message["id"] = outer;
        output.send(message);
    }
    void receive(Json message)
    {
        if (message.value("method", "") != "Automation.receiveInspectorMessage") {
            if (message.value("id", Id(0)) < 0)
                inspectorReply(std::move(message));
            else
                output.send(message);
            return;
        }
        auto& params = message.at("params");
        std::string handle = params.at("browsingContextHandle");
        auto inner = Json::parse(params.at("message").get<std::string>());
        if (inner.value("method", "") == "Target.targetCreated") {
            auto& info = inner["params"]["targetInfo"];
            if (info.value("type", "") == "page" && !info.value("isProvisional", false))
                targets[handle] = info.at("targetId");
        } else if (inner.value("method", "") == "Target.didCommitProvisionalTarget")
            targets[handle] = inner["params"].at("newTargetId");
        if (inner.value("method", "") == "Target.dispatchMessageFromTarget")
            inner = Json::parse(inner["params"].at("message").get<std::string>());
        if (inner.contains("id"))
            inspectorReply(std::move(inner));
        else if (inner.contains("method")) {
            event("Brimp.inspectorEvent", {{"handle", handle}, {"message", inner}});
            auto found = pages.find(handle);
            if (found != pages.end() && found->second->early && found->second->committed
                && inner["method"] == "Page.domContentEventFired")
                finishNavigation(*found->second);
        }
    }
    void popupReady(Page& page)
    {
        if (page.popup) {
            page.popup = false;
            event("Brimp.popupReady", {{"handle", page.handle}});
        }
    }
    void finishNavigation(Page& page)
    {
        popupReady(page);
        if (!page.navigation)
            return;
        Id id = std::exchange(page.navigation, 0);
        webkit_settings_set_enable_javascript_markup(webkit_web_view_get_settings(page.view), TRUE);
        const char* uri = webkit_web_view_get_uri(page.view);
        reply(id, {{"url", uri ? uri : "about:blank"}});
    }
    void closePage(std::string handle, Id closeId = 0)
    {
        auto found = pages.find(handle);
        if (found == pages.end())
            return;
        auto page = std::move(found->second);
        pages.erase(found);
        targets.erase(handle);
        if (page->navigation)
            fail(page->navigation, "closed", "Page closed during navigation");
        for (auto it = requestPages.begin(); it != requestPages.end();) {
            if (it->second != handle) {
                ++it;
                continue;
            }
            Id id = it->first;
            ++it;
            fail(id, "closed", "Page closed during operation");
            forgetRequest(id);
        }
        g_signal_handlers_disconnect_by_data(page->view, page.get());
        webkit_web_view_stop_loading(page->view);
        webkit_web_view_close(page->view, nullptr, complete, new Completion([this, closeId](GObject* object, GAsyncResult* result) {
            GError* error = nullptr;
            webkit_web_view_close_finish(WEBKIT_WEB_VIEW(object), result, &error);
            if (closeId)
                asyncResult(closeId, error);
            else if (error)
                g_error_free(error);
        }));
        gtk_window_destroy(page->window);
        g_object_unref(page->window);
        event("Brimp.closed", {{"handle", handle}});
    }
    WebKitWebView* createPage(Page* opener = nullptr)
    {
        auto* settings = webkit_settings_new();
        webkit_settings_set_javascript_can_open_windows_automatically(settings, TRUE);
        auto* view = WEBKIT_WEB_VIEW(opener
            ? g_object_new(WEBKIT_TYPE_WEB_VIEW, "related-view", opener->view, "settings", settings, nullptr)
            : g_object_new(WEBKIT_TYPE_WEB_VIEW, "web-context", context,
                "is-controlled-by-automation", TRUE, "settings", settings, nullptr));
        g_object_unref(settings);
        auto* window = GTK_WINDOW(gtk_window_new());
        g_object_ref_sink(window);
        gtk_window_set_default_size(window, 1280, 720);
        gtk_window_set_child(window, GTK_WIDGET(view));
        auto* registered = webkit_automation_session_register_web_view(automation, view);
        if (!registered)
            std::_Exit(70);
        std::string handle = registered;
        g_free(registered);
        auto page = std::make_unique<Page>(Page { this, handle, view, window });
        auto* p = page.get();
        pages.emplace(handle, std::move(page));
        g_signal_connect(view, "load-changed", G_CALLBACK((+[](WebKitWebView*, WebKitLoadEvent state, Page* p) {
            if (state == WEBKIT_LOAD_STARTED)
                p->committed = false;
            else if (state == WEBKIT_LOAD_COMMITTED)
                p->committed = true;
            else if (state == WEBKIT_LOAD_FINISHED)
                p->host->finishNavigation(*p);
        })), p);
        g_signal_connect(view, "load-failed", G_CALLBACK((+[](WebKitWebView*, WebKitLoadEvent, const char*, GError* error, Page* p) -> gboolean {
            p->host->popupReady(*p);
            if (p->navigation)
                p->host->fail(std::exchange(p->navigation, 0), "navigation", error->message);
            webkit_settings_set_enable_javascript_markup(webkit_web_view_get_settings(p->view), TRUE);
            return TRUE; // Do not navigate to a synthetic error document.
        })), p);
        g_signal_connect(view, "close", G_CALLBACK((+[](WebKitWebView*, Page* p) {
            auto handle = p->handle;
            p->host->closePage(handle);
        })), p);
        g_signal_connect(view, "web-process-terminated", G_CALLBACK((+[](WebKitWebView*, WebKitWebProcessTerminationReason, Page* p) {
            auto handle = p->handle;
            p->host->event("Brimp.crashed", {{"handle", handle}});
            p->host->closePage(handle);
        })), p);
        g_signal_connect(view, "create", G_CALLBACK((+[](WebKitWebView*, WebKitNavigationAction* action, Page* p) -> GtkWidget* {
            auto* child = p->host->createPage(p);
            const char* uri = webkit_uri_request_get_uri(webkit_navigation_action_get_request(action));
            if (!uri || !*uri || !strcmp(uri, "about:blank")) {
                auto* handle = webkit_automation_session_register_web_view(p->host->automation, child);
                auto* data = new std::pair<Host*, std::string>(p->host, handle);
                g_free(handle);
                g_idle_add_full(G_PRIORITY_DEFAULT, +[](gpointer raw) -> gboolean {
                    auto& [host, handle] = *static_cast<std::pair<Host*, std::string>*>(raw);
                    if (auto found = host->pages.find(handle); found != host->pages.end())
                        host->popupReady(*found->second);
                    return G_SOURCE_REMOVE;
                }, data, +[](gpointer raw) { delete static_cast<std::pair<Host*, std::string>*>(raw); });
            }
            return GTK_WIDGET(child);
        })), p);
        gtk_window_present(window);
        gtk_widget_grab_focus(GTK_WIDGET(view));
        if (opener) {
            p->popup = true;
            event("Brimp.popup", {{"opener", opener->handle}, {"handle", handle}});
        } else
            webkit_web_view_load_uri(view, "about:blank");
        return view;
    }
    using Completion = std::function<void(GObject*, GAsyncResult*)>;
    static void complete(GObject* object, GAsyncResult* result, gpointer raw)
    {
        std::unique_ptr<Completion> callback(static_cast<Completion*>(raw));
        (*callback)(object, result);
    }
    void asyncResult(Id id, GError* error)
    {
        if (error) {
            fail(id, "transport", error->message);
            g_error_free(error);
        } else
            reply(id);
    }
public:
    Host()
    {
        Dl_info info { };
        if (!dladdr(reinterpret_cast<void*>(webkit_web_view_get_type), &info) || !info.dli_fname)
            throw std::runtime_error("Cannot identify loaded WebKit library");
        char* actual = realpath(info.dli_fname, nullptr);
        char* expected = realpath(g_getenv("BRIMP_WEBKIT_LIBRARY") ? g_getenv("BRIMP_WEBKIT_LIBRARY") : "", nullptr);
        if (!actual || !expected || strcmp(actual, expected))
            throw std::runtime_error("Loaded WebKit library does not match the selected SDK");
        libraryPath = actual;
        free(actual);
        free(expected);
        context = webkit_web_context_new();
        if (const char* root = g_getenv("BRIMP_WEBKIT_ROOT"))
            webkit_web_context_add_path_to_sandbox(context, root, TRUE);
        webkit_web_context_set_automation_allowed(context, TRUE);
        auto* id = g_uuid_string_random();
        automation = webkit_web_context_create_local_automation_session(context, id);
        g_free(id);
        if (!automation)
            throw std::runtime_error("Could not create local automation session");
        network = webkit_web_context_get_network_session_for_automation(context);
        g_signal_connect(automation, "create-web-view", G_CALLBACK((+[](WebKitAutomationSession*, Host* host) -> WebKitWebView* {
            return host->createPage();
        })), this);
        g_signal_connect(automation, "message-to-frontend", G_CALLBACK((+[](WebKitAutomationSession*, const char* message, Host* host) {
            host->receive(Json::parse(message));
        })), this);
        webkit_automation_session_connect_local(automation);
        output.start();
    }
    void dispatch(const Json& message)
    {
        if (closing)
            return;
        Id id = message.at("id").get<Id>();
        auto method = message.at("method").get<std::string>();
        Json params = message.value("params", Json::object());
        if (method.starts_with("Automation.")) {
            if (auto found = pages.find(params.value("handle", "")); found != pages.end()) {
                gtk_window_present(found->second->window);
                gtk_widget_grab_focus(GTK_WIDGET(found->second->view));
            }
            sendAutomation(message);
            return;
        }
        if (method == "Brimp.hello") {
            reply(id, {{"protocol", 1}, {"engine", "webkit"}, {"pid", getpid()}, {"framework", libraryPath}});
            return;
        }
        if (method == "Brimp.close") {
            closing = true;
            while (!pages.empty())
                closePage(pages.begin()->first);
            webkit_automation_session_disconnect_local(automation);
            g_object_unref(automation);
            g_object_unref(context);
            reply(id);
            output.close();
            return;
        }
        auto* cookies = webkit_network_session_get_cookie_manager(network);
        if (method == "Brimp.cookies") {
            webkit_cookie_manager_get_all_cookies(cookies, nullptr, complete, new Completion([this, id](GObject* object, GAsyncResult* result) {
                GError* error = nullptr;
                auto* list = webkit_cookie_manager_get_all_cookies_finish(WEBKIT_COOKIE_MANAGER(object), result, &error);
                if (error) { asyncResult(id, error); return; }
                auto records = Json::array();
                for (auto* item = list; item; item = item->next) {
                    auto* cookie = static_cast<SoupCookie*>(item->data);
                    const char* domain = soup_cookie_get_domain(cookie);
                    auto* expires = soup_cookie_get_expires(cookie);
                    Json sameSite = nullptr;
                    switch (soup_cookie_get_same_site_policy(cookie)) {
                    case SOUP_SAME_SITE_POLICY_LAX: sameSite = "Lax"; break;
                    case SOUP_SAME_SITE_POLICY_STRICT: sameSite = "Strict"; break;
                    default: break;
                    }
                    records.push_back({{"name", soup_cookie_get_name(cookie)}, {"value", soup_cookie_get_value(cookie)},
                        {"domain", domain}, {"host_only", domain[0] != '.'}, {"path", soup_cookie_get_path(cookie)},
                        {"expires", expires ? Json(g_date_time_to_unix(expires)) : Json(nullptr)},
                        {"http_only", bool(soup_cookie_get_http_only(cookie))}, {"secure", bool(soup_cookie_get_secure(cookie))},
                        {"same_site", sameSite}});
                }
                g_list_free_full(list, reinterpret_cast<GDestroyNotify>(soup_cookie_free));
                reply(id, std::move(records));
            }));
            return;
        }
        if (method == "Brimp.setCookie") {
            auto* uri = g_uri_parse(params.at("url").get<std::string>().c_str(), G_URI_FLAGS_NONE, nullptr);
            auto* cookie = uri && g_uri_get_host(uri) ? soup_cookie_new(params.at("name").get<std::string>().c_str(),
                params.at("value").get<std::string>().c_str(), g_uri_get_host(uri), "/", -1) : nullptr;
            if (uri) g_uri_unref(uri);
            if (!cookie) { fail(id, "invalid_input", "Invalid cookie or URL"); return; }
            webkit_cookie_manager_add_cookie(cookies, cookie, nullptr, complete, new Completion([this, id](GObject* object, GAsyncResult* result) {
                GError* error = nullptr;
                webkit_cookie_manager_add_cookie_finish(WEBKIT_COOKIE_MANAGER(object), result, &error);
                asyncResult(id, error);
            }));
            soup_cookie_free(cookie);
            return;
        }
        if (method == "Brimp.clearCookies") {
            webkit_cookie_manager_replace_cookies(cookies, nullptr, nullptr, complete, new Completion([this, id](GObject* object, GAsyncResult* result) {
                GError* error = nullptr;
                webkit_cookie_manager_replace_cookies_finish(WEBKIT_COOKIE_MANAGER(object), result, &error);
                asyncResult(id, error);
            }));
            return;
        }
        if (method == "Brimp.clearCache") {
            auto* manager = webkit_network_session_get_website_data_manager(network);
            auto types = static_cast<WebKitWebsiteDataTypes>(WEBKIT_WEBSITE_DATA_MEMORY_CACHE | WEBKIT_WEBSITE_DATA_DISK_CACHE | WEBKIT_WEBSITE_DATA_DOM_CACHE);
            webkit_website_data_manager_clear(manager, types, 0, nullptr, complete, new Completion([this, id](GObject* object, GAsyncResult* result) {
                GError* error = nullptr;
                webkit_website_data_manager_clear_finish(WEBKIT_WEBSITE_DATA_MANAGER(object), result, &error);
                asyncResult(id, error);
            }));
            return;
        }
        std::string handle = params.value("handle", "");
        auto found = pages.find(handle);
        if (found == pages.end()) { fail(id, "closed", "Unknown or closed page"); return; }
        auto& page = *found->second;
        if (method == "Brimp.closePage") {
            closePage(handle, id);
        } else if (method == "Brimp.terminatePage") {
            webkit_web_view_terminate_web_process(page.view);
            reply(id);
        } else if (method == "Brimp.addInitScript") {
            auto* script = webkit_user_script_new(params.at("source").get<std::string>().c_str(), WEBKIT_USER_CONTENT_INJECT_ALL_FRAMES,
                WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START, nullptr, nullptr);
            webkit_user_content_manager_add_script(webkit_web_view_get_user_content_manager(page.view), script);
            webkit_user_script_unref(script);
            reply(id);
        } else if (method == "Brimp.stopLoading") {
            if (page.navigation)
                fail(std::exchange(page.navigation, 0), "cancelled", "Navigation cancelled");
            webkit_web_view_stop_loading(page.view);
            reply(id);
        } else if (method == "Brimp.inspector") {
            auto command = params.at("method").get<std::string>();
            Id innerId = internalId--;
            Json inner = {{"id", innerId}, {"method", command}, {"params", params.value("params", Json::object())}};
            if (!command.starts_with("Target.")) {
                auto target = params.value("target", targets[handle]);
                if (target.empty()) { fail(id, "invalid_input", "Inspector page target is not ready"); return; }
                Id wrapper = internalId--;
                inner = {{"id", wrapper}, {"method", "Target.sendMessageToTarget"}, {"params", {{"targetId", target}, {"message", inner.dump()}}}};
                inspectorRequests[wrapper] = id;
                intermediateRequests.insert(wrapper);
            }
            inspectorRequests[innerId] = id;
            requestPages[id] = handle;
            Id wrapper = internalId--;
            inspectorRequests[wrapper] = id;
            intermediateRequests.insert(wrapper);
            sendAutomation({{"id", wrapper}, {"method", "Automation.sendInspectorMessage"}, {"params", {{"browsingContextHandle", handle}, {"message", inner.dump()}}}});
        } else if (method == "Brimp.loadHTML" || method == "Brimp.request") {
            if (page.navigation) { fail(id, "invalid_input", "Navigation is already in progress"); return; }
            page.navigation = id;
            page.committed = false;
            page.early = params.value("waitUntil", "load") == "domcontentloaded";
            webkit_settings_set_enable_javascript_markup(webkit_web_view_get_settings(page.view), method != "Brimp.loadHTML");
            auto url = params.at("url").get<std::string>();
            if (method == "Brimp.loadHTML")
                webkit_web_view_load_html(page.view, params.at("html").get<std::string>().c_str(), url.c_str());
            else {
                auto* request = webkit_uri_request_new(url.c_str());
                webkit_uri_request_set_http_method(request, params.value("method", "GET").c_str());
                auto* headers = webkit_uri_request_get_http_headers(request);
                for (auto& header : params.value("headers", Json::array()))
                    soup_message_headers_append(headers, header.at(0).get<std::string>().c_str(), header.at(1).get<std::string>().c_str());
                if (params.contains("body") && params["body"].is_string()) {
                    gsize size;
                    auto* data = g_base64_decode(params["body"].get<std::string>().c_str(), &size);
                    auto* body = g_bytes_new_take(data, size);
                    webkit_uri_request_set_http_body(request, body);
                    g_bytes_unref(body);
                }
                webkit_web_view_load_request(page.view, request);
                g_object_unref(request);
            }
        } else
            fail(id, "unsupported", "Unknown method: " + method);
    }
};

int main()
{
    signal(SIGPIPE, SIG_IGN);
    // This host uses only its inherited pipes, never the optional debug servers.
    g_unsetenv("WEBKIT_INSPECTOR_SERVER");
    g_unsetenv("WEBKIT_INSPECTOR_HTTP_SERVER");
    try {
        gtk_init();
        auto* host = new Host;
        std::thread([host] {
            for (;;) {
                uint32_t size;
                Json message;
                if (!transfer(STDIN_FILENO, &size, sizeof(size), false))
                    message = {{"id", 0}, {"method", "Brimp.close"}};
                else {
                    size = ntohl(size);
                    if (!size || size > maxMessageSize) std::_Exit(65);
                    std::string data(size, '\0');
                    if (!transfer(STDIN_FILENO, data.data(), size, false)) std::_Exit(65);
                    message = Json::parse(data, nullptr, false);
                    if (!message.is_object() || !message.contains("id") || !message["id"].is_number_integer()
                        || !message.contains("method") || !message["method"].is_string())
                        std::_Exit(65);
                }
                bool eof = message["id"] == 0;
                auto* data = new std::pair<Host*, Json>(host, std::move(message));
                g_idle_add_full(G_PRIORITY_DEFAULT, +[](gpointer raw) -> gboolean {
                    auto& [host, message] = *static_cast<std::pair<Host*, Json>*>(raw);
                    try { host->dispatch(message); }
                    catch (const std::exception& error) {
                        fprintf(stderr, "brimp-webkit: %s\n", error.what());
                        std::_Exit(65);
                    }
                    return G_SOURCE_REMOVE;
                }, data, +[](gpointer raw) { delete static_cast<std::pair<Host*, Json>*>(raw); });
                if (eof) break;
            }
        }).detach();
        auto* loop = g_main_loop_new(nullptr, FALSE);
        g_main_loop_run(loop);
    } catch (const std::exception& error) {
        fprintf(stderr, "brimp-webkit: %s\n", error.what());
        return 70;
    }
}
