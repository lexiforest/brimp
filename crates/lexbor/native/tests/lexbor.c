#include "lexbor.h"
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
int main(void) {
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
    puts("Lexbor script boundary, nested fragments, detached nodes, selectors, declarations, and teardown passed");
}
