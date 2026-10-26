#include "lexbor.h"
#include "lexbor/dom/interfaces/node.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

static void found(br_node *node, void *ctx) { *(br_node **)ctx = node; }
static br_node *query(br_node *root, const char *selector) {
    br_node *node = NULL;
    assert(br_select(root, (const unsigned char *)selector, strlen(selector), false, found, &node) == 0);
    return node;
}
static void text_is(br_node *node, const char *text) {
    br_bytes bytes = {0};
    assert(br_node_text(node, &bytes) == 0);
    assert(bytes.len == strlen(text));
    assert(!memcmp(bytes.data, text, bytes.len));
    br_bytes_free(&bytes);
}
static void declarations(const unsigned char *name, size_t name_len, const unsigned char *value, size_t value_len, bool important, void *ctx) {
    (void)name; (void)name_len; (void)value; (void)value_len; (void)important;
    (*(size_t *)ctx)++;
}
static void rules(unsigned depth, unsigned kind, const unsigned char *prelude, size_t prelude_len, const unsigned char *body, size_t body_len, void *ctx) {
    (void)depth; (void)kind; (void)prelude; (void)prelude_len; (void)body; (void)body_len;
    (*(size_t *)ctx)++;
}
static void parser_forms(void) {
    br_document *doc = br_document_new(); assert(doc);
    br_node *root = br_document_root(doc), *script = NULL;
    br_parser *parser = br_parser_new(root, false); assert(parser);
    const char *source = "<main><table><form id=f><tr><td><input id=i></table></form></main>";
    size_t offset = 0;
    assert(br_parser_resume(parser, (const unsigned char *)source, strlen(source), &offset, &script) == 0);
    br_parser_free(parser);
    br_node *form = query(root, "#f"), *input = query(root, "#i"), *table = query(root, "table");
    assert(br_node_parser_form(input) == form);
    assert(br_node_insert(query(root, "main"), table, NULL) == 0);
    assert(br_node_parser_form(input) == form);
    br_node *clone = br_node_clone(table, true); assert(clone);
    assert(br_node_parser_form(query(clone, "input")) == NULL);
    assert(br_node_set_attribute_ns(input, (const unsigned char *)"urn:test", 8, (const unsigned char *)"form", 4, (const unsigned char *)"f", 1) == 0);
    assert(br_node_parser_form(input) == form);
    br_node_detach(form);
    assert(br_node_parser_form(input) == NULL);
    assert(br_node_insert(table, form, NULL) == 0);
    assert(br_node_parser_form(input) == NULL);
    br_document_free(doc);
}

static bool charset_utf8(const unsigned char *label, size_t length, void *context) {
    bool accepted = length == 5 && !memcmp(label, "utf-8", 5);
    if (accepted) *(bool *)context = true;
    return accepted;
}

static void test_charset_scan(void) {
    const char *inputs[] = {
        "", "text", "<meta charset=", "<meta charset='utf-8", "<!--<meta charset=utf-8>",
        "<script>\"<meta charset=bad>\"</script><meta charset=\"utf&#45;8\">",
        "<meta charset=bad><meta http-equiv=content-type content='charset=utf-8'>",
        "<title><meta charset=bad></title><noscript><meta charset=utf-8></noscript>",
        "<meta charset=utf-8 charset=bad>",
    };
    for (size_t i = 0; i < sizeof(inputs)/sizeof(inputs[0]); i++) {
        bool found = false;
        size_t length = strlen(inputs[i]);
        assert(br_html_meta_charset((const unsigned char *)inputs[i], length, charset_utf8, &found) == 0);
        assert(found == (i >= 5));
        /* Every truncation exercises suspended states and EOF cleanup. */
        for (size_t n = 0; n < length; n++) {
            found = false;
            assert(br_html_meta_charset((const unsigned char *)inputs[i], n, charset_utf8, &found) == 0);
        }
    }
    unsigned char large[6000];
    memset(large, ' ', sizeof(large));
    memcpy(large, "<head>", 6);
    memcpy(large + 5000, "<meta charset=utf-8>", 20);
    bool found = false;
    assert(br_html_meta_charset(large, sizeof(large), charset_utf8, &found) == 0 && found);
    memcpy(large, "<body>", 6); found = false;
    assert(br_html_meta_charset(large, sizeof(large), charset_utf8, &found) == 0 && !found);
    for (size_t i = 0; i < sizeof(large); i++) large[i] = (unsigned char)(i * 137);
    assert(br_html_meta_charset(large, sizeof(large), charset_utf8, &found) == 0);
}

