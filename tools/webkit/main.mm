// Local WebKit host. Browser objects and protocol dispatch stay on the main run loop.
#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>
#import <WebKit/WKProcessPoolPrivate.h>
#import <WebKit/WKWebViewConfigurationPrivate.h>
#import <WebKit/WKWebViewPrivate.h>
#import <WebKit/WKUIDelegatePrivate.h>
#import <WebKit/_WKAutomationSession.h>
#import <WebKit/_WKAutomationSessionConfiguration.h>
#import <WebKit/_WKAutomationSessionDelegate.h>
#import <WebKit/_WKAutomationSessionLocalConnection.h>
#include <arpa/inet.h>
#include <unistd.h>

static const uint32_t maxMessageSize = 64 * 1024 * 1024;
static dispatch_queue_t outputQueue;

static bool transfer(int fd, void *buffer, size_t size, bool writing)
{
    auto *bytes = static_cast<unsigned char *>(buffer);
    while (size) {
        ssize_t count = writing ? write(fd, bytes, size) : read(fd, bytes, size);
        if (count < 0 && errno == EINTR)
            continue;
        if (count <= 0)
            return false;
        bytes += count;
        size -= count;
    }
    return true;
}

static void sendMessage(NSDictionary *message)
{
    NSData *data = [NSJSONSerialization dataWithJSONObject:message options:0 error:nil];
    if (!data || data.length > maxMessageSize)
        _exit(70);
    dispatch_async(outputQueue, ^{
        uint32_t size = htonl(static_cast<uint32_t>(data.length));
        if (!transfer(STDOUT_FILENO, &size, sizeof(size), true)
            || !transfer(STDOUT_FILENO, const_cast<void *>(data.bytes), data.length, true))
            _exit(74);
    });
}

static void reply(NSNumber *identifier, id result)
{
    sendMessage(@{ @"id": identifier, @"result": result ?: @{} });
}

static void fail(NSNumber *identifier, NSString *code, NSString *message)
{
    sendMessage(@{ @"id": identifier, @"error": @{ @"code": code, @"message": message } });
}

@interface BrowserHost : NSObject <_WKAutomationSessionDelegate, WKNavigationDelegate, WKUIDelegate>
@property(nonatomic, strong) WKProcessPool *pool;
@property(nonatomic, strong) WKWebsiteDataStore *store;
@property(nonatomic, strong) _WKAutomationSession *automation;
@property(nonatomic, strong) NSMutableDictionary<NSString *, WKWebView *> *pages;
@property(nonatomic, strong) NSMutableDictionary<NSString *, NSWindow *> *windows;
@property(nonatomic, strong) NSMapTable<WKWebView *, NSNumber *> *navigations;
@property(nonatomic, strong) NSMapTable<WKWebView *, WKNavigation *> *activeNavigations;
@property(nonatomic, strong) NSHashTable<WKWebView *> *earlyNavigations;
@property(nonatomic, strong) NSHashTable<WKNavigation *> *committedNavigations;
@property(nonatomic, strong) NSMapTable<WKWebView *, NSNumber *> *scriptsAllowed;
@property(nonatomic, strong) NSMutableDictionary<NSString *, NSString *> *targets;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, NSNumber *> *inspectorRequests;
@property(nonatomic, strong) NSMutableSet<NSNumber *> *intermediateRequests;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, NSString *> *requestPages;
@property(nonatomic, strong) NSMutableSet<NSString *> *pendingPopups;
@property(nonatomic) long long internalId;
@end