int main(void) {
    test_charset_scan();
    parser_forms();
    {
        br_document *doc = br_document_new(); assert(doc);
        const unsigned char *html = (const unsigned char *)"http://www.w3.org/1999/xhtml";
        br_node *input = br_node_create(doc, 1, (const unsigned char *)"input", 5,
            html, strlen((const char *)html), (const unsigned char *)"", 0);
        assert(input);
        const unsigned char value[] = {0xed, 0xa0, 0x80, 0, 0xed, 0xb0, 0x80};
        assert(br_node_set_attribute(input, (const unsigned char *)"name", 4, value, sizeof(value)) == 0);
        br_node *copy = br_node_clone(input, true); assert(copy);
        br_bytes bytes = {0};
        assert(br_node_attribute(copy, (const unsigned char *)"name", 4, &bytes) == 0);
        assert(bytes.len == sizeof(value) && !memcmp(bytes.data, value, sizeof(value)));
        br_bytes_free(&bytes);
        const unsigned char expected[] = "<input name=\"\xed\xa0\x80\0\xed\xb0\x80\">";
        assert(br_node_serialize(copy, false, &bytes) == 0);
        assert(bytes.len == sizeof(expected) - 1 && !memcmp(bytes.data, expected, sizeof(expected) - 1));
        br_bytes_free(&bytes);
        br_document_free(doc);
    }
    for (int iteration = 0; iteration < 100; iteration++) {
        const char *source = "<!doctype html><main><p id='first'>before</p>"
            "<!-- <script>bad</script> --><script data-x='>'>let a = '<div>'; /* > */</script>"
            "<p id='later'>after</p><template><script>inert</script></template>"
            "<script>second</script><p id='end'>end</p></main>";
        br_document *doc = br_document_new(); assert(doc);
        br_parser *parser = br_parser_new(br_document_root(doc), true); assert(parser);
        br_node *root = br_document_root(doc), *script = NULL;
        size_t offset = 0;
        assert(br_parser_resume(parser, (const unsigned char *)source, strlen(source), &offset, &script) == 1);
        text_is(script, "let a = '<div>'; /* > */");
        assert(query(root, "#first")); assert(!query(root, "#later"));
        br_node *main = query(root, "main");
        br_node *detached = query(root, "#first"); br_node_detach(detached);
        text_is(detached, "before");
        assert(br_node_set_text(detached, (const unsigned char *)"detached", 8) == 0);
        const char *markup = "<b class='added'>added &amp; text</b>";
        br_node *fragment = br_fragment(doc, main, (const unsigned char *)markup, strlen(markup)); assert(fragment);
        br_node *added = br_node_first(fragment); assert(added);
        assert(fragment->owner_document == root->owner_document);
        assert(added->owner_document == root->owner_document);
        assert(br_node_insert(main, added, NULL) == 0);
        text_is(query(root, "b.added"), "added & text");
        assert(br_parser_resume(parser, (const unsigned char *)source, strlen(source), &offset, &script) == 1);
        text_is(script, "second"); assert(query(root, "#later")); assert(!query(root, "#end"));
        assert(br_parser_resume(parser, (const unsigned char *)source, strlen(source), &offset, &script) == 0);
        assert(query(root, "#end")); text_is(detached, "detached");
        assert(query(main, ":scope > p:has(+ template)"));
        assert(br_node_insert(added, main, NULL) == -1);
        br_node *copy = br_node_clone(main, true); assert(copy);
        br_bytes serialized = {0}; assert(br_node_serialize(copy, false, &serialized) == 0); assert(serialized.len > 50); br_bytes_free(&serialized);
        br_node *template_copy = query(copy, "template");
        text_is(br_node_template_content(template_copy), "inert");
        size_t count = 0;
        const char *css = "color: red; padding: 2px !important; --message: 'a;b'; invalid";
        assert(br_declarations((const unsigned char *)css, strlen(css), declarations, &count) == 0); assert(count == 3);
        const unsigned char *ns = (const unsigned char *)"http://www.w3.org/2000/svg";
        br_node *gradient = br_node_create(doc, 1, (const unsigned char *)"linearGradient", 14,
            ns, strlen((const char *)ns), (const unsigned char *)"s", 1);
        assert(gradient);
        br_bytes name = {0}; assert(br_node_name(gradient, 1, &name) == 0);
        assert(name.len == 14 && !memcmp(name.data, "linearGradient", 14)); br_bytes_free(&name);
        assert(br_node_name(gradient, 3, &name) == 0);
        assert(name.len == 1 && name.data[0] == 's'); br_bytes_free(&name);
        br_node *other = br_node_create(doc, 9, NULL, 0, NULL, 0, NULL, 0); assert(other);
        br_parser *other_parser = br_parser_new(other, false); assert(other_parser);
        const char *other_source = "<p>other</p>";
        size_t other_offset = 0;
        assert(br_parser_resume(other_parser, (const unsigned char *)other_source, strlen(other_source), &other_offset, &script) == 0);
        text_is(query(other, "p"), "other");
        br_parser_free(other_parser);
        br_parser_free(parser);
        const char *sheet = "@media screen { b {color:red} } p {color:blue}";
        count = 0; assert(br_stylesheet((const unsigned char *)sheet,strlen(sheet),rules,&count)==0);assert(count==3);
        assert(br_node_set_attribute_ns(main,(const unsigned char *)"urn:test",8,(const unsigned char *)"t:Key",5,(const unsigned char *)"value",5)==0);
        assert(br_node_remove_attribute_ns(main,(const unsigned char *)"urn:test",8,(const unsigned char *)"Key",3)==0);
        br_node_clear(root);br_parser *writer=br_writer_new(root);assert(writer);
        const char *first="<!doctype html><p id='sp", *second="lit'>hello<script>run</script><i>end</i>";
        offset=0;assert(br_writer_feed(writer,(const unsigned char *)first,strlen(first),&offset,&script)==0);
        offset=0;assert(br_writer_feed(writer,(const unsigned char *)second,strlen(second),&offset,&script)==1);text_is(script,"run");
        assert(br_writer_feed(writer,(const unsigned char *)second,strlen(second),&offset,&script)==0);
        offset=0;assert(br_parser_resume(writer,(const unsigned char *)"",0,&offset,&script)==0);assert(query(root,"#split > i"));
        br_parser_free(writer);br_document_free(doc);
    }
    {
        const char *source = "<!doctype html SYSTEM 'about:legacy-compat'>";
        br_document *doc = br_document_new(); assert(doc);
        br_node *root = br_document_root(doc), *script = NULL;
        br_parser *parser = br_parser_new(root, true); assert(parser);
        size_t offset = 0;
        assert(br_parser_resume(parser, (const unsigned char *)source, strlen(source), &offset, &script) == 0);
        assert(br_document_quirks(root) == 0);
        br_node *doctype = br_node_first(root), *copy = br_node_clone(doctype, true);
        assert(copy && br_node_type(copy) == 10);
        br_bytes value = {0};
        assert(br_node_name(copy, 4, &value) == 0 && value.len == 0);
        assert(br_node_name(copy, 5, &value) == 0 && value.len == strlen("about:legacy-compat"));
        assert(!memcmp(value.data, "about:legacy-compat", value.len));
        br_bytes_free(&value);
        assert(br_document_quirks(doctype) == -1);
        assert(br_document_set_no_quirks(doctype) == -1);
        assert(br_document_set_no_quirks(root) == 0);
        assert(br_document_copy_mode(doctype, root) == -1);
        assert(br_node_name(root, 4, &value) == -1);
        br_node *cloned_doc = br_node_clone(root, true); assert(cloned_doc);
        assert(br_document_copy_mode(root, cloned_doc) == 0);
        br_parser_free(parser); br_document_free(doc);
    }
    puts("Lexbor script boundary, nested fragments, detached nodes, doctypes, modes, selectors, declarations, and teardown passed");
}