@implementation BrowserHost
- (instancetype)init
{
    if (!(self = [super init]))
        return nil;
    _pool = [WKProcessPool new];
    _store = [WKWebsiteDataStore nonPersistentDataStore];
    _pages = [NSMutableDictionary new];
    _windows = [NSMutableDictionary new];
    _navigations = [NSMapTable strongToStrongObjectsMapTable];
    _activeNavigations = [NSMapTable strongToStrongObjectsMapTable];
    _earlyNavigations = [NSHashTable weakObjectsHashTable];
    _committedNavigations = [NSHashTable weakObjectsHashTable];
    _scriptsAllowed = [NSMapTable strongToStrongObjectsMapTable];
    _targets = [NSMutableDictionary new];
    _inspectorRequests = [NSMutableDictionary new];
    _intermediateRequests = [NSMutableSet new];
    _requestPages = [NSMutableDictionary new];
    _pendingPopups = [NSMutableSet new];
    _internalId = -1;
    auto *configuration = [_WKAutomationSessionConfiguration new];
    configuration.navigatorWebDriverEnabled = YES;
    _automation = [[_WKAutomationSession alloc] initWithConfiguration:configuration];
    _automation.sessionIdentifier = [[NSUUID UUID] UUIDString];
    _automation.delegate = self;
    [_pool _setAutomationSession:_automation];
    __weak BrowserHost *weakSelf = self;
    [_automation _setMessageToFrontendHandler:^(NSString *message) {
        NSDictionary *object = [NSJSONSerialization JSONObjectWithData:[message dataUsingEncoding:NSUTF8StringEncoding] options:0 error:nil];
        if (object)
            [weakSelf receive:object];
    }];
    return self;
}

- (void)receive:(NSDictionary *)message
{
    if (![message[@"method"] isEqual:@"Automation.receiveInspectorMessage"]) {
        if ([message[@"id"] longLongValue] < 0)
            [self receiveInspectorReply:message];
        else
            sendMessage(message);
        return;
    }
    NSString *handle = message[@"params"][@"browsingContextHandle"];
    NSString *serialized = message[@"params"][@"message"];
    NSDictionary *inner = [NSJSONSerialization JSONObjectWithData:[serialized dataUsingEncoding:NSUTF8StringEncoding] options:0 error:nil];
    if ([inner[@"method"] isEqual:@"Target.targetCreated"]) {
        NSDictionary *info = inner[@"params"][@"targetInfo"];
        if ([info[@"type"] isEqual:@"page"] && ![info[@"isProvisional"] boolValue])
            _targets[handle] = info[@"targetId"];
    } else if ([inner[@"method"] isEqual:@"Target.didCommitProvisionalTarget"])
        _targets[handle] = inner[@"params"][@"newTargetId"];
    if ([inner[@"method"] isEqual:@"Target.dispatchMessageFromTarget"]) {
        serialized = inner[@"params"][@"message"];
        inner = [NSJSONSerialization JSONObjectWithData:[serialized dataUsingEncoding:NSUTF8StringEncoding] options:0 error:nil];
    }
    if (inner[@"id"])
        [self receiveInspectorReply:inner];
    else if (inner[@"method"]) {
        sendMessage(@{ @"method": @"Brimp.inspectorEvent", @"params": @{ @"handle": handle, @"message": inner } });
        WKWebView *page = _pages[handle];
        WKNavigation *navigation = [_activeNavigations objectForKey:page];
        if ([inner[@"method"] isEqual:@"Page.domContentEventFired"] && [_earlyNavigations containsObject:page] && [_committedNavigations containsObject:navigation])
            [self webView:page didFinishNavigation:navigation];
    }
}

- (void)receiveInspectorReply:(NSDictionary *)inner
{
    NSNumber *outer = _inspectorRequests[inner[@"id"]];
    if (outer) {
        BOOL intermediate = [_intermediateRequests containsObject:inner[@"id"]];
        [_inspectorRequests removeObjectForKey:inner[@"id"]];
        [_intermediateRequests removeObject:inner[@"id"]];
        if (intermediate && !inner[@"error"])
            return;
        for (NSNumber *alias in [_inspectorRequests allKeysForObject:outer]) {
            [_inspectorRequests removeObjectForKey:alias];
            [_intermediateRequests removeObject:alias];
        }
        [_requestPages removeObjectForKey:outer];
        NSMutableDictionary *response = [inner mutableCopy];
        response[@"id"] = outer;
        sendMessage(response);
    }
}

- (void)sendInspector:(NSDictionary *)message handle:(NSString *)handle
{
    NSData *data = [NSJSONSerialization dataWithJSONObject:message options:0 error:nil];
    NSDictionary *outer = @{ @"id": @(_internalId--), @"method": @"Automation.sendInspectorMessage", @"params": @{ @"browsingContextHandle": handle, @"message": [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding] } };
    _inspectorRequests[outer[@"id"]] = _inspectorRequests[message[@"id"]];
    [_intermediateRequests addObject:outer[@"id"]];
    data = [NSJSONSerialization dataWithJSONObject:outer options:0 error:nil];
    [_automation _dispatchMessageFromRemote:[[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding]];
}

- (BOOL)_automationSessionShouldEnableInspectorProtocol:(_WKAutomationSession *)session
{
    return YES;
}

- (WKWebView *)createPage:(WKWebViewConfiguration *)configuration opener:(WKWebView *)opener
{
    configuration = configuration ?: [WKWebViewConfiguration new];
    configuration.processPool = _pool;
    configuration.websiteDataStore = _store;
    configuration._controlledByAutomation = YES;
    configuration.preferences.javaScriptCanOpenWindowsAutomatically = YES;
    WKWebView *page = [[WKWebView alloc] initWithFrame:NSMakeRect(0, 0, 1280, 720) configuration:configuration];
    page.navigationDelegate = self;
    page.UIDelegate = self;
    page.inspectable = YES;
    NSWindow *window = [[NSWindow alloc] initWithContentRect:page.frame styleMask:NSWindowStyleMaskBorderless backing:NSBackingStoreBuffered defer:NO];
    window.releasedWhenClosed = NO;
    window.alphaValue = 0;
    window.ignoresMouseEvents = YES;
    window.contentView = page;
    NSString *handle = [_automation _registerWebView:page];
    _pages[handle] = page;
    _windows[handle] = window;
    if (opener) {
        [_pendingPopups addObject:handle];
        sendMessage(@{ @"method": @"Brimp.popup", @"params": @{ @"opener": [_automation _registerWebView:opener], @"handle": handle } });
    }
    return page;
}

- (void)_automationSession:(_WKAutomationSession *)session requestNewWebViewWithOptions:(_WKAutomationSessionBrowsingContextOptions)options completionHandler:(void (^)(WKWebView *))completionHandler
{
    completionHandler([self createPage:nil opener:nil]);
}

- (void)_automationSession:(_WKAutomationSession *)session requestSwitchToWebView:(WKWebView *)webView completionHandler:(void (^)(void))completionHandler
{
    completionHandler();
}

- (void)_webView:(WKWebView *)webView getWindowFrameWithCompletionHandler:(void (^)(CGRect))completionHandler
{
    completionHandler(webView.window.frame);
}

- (WKWebView *)webView:(WKWebView *)webView createWebViewWithConfiguration:(WKWebViewConfiguration *)configuration forNavigationAction:(WKNavigationAction *)action windowFeatures:(WKWindowFeatures *)features
{
    WKWebView *page = [self createPage:configuration opener:webView];
    if (!action.request.URL || [action.request.URL.absoluteString isEqual:@"about:blank"])
        dispatch_async(dispatch_get_main_queue(), ^{ [self popupReady:page]; });
    return page;
}

- (void)popupReady:(WKWebView *)page
{
    NSString *handle = [_pages allKeysForObject:page].firstObject;
    if (handle && [_pendingPopups containsObject:handle]) {
        [_pendingPopups removeObject:handle];
        sendMessage(@{ @"method": @"Brimp.popupReady", @"params": @{ @"handle": handle } });
    }
}

- (void)webViewDidClose:(WKWebView *)page
{
    NSString *handle = [_pages allKeysForObject:page].firstObject;
    if (!handle)
        return;
    NSNumber *pending = [_navigations objectForKey:page];
    if (pending)
        fail(pending, @"closed", @"Page closed during navigation");
    [_navigations removeObjectForKey:page];
    [_activeNavigations removeObjectForKey:page];
    [_scriptsAllowed removeObjectForKey:page];
    [_windows[handle] close];
    [_windows removeObjectForKey:handle];
    [_pages removeObjectForKey:handle];
    [_targets removeObjectForKey:handle];
    [_pendingPopups removeObject:handle];
    for (NSNumber *request in [_requestPages allKeysForObject:handle]) {
        fail(request, @"closed", @"Page closed during operation");
        [_requestPages removeObjectForKey:request];
        for (NSNumber *alias in [_inspectorRequests allKeysForObject:request]) {
            [_inspectorRequests removeObjectForKey:alias];
            [_intermediateRequests removeObject:alias];
        }
    }
    sendMessage(@{ @"method": @"Brimp.closed", @"params": @{ @"handle": handle } });
}

- (void)webView:(WKWebView *)page decidePolicyForNavigationAction:(WKNavigationAction *)action preferences:(WKWebpagePreferences *)preferences decisionHandler:(void (^)(WKNavigationActionPolicy, WKWebpagePreferences *))decisionHandler
{
    NSNumber *allowed = [_scriptsAllowed objectForKey:page];
    preferences.allowsContentJavaScript = !allowed || allowed.boolValue;
    decisionHandler(WKNavigationActionPolicyAllow, preferences);
}

- (void)webViewWebContentProcessDidTerminate:(WKWebView *)page
{
    NSString *handle = [_pages allKeysForObject:page].firstObject;
    if (!handle)
        return;
    sendMessage(@{ @"method": @"Brimp.crashed", @"params": @{ @"handle": handle } });
    [self webViewDidClose:page];
}

- (void)webView:(WKWebView *)page didFinishNavigation:(WKNavigation *)navigation
{
    [self popupReady:page];
    if ([_activeNavigations objectForKey:page] != navigation)
        return;
    NSNumber *pending = [_navigations objectForKey:page];
    if (pending) {
        [_navigations removeObjectForKey:page];
        [_activeNavigations removeObjectForKey:page];
        if ([[_scriptsAllowed objectForKey:page] isEqual:@NO]) {
            [_scriptsAllowed setObject:@YES forKey:page];
            WKWebpagePreferences *preferences = [WKWebpagePreferences new];
            preferences.allowsContentJavaScript = YES;
            [page _updateWebpagePreferences:preferences];
        }
        reply(pending, @{ @"url": page.URL.absoluteString ?: @"about:blank" });
    }
}

- (void)webView:(WKWebView *)page didCommitNavigation:(WKNavigation *)navigation
{
    [_committedNavigations addObject:navigation];
}

- (void)webView:(WKWebView *)page didFailNavigation:(WKNavigation *)navigation withError:(NSError *)error
{
    [self popupReady:page];
    if ([_activeNavigations objectForKey:page] != navigation)
        return;
    NSNumber *pending = [_navigations objectForKey:page];
    if (pending) {
        [_navigations removeObjectForKey:page];
        [_activeNavigations removeObjectForKey:page];
        fail(pending, @"navigation", error.localizedDescription);
    }
}

- (void)webView:(WKWebView *)page didFailProvisionalNavigation:(WKNavigation *)navigation withError:(NSError *)error
{
    [self webView:page didFailNavigation:navigation withError:error];
}

- (void)dispatch:(NSDictionary *)message
{
    NSNumber *identifier = message[@"id"];
    NSString *method = message[@"method"];
    NSDictionary *params = message[@"params"] ?: @{};
    if (![identifier isKindOfClass:NSNumber.class] || ![method isKindOfClass:NSString.class])
        _exit(65);
    if ([method hasPrefix:@"Automation."]) {
        NSData *data = [NSJSONSerialization dataWithJSONObject:message options:0 error:nil];
        [_automation _dispatchMessageFromRemote:[[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding]];
        return;
    }
    if ([method isEqualToString:@"Brimp.hello"]) {
        reply(identifier, @{ @"protocol": @1, @"engine": @"webkit", @"pid": @(getpid()), @"framework": [NSBundle bundleForClass:WKWebView.class].bundlePath });
        return;
    }
    if ([method isEqualToString:@"Brimp.cookies"]) {
        [_store.httpCookieStore getAllCookies:^(NSArray<NSHTTPCookie *> *cookies) {
            NSMutableArray *records = [NSMutableArray new];
            for (NSHTTPCookie *cookie in cookies) {
                [records addObject:@{ @"name": cookie.name, @"value": cookie.value, @"domain": cookie.domain,
                    @"host_only": @(![cookie.domain hasPrefix:@"."]), @"path": cookie.path,
                    @"expires": cookie.expiresDate ? @((long long)cookie.expiresDate.timeIntervalSince1970) : NSNull.null,
                    @"http_only": @(cookie.HTTPOnly), @"secure": @(cookie.secure),
                    @"same_site": [cookie.properties[NSHTTPCookieSameSitePolicy] capitalizedString] ?: NSNull.null }];
            }
            reply(identifier, records);
        }];
        return;
    }
    if ([method isEqualToString:@"Brimp.setCookie"]) {
        NSURL *url = [NSURL URLWithString:params[@"url"]];
        NSHTTPCookie *cookie = url.host ? [NSHTTPCookie cookieWithProperties:@{ NSHTTPCookieName: params[@"name"], NSHTTPCookieValue: params[@"value"], NSHTTPCookieDomain: url.host, NSHTTPCookiePath: @"/", NSHTTPCookieOriginURL: url }] : nil;
        if (!cookie) fail(identifier, @"invalid_input", @"Invalid cookie or URL");
        else [_store.httpCookieStore setCookie:cookie completionHandler:^{ reply(identifier, @{}); }];
        return;
    }
    if ([method isEqualToString:@"Brimp.clearCookies"]) {
        [_store.httpCookieStore getAllCookies:^(NSArray<NSHTTPCookie *> *cookies) {
            dispatch_group_t group = dispatch_group_create();
            for (NSHTTPCookie *cookie in cookies) {
                dispatch_group_enter(group);
                [self.store.httpCookieStore deleteCookie:cookie completionHandler:^{ dispatch_group_leave(group); }];
            }
            dispatch_group_notify(group, dispatch_get_main_queue(), ^{ reply(identifier, @{}); });
        }];
        return;
    }
    if ([method isEqualToString:@"Brimp.clearCache"]) {
        NSSet *types = [NSSet setWithArray:@[ WKWebsiteDataTypeDiskCache, WKWebsiteDataTypeMemoryCache, WKWebsiteDataTypeFetchCache ]];
        [_store removeDataOfTypes:types modifiedSince:NSDate.distantPast completionHandler:^{ reply(identifier, @{}); }];
        return;
    }
    if ([method isEqualToString:@"Brimp.close"]) {
        for (WKWebView *page in _pages.allValues) {
            [self webViewDidClose:page];
            [page _close];
        }
        [_automation _setMessageToFrontendHandler:nil];
        [_automation terminate];
        reply(identifier, @{});
        dispatch_async(outputQueue, ^{ exit(0); });
        return;
    }
    WKWebView *page = _pages[params[@"handle"]];
    if (!page) {
        fail(identifier, @"closed", @"Unknown or closed page");
        return;
    }
    if ([method isEqualToString:@"Brimp.closePage"]) {
        [page stopLoading];
        [self webViewDidClose:page];
        [page _close];
        reply(identifier, @{});
    } else if ([method isEqualToString:@"Brimp.terminatePage"]) {
        [page _killWebContentProcess];
        reply(identifier, @{});
    } else if ([method isEqualToString:@"Brimp.addInitScript"]) {
        WKUserScript *script = [[WKUserScript alloc] initWithSource:params[@"source"] injectionTime:WKUserScriptInjectionTimeAtDocumentStart forMainFrameOnly:NO];
        [page.configuration.userContentController addUserScript:script];
        reply(identifier, @{});
    } else if ([method isEqualToString:@"Brimp.loadHTML"]) {
        if ([_navigations objectForKey:page]) {
            fail(identifier, @"invalid_input", @"Navigation is already in progress");
            return;
        }
        [_scriptsAllowed setObject:@NO forKey:page];
        [_earlyNavigations removeObject:page];
        [_navigations setObject:identifier forKey:page];
        [_activeNavigations setObject:[page loadHTMLString:params[@"html"] baseURL:[NSURL URLWithString:params[@"url"]]] forKey:page];
    } else if ([method isEqualToString:@"Brimp.inspector"]) {
        NSString *handle = params[@"handle"];
        NSString *command = params[@"method"];
        NSNumber *innerId = @(_internalId--);
        NSDictionary *inner = @{ @"id": innerId, @"method": command, @"params": params[@"params"] ?: @{} };
        if (![command hasPrefix:@"Target."]) {
            NSString *target = params[@"target"] ?: _targets[handle];
            if (!target) {
                fail(identifier, @"invalid_input", @"Inspector page target is not ready");
                return;
            }
            NSData *data = [NSJSONSerialization dataWithJSONObject:inner options:0 error:nil];
            inner = @{ @"id": @(_internalId--), @"method": @"Target.sendMessageToTarget", @"params": @{ @"targetId": target, @"message": [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding] } };
            _inspectorRequests[inner[@"id"]] = identifier;
            [_intermediateRequests addObject:inner[@"id"]];
        }
        _inspectorRequests[innerId] = identifier;
        _requestPages[identifier] = handle;
        [self sendInspector:inner handle:handle];
    } else if ([method isEqualToString:@"Brimp.request"]) {
        if ([_navigations objectForKey:page]) {
            fail(identifier, @"invalid_input", @"Navigation is already in progress");
            return;
        }
        NSURL *url = [NSURL URLWithString:params[@"url"]];
        if (!url) {
            fail(identifier, @"invalid_input", @"Invalid URL");
            return;
        }
        NSMutableURLRequest *request = [NSMutableURLRequest requestWithURL:url];
        request.HTTPMethod = params[@"method"] ?: @"GET";
        for (NSArray *header in params[@"headers"])
            [request addValue:header[1] forHTTPHeaderField:header[0]];
        if ([params[@"body"] isKindOfClass:NSString.class])
            request.HTTPBody = [[NSData alloc] initWithBase64EncodedString:params[@"body"] options:0];
        [_navigations setObject:identifier forKey:page];
        [_scriptsAllowed setObject:@YES forKey:page];
        if ([params[@"waitUntil"] isEqual:@"domcontentloaded"])
            [_earlyNavigations addObject:page];
        else
            [_earlyNavigations removeObject:page];
        [_activeNavigations setObject:[page loadRequest:request] forKey:page];
    } else if ([method isEqualToString:@"Brimp.stopLoading"]) {
        NSNumber *pending = [_navigations objectForKey:page];
        [_navigations removeObjectForKey:page];
        [_activeNavigations removeObjectForKey:page];
        if (pending)
            fail(pending, @"cancelled", @"Navigation cancelled");
        [page stopLoading];
        reply(identifier, @{});
    } else {
        fail(identifier, @"unsupported", [@"Unknown method: " stringByAppendingString:method]);
    }
}
@end

int main()
{
    @autoreleasepool {
        signal(SIGPIPE, SIG_IGN);
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
        outputQueue = dispatch_queue_create("brimp.webkit.output", DISPATCH_QUEUE_SERIAL);
        BrowserHost *host = [BrowserHost new];
        dispatch_async(dispatch_get_global_queue(QOS_CLASS_USER_INITIATED, 0), ^{
            while (true) {
                @autoreleasepool {
                    uint32_t size;
                    if (!transfer(STDIN_FILENO, &size, sizeof(size), false))
                        break;
                    size = ntohl(size);
                    if (!size || size > maxMessageSize)
                        _exit(65);
                    NSMutableData *data = [NSMutableData dataWithLength:size];
                    if (!transfer(STDIN_FILENO, data.mutableBytes, size, false))
                        break;
                    NSDictionary *message = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
                    if (![message isKindOfClass:NSDictionary.class])
                        _exit(65);
                    dispatch_async(dispatch_get_main_queue(), ^{ [host dispatch:message]; });
                }
            }
            dispatch_async(dispatch_get_main_queue(), ^{
                [host dispatch:@{ @"id": @0, @"method": @"Brimp.close" }];
            });
        });
        [NSApp run];
    }
    return 0;
}
