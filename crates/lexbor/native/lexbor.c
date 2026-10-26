#include "lexbor.h"
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "lexbor/html/html.h"
#include "lexbor/html/tree/insertion_mode.h"
#include "lexbor/html/interfaces/template_element.h"
#include "lexbor/dom/dom.h"
#include "lexbor/css/css.h"
#include "lexbor/css/declaration.h"
#include "lexbor/selectors/selectors.h"

/* Pinned Lexbor entry point also used by html/interfaces/document.c. Keeping
 * this declaration in C avoids exposing any upstream layout to Rust. */
extern lxb_status_t lxb_html_parse_chunk_prepare(lxb_html_parser_t *, lxb_html_document_t *);
extern lxb_status_t lxb_dom_element_qualified_name_set(lxb_dom_element_t *,
    const lxb_char_t *, size_t, const lxb_char_t *, size_t);

struct br_document { lxb_html_document_t *html; };
struct br_parser {
    lxb_html_parser_t *html;
    lxb_html_tokenizer_token_f callback;
    void *callback_ctx;
    br_node *script;
    bool done, mutation_failed;
    br_mutation *mutations;
    size_t mutation_count, mutation_capacity;
};

static lxb_status_t parser_mutation(br_node *node, br_node *parent, bool removed) {
    br_parser *parser = node->owner_document->user;
    if (!parser || !parent) return LXB_STATUS_OK;
    if (parser->mutation_count == parser->mutation_capacity) {
        size_t capacity = parser->mutation_capacity ? parser->mutation_capacity * 2 : 64;
        br_mutation *items = realloc(parser->mutations, capacity * sizeof(*items));
        if (!items) { parser->mutation_failed = true; return LXB_STATUS_ERROR_MEMORY_ALLOCATION; }
        parser->mutations = items; parser->mutation_capacity = capacity;
    }
    parser->mutations[parser->mutation_count++] = (br_mutation){parent,node,node->prev,node->next,removed};
    return LXB_STATUS_OK;
}
/* Node.user is reserved by this shim for a parser-assigned form owner. Both
 * nodes belong to the same arena, which retains detached nodes until teardown. */
static bool form_control(br_node *node) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT || node->ns != LXB_NS_HTML) return false;
    switch (node->local_name) {
        case LXB_TAG_INPUT: case LXB_TAG_BUTTON: case LXB_TAG_SELECT:
        case LXB_TAG_TEXTAREA: case LXB_TAG_OUTPUT: case LXB_TAG_OBJECT:
        case LXB_TAG_FIELDSET: return true;
        default: return false;
    }
}
static br_node *tree_root(br_node *node) {
    while (node->parent) node = node->parent;
    return node;
}
static lxb_status_t parser_inserted(br_node *node) {
    br_parser *parser = node->owner_document->user;
    lxb_html_tree_t *tree = parser ? parser->html->tree : NULL;
    if (tree && tree->form && !tree->fragment && !tree->template_insertion_modes->length && form_control(node)
        && !lxb_dom_element_attr_by_name(lxb_dom_interface_element(node), (const unsigned char *)"form", 4)
        && tree_root(node) == tree_root(lxb_dom_interface_node(tree->form)))
        node->user = lxb_dom_interface_node(tree->form);
    return parser_mutation(node,node->parent,false);
}
static lxb_status_t parser_removed(br_node *node, br_node *parent) { return parser_mutation(node,parent,true); }
static lxb_status_t parser_character_data(br_node *node, const unsigned char *old_data, size_t length) {
    lxb_status_t status = parser_mutation(node,node,false);
    if (status != LXB_STATUS_OK) return status;
    br_parser *parser = node->owner_document->user;
    br_mutation *record = &parser->mutations[parser->mutation_count - 1];
    record->character_data = true;
    unsigned char *copy = malloc(length ? length : 1);
    if (!copy) { parser->mutation_failed = true; return LXB_STATUS_ERROR_MEMORY_ALLOCATION; }
    if (length) memcpy(copy,old_data,length);
    record->old_data = copy; record->old_length = length;
    return LXB_STATUS_OK;
}
static const lxb_dom_document_mutation_cb_t parser_callbacks = {.inserted=parser_inserted,.removed=parser_removed,.character_data=parser_character_data};
size_t br_parser_mutation_count(br_parser *parser) { return parser->mutation_count; }
bool br_parser_mutation_at(br_parser *parser, size_t index, br_mutation *out) {
    if (index >= parser->mutation_count) return false;
    *out = parser->mutations[index]; return true;
}
void br_parser_mutations_clear(br_parser *parser) { for (size_t i=0;i<parser->mutation_count;i++) free((void *)parser->mutations[i].old_data); parser->mutation_count = 0; }

static lxb_status_t append_bytes(const unsigned char *data, size_t len, void *ctx) {
    br_bytes *out = ctx;
    if (len > SIZE_MAX - out->len) return LXB_STATUS_ERROR_OVERFLOW;
    size_t needed = out->len + len;
    if (needed > out->capacity) {
        size_t capacity = needed > SIZE_MAX / 2 ? needed : needed * 2;
        unsigned char *next = realloc(out->data, capacity);
        if (!next) return LXB_STATUS_ERROR_MEMORY_ALLOCATION;
        out->data = next; out->capacity = capacity;
    }
    if (len) memcpy(out->data + out->len, data, len);
    out->len = needed;
    return LXB_STATUS_OK;
}
void br_bytes_free(br_bytes *bytes) { free(bytes->data); memset(bytes, 0, sizeof(*bytes)); }

br_document *br_document_new(void) {
    br_document *doc = calloc(1, sizeof(*doc));
    if (!doc) return NULL;
    doc->html = lxb_html_document_create();
    if (!doc->html) { free(doc); return NULL; }
    return doc;
}
void br_document_free(br_document *doc) {
    if (doc) { lxb_html_document_destroy(doc->html); free(doc); }
}
br_node *br_document_root(br_document *doc) { return lxb_dom_interface_node(doc->html); }

static lxb_html_token_t *token_done(lxb_html_tokenizer_t *tkz, lxb_html_token_t *token, void *ctx) {
    br_parser *parser = ctx;
    lxb_html_tree_t *tree = parser->html->tree;
    br_node *node = lxb_html_tree_current_node(tree);
    bool script = node && node->ns == LXB_NS_HTML && node->local_name == LXB_TAG_SCRIPT
        && token->tag_id == LXB_TAG_SCRIPT && (token->type & LXB_HTML_TOKEN_TYPE_CLOSE)
        && tree->mode == lxb_html_tree_insertion_mode_text
        && tree->template_insertion_modes->length == 0;
    lxb_html_token_t *result = parser->callback(tkz, token, parser->callback_ctx);
    if (result && script) parser->script = node;
    return result;
}
br_parser *br_parser_new(br_node *root, bool scripting) {
    if (root->type != LXB_DOM_NODE_TYPE_DOCUMENT) return NULL;
    lxb_html_document_t *document = lxb_html_interface_document(root);
    br_parser *parser = calloc(1, sizeof(*parser));
    if (!parser) return NULL;
    parser->html = lxb_html_parser_create();
    if (!parser->html || lxb_html_parser_init(parser->html) != LXB_STATUS_OK) goto failed;
    lxb_html_parser_scripting_set(parser->html, scripting);
    document->dom_document.scripting = scripting;
    document->dom_document.compat_mode = LXB_DOM_DOCUMENT_CMODE_NO_QUIRKS;
    if (lxb_html_parse_chunk_prepare(parser->html, document) != LXB_STATUS_OK) goto failed;
    parser->callback = parser->html->tkz->callback_token_done;
    parser->callback_ctx = parser->html->tkz->callback_token_ctx;
    lxb_html_tokenizer_callback_token_done_set(parser->html->tkz, token_done, parser);
    return parser;
failed:
    br_parser_free(parser); return NULL;
}
void br_parser_free(br_parser *parser) {
    if (parser) {
        if (parser->html) lxb_html_parser_destroy(parser->html);
        br_parser_mutations_clear(parser);
        free(parser->mutations);
        free(parser);
    }
}
static int parser_resume(br_parser *parser, const unsigned char *data, size_t len, size_t *offset, br_node **script) {
    *script = NULL;
    if (parser->done) return 0;
    parser->script = NULL;
    while (*offset < len) {
        const unsigned char *start = data + *offset;
        const unsigned char *end = memchr(start, '>', len - *offset);
        size_t size = end ? (size_t)(end + 1 - start) : len - *offset;
        if (lxb_html_parse_chunk_process(parser->html, start, size) != LXB_STATUS_OK) return -1;
        *offset += size;
        if (parser->script) { *script = parser->script; return 1; }
    }
    parser->done = true;
    return lxb_html_parse_chunk_end(parser->html) == LXB_STATUS_OK ? 0 : -1;
}
int br_parser_resume(br_parser *parser, const unsigned char *data, size_t len, size_t *offset, br_node **script) {
    lxb_dom_document_t *doc = lxb_dom_document_owner(&parser->html->tree->document->dom_document);
    const lxb_dom_document_mutation_cb_t *previous = doc->mutation;
    void *user = doc->user;
    doc->mutation = &parser_callbacks; doc->user = parser;
    int result = parser_resume(parser,data,len,offset,script);
    doc->mutation = previous; doc->user = user;
    return parser->mutation_failed ? -1 : result;
}
br_node *br_fragment(br_document *doc, br_node *context, const unsigned char *input, size_t len) {
    if (context->type != LXB_DOM_NODE_TYPE_ELEMENT) return NULL;
    lxb_html_parser_t *parser = lxb_html_parser_create();
    if (!parser) return NULL;
    if (lxb_html_parser_init(parser) != LXB_STATUS_OK) { lxb_html_parser_destroy(parser); return NULL; }
    lxb_html_parser_scripting_set(parser, true);
    /* A fresh parser has no document. Pass our arena explicitly so fragment
     * nodes and interned names live until br_document_free(), without leaking
     * a separate document for every fragment. */
    br_node *root = lxb_html_parse_fragment_by_tag_id(parser, doc->html,
        context->local_name, context->ns, input, len);
    lxb_html_parser_destroy(parser);
    return root;
}
unsigned br_node_type(br_node *node) { return node->type; }
int br_document_quirks(br_node *node) {
    if (node->type != LXB_DOM_NODE_TYPE_DOCUMENT) return -1;
    return lxb_dom_interface_document(node)->compat_mode == LXB_DOM_DOCUMENT_CMODE_QUIRKS;
}
int br_document_set_no_quirks(br_node *node) {
    if (node->type != LXB_DOM_NODE_TYPE_DOCUMENT) return -1;
    lxb_dom_interface_document(node)->compat_mode = LXB_DOM_DOCUMENT_CMODE_NO_QUIRKS;
    return 0;
}
int br_document_copy_mode(br_node *source, br_node *target) {
    if (source->type != LXB_DOM_NODE_TYPE_DOCUMENT || target->type != LXB_DOM_NODE_TYPE_DOCUMENT) return -1;
    lxb_dom_interface_document(target)->compat_mode = lxb_dom_interface_document(source)->compat_mode;
    return 0;
}
br_node *br_node_parent(br_node *node) { return node->parent; }
br_node *br_node_first(br_node *node) { return node->first_child; }
br_node *br_node_next(br_node *node) { return node->next; }
br_node *br_node_template_content(br_node *node) {
    if (node->ns == LXB_NS_HTML && node->local_name == LXB_TAG_TEMPLATE)
        return lxb_dom_interface_node(lxb_html_interface_template(node)->content);
    return NULL;
}
int br_node_name(br_node *node, unsigned part, br_bytes *out) {
    size_t len = 0; const unsigned char *name = NULL;
    if (part == 0) name = lxb_dom_node_name(node, &len);
    else if (part == 1 && node->type == LXB_DOM_NODE_TYPE_ELEMENT) {
        name = lxb_dom_element_qualified_name(lxb_dom_interface_element(node), &len);
        if (node->prefix && name) {
            const unsigned char *colon = memchr(name, ':', len);
            if (colon) { len -= (size_t)(colon + 1 - name); name = colon + 1; }
        }
    }
    else if (part == 2) { const lxb_ns_data_t *ns = lxb_ns_data_by_id(node->owner_document->ns, node->ns); if (ns) name = lexbor_hash_entry_str(&ns->entry), len = ns->entry.length; }
    else if (part == 3 && node->type == LXB_DOM_NODE_TYPE_ELEMENT && node->prefix) {
        /* v3.0.0's prefix accessor does not write its length. Read the canonical
         * qualified name instead, without exposing an uninitialized length. */
        name = lxb_dom_element_qualified_name(lxb_dom_interface_element(node), &len);
        const unsigned char *colon = name ? memchr(name, ':', len) : NULL;
        len = colon ? (size_t)(colon - name) : 0;
    }
    else if (part == 4 || part == 5) {
        if (node->type != LXB_DOM_NODE_TYPE_DOCUMENT_TYPE) return -1;
        lxb_dom_document_type_t *doctype = lxb_dom_interface_document_type(node);
        name = part == 4 ? lxb_dom_document_type_public_id(doctype, &len)
                         : lxb_dom_document_type_system_id(doctype, &len);
    }
    return append_bytes(name, len, out) == LXB_STATUS_OK ? 0 : -1;
}
int br_node_text(br_node *node, br_bytes *out) {
    size_t len = 0; unsigned char *text = lxb_dom_node_text_content(node, &len);
    int status = append_bytes(text, len, out) == LXB_STATUS_OK ? 0 : -1;
    if (text) lexbor_mraw_free(node->owner_document->text, text);
    return status;
}
br_node *br_node_parser_form(br_node *node) { return form_control(node) ? node->user : NULL; }
static bool contains_node(br_node *parent, br_node *node) {
    for (; node; node = node->parent) if (node == parent) return true;
    return false;
}
static br_node *subtree_next(br_node *node, br_node *root) {
    if (node->first_child) return node->first_child;
    while (node != root && !node->next) node = node->parent;
    return node == root ? NULL : node->next;
}
void br_node_detach(br_node *node) {
    if (!node->parent) return;
    bool relevant = false;
    for (br_node *item = node; item; item = subtree_next(item, node)) {
        if (br_node_parser_form(item) || (item->ns == LXB_NS_HTML && item->local_name == LXB_TAG_FORM)) {
            relevant = true; break;
        }
    }
    if (relevant) {
        br_node *root = tree_root(node);
        for (br_node *item = root; item; item = subtree_next(item, root)) {
            br_node *owner = br_node_parser_form(item);
            if (owner && contains_node(node, item) != contains_node(node, owner)) item->user = NULL;
        }
    }
    lxb_dom_node_remove(node);
}
void br_node_clear(br_node *node) { while (node->first_child) br_node_detach(node->first_child); }
int br_node_set_text(br_node *node, const unsigned char *text, size_t len) {
    if (node->type == LXB_DOM_NODE_TYPE_TEXT || node->type == LXB_DOM_NODE_TYPE_COMMENT) {
        lxb_dom_character_data_t *data = lxb_dom_interface_character_data(node);
        return lxb_dom_character_data_replace(data, text, len, 0, data->data.length) == LXB_STATUS_OK ? 0 : -1;
    }
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT && node->type != LXB_DOM_NODE_TYPE_DOCUMENT_FRAGMENT) return -1;
    br_node *child = len ? lxb_dom_interface_node(lxb_dom_document_create_text_node(node->owner_document, text, len)) : NULL;
    if (len && !child) return -1;
    br_node_clear(node);
    if (child) lxb_dom_node_insert_child(node, child);
    return 0;
}
int br_node_serialize(br_node *node, bool inner, br_bytes *out) {
    lxb_status_t status = inner ? lxb_html_serialize_deep_cb(node, append_bytes, out) : lxb_html_serialize_tree_cb(node, append_bytes, out);
    return status == LXB_STATUS_OK ? 0 : -1;
}
br_node *br_node_create(br_document *doc, unsigned kind, const unsigned char *name, size_t len, const unsigned char *ns, size_t ns_len, const unsigned char *prefix, size_t prefix_len) {
    lxb_dom_document_t *owner = &doc->html->dom_document;
    switch (kind) {
        case LXB_DOM_NODE_TYPE_ELEMENT: {
            lxb_dom_element_t *element = lxb_dom_element_create(owner, name, len,
                ns_len ? ns : NULL, ns_len, prefix_len ? prefix : NULL, prefix_len, NULL, 0, false);
            if (!element) return NULL;
            if (lxb_dom_element_qualified_name_set(element, prefix, prefix_len, name, len) != LXB_STATUS_OK)
                return NULL;
            return lxb_dom_interface_node(element);
        }
        case LXB_DOM_NODE_TYPE_TEXT: return lxb_dom_interface_node(lxb_dom_document_create_text_node(owner, name, len));
        case LXB_DOM_NODE_TYPE_COMMENT: return lxb_dom_interface_node(lxb_dom_document_create_comment(owner, name, len));
        case LXB_DOM_NODE_TYPE_DOCUMENT_FRAGMENT: return lxb_dom_interface_node(lxb_dom_document_create_document_fragment(owner));
        case LXB_DOM_NODE_TYPE_DOCUMENT: {
            lxb_html_document_t *created = lxb_html_document_interface_create(doc->html);
            if (!created) return NULL;
            /* Memory ownership does not imply inheriting the owner's mode. */
            created->dom_document.compat_mode = LXB_DOM_DOCUMENT_CMODE_NO_QUIRKS;
            return lxb_dom_interface_node(created);
        }
        default: return NULL;
    }
}
br_node *br_node_clone(br_node *node, bool deep) {
    br_node *copy = lxb_dom_node_clone(node, deep);
    if (copy) copy->user = NULL;
    if (!copy || !deep) return copy;
    /* Lexbor clones ordinary descendants. HTML template contents live outside
     * child links, so copy those explicitly, including nested templates. */
    struct pair { br_node *source, *copy; };
    lexbor_array_obj_t *pending = lexbor_array_obj_create();
    if (!pending || lexbor_array_obj_init(pending, 16, sizeof(struct pair)) != LXB_STATUS_OK) {
        if (pending) lexbor_array_obj_destroy(pending, true);
        return NULL;
    }
    struct pair *pair = lexbor_array_obj_push(pending);
    if (!pair) goto failed;
    *pair = (struct pair){node, copy};
    while (pending->length) {
        struct pair next = *(struct pair *)lexbor_array_obj_pop(pending);
        next.copy->user = NULL;
        br_node *content = br_node_template_content(next.source);
        if (content) {
            br_node *target = br_node_template_content(next.copy);
            if (!target) goto failed;
            for (br_node *child = content->first_child; child; child = child->next) {
                br_node *cloned = lxb_dom_node_clone(child, true);
                if (!cloned) goto failed;
                lxb_dom_node_insert_child(target, cloned);
                pair = lexbor_array_obj_push(pending);
                if (!pair) goto failed;
                *pair = (struct pair){child, cloned};
            }
        }
        br_node *child = next.source->first_child, *cloned = next.copy->first_child;
        for (; child && cloned; child = child->next, cloned = cloned->next) {
            pair = lexbor_array_obj_push(pending);
            if (!pair) goto failed;
            *pair = (struct pair){child, cloned};
        }
    }
    lexbor_array_obj_destroy(pending, true);
    return copy;
failed:
    lexbor_array_obj_destroy(pending, true);
    return NULL;
}
int br_node_insert(br_node *parent, br_node *child, br_node *before) {
    if (before && before->parent != parent) return -1;
    if (child == before) return 0;
    if (parent->type != LXB_DOM_NODE_TYPE_DOCUMENT && parent->type != LXB_DOM_NODE_TYPE_ELEMENT && parent->type != LXB_DOM_NODE_TYPE_DOCUMENT_FRAGMENT) return -1;
    for (br_node *node = parent; node; node = node->parent) if (node == child) return -1;
    if (child->type == LXB_DOM_NODE_TYPE_DOCUMENT) return -1;
    if (child->type == LXB_DOM_NODE_TYPE_DOCUMENT_FRAGMENT) {
        while (child->first_child) {
            br_node *next = child->first_child; br_node_detach(next);
            if (before) lxb_dom_node_insert_before(before, next); else lxb_dom_node_insert_child(parent, next);
        }
    } else {
        br_node_detach(child);
        if (before) lxb_dom_node_insert_before(before, child); else lxb_dom_node_insert_child(parent, child);
    }
    return 0;
}
int br_node_attribute(br_node *node, const unsigned char *name, size_t len, br_bytes *out) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) return -1;
    lxb_dom_attr_t *attr = lxb_dom_element_attr_by_name(lxb_dom_interface_element(node), name, len);
    if (!attr) return 1;
    size_t value_len = 0;
    const unsigned char *value = lxb_dom_attr_value(attr, &value_len);
    return append_bytes(value, value_len, out) == LXB_STATUS_OK ? 0 : -1;
}
static bool form_attribute(lxb_dom_attr_t *attr) {
    if (!attr || attr->node.ns != LXB_NS__UNDEF) return false;
    size_t len = 0;
    const unsigned char *name = lxb_dom_attr_local_name(attr, &len);
    return len == 4 && !memcmp(name, "form", 4);
}
int br_node_set_attribute(br_node *node, const unsigned char *name, size_t len, const unsigned char *value, size_t value_len) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) return -1;
    lxb_dom_attr_t *attr = lxb_dom_element_set_attribute(lxb_dom_interface_element(node), name, len, value, value_len);
    if (!attr) return -1;
    if (form_attribute(attr)) node->user = NULL;
    return 0;
}
int br_node_remove_attribute(br_node *node, const unsigned char *name, size_t len) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) return -1;
    lxb_dom_attr_t *attr = lxb_dom_element_attr_by_name(lxb_dom_interface_element(node), name, len);
    bool reset_form = form_attribute(attr);
    if (lxb_dom_element_remove_attribute(lxb_dom_interface_element(node), name, len) != LXB_STATUS_OK) return -1;
    if (reset_form) node->user = NULL;
    return 0;
}

extern lxb_status_t lxb_dom_attr_set_name_ns(lxb_dom_attr_t *, const lxb_char_t *, size_t, const lxb_char_t *, size_t, bool);
static lxb_dom_attr_t *attribute_ns(br_node *node, const unsigned char *ns, size_t ns_len, const unsigned char *local, size_t local_len) {
    for (lxb_dom_attr_t *a = lxb_dom_interface_element(node)->first_attr; a; a = a->next) {
        size_t len; const unsigned char *name = lxb_dom_attr_qualified_name(a, &len);
        const unsigned char *colon = a->node.prefix ? memchr(name, ':', len) : NULL;
        if (colon) { len -= (size_t)(colon + 1 - name); name = colon + 1; }
        if (len != local_len || memcmp(name, local, len)) continue;
        if (a->node.ns == LXB_NS__UNDEF) { if (!ns_len) return a; continue; }
        const lxb_ns_data_t *data = lxb_ns_data_by_id(node->owner_document->ns, a->node.ns);
        if (data && data->entry.length == ns_len && !memcmp(lexbor_hash_entry_str(&data->entry), ns, ns_len)) return a;
    }
    return NULL;
}
int br_node_set_attribute_ns(br_node *node, const unsigned char *ns, size_t ns_len, const unsigned char *name, size_t name_len, const unsigned char *value, size_t value_len) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) return -1;
    const unsigned char *colon = memchr(name, ':', name_len), *local = colon ? colon + 1 : name;
    size_t local_len = name_len - (size_t)(local - name);
    lxb_dom_attr_t *attr = attribute_ns(node, ns, ns_len, local, local_len);
    if (attr) {
        if (lxb_dom_attr_set_value(attr, value, value_len) != LXB_STATUS_OK) return -1;
        if (!ns_len && local_len == 4 && !memcmp(local, "form", 4)) node->user = NULL;
        return 0;
    }
    attr = lxb_dom_attr_interface_create(node->owner_document);
    if (!attr) return -1;
    lxb_status_t status = ns_len
        ? lxb_dom_attr_set_name_ns(attr, ns, ns_len, name, name_len, false)
        : lxb_dom_attr_set_name(attr, name, name_len, false);
    if (status == LXB_STATUS_OK) status = lxb_dom_attr_set_value(attr, value, value_len);
    if (status == LXB_STATUS_OK) status = lxb_dom_element_attr_append(lxb_dom_interface_element(node), attr);
    if (status != LXB_STATUS_OK) { lxb_dom_attr_interface_destroy(attr); return -1; }
    if (!ns_len && local_len == 4 && !memcmp(local, "form", 4)) node->user = NULL;
    return 0;
}
int br_node_remove_attribute_ns(br_node *node, const unsigned char *ns, size_t ns_len, const unsigned char *local, size_t local_len) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) return -1;
    lxb_dom_attr_t *attr = attribute_ns(node, ns, ns_len, local, local_len);
    if (!attr) return 0;
    if (!ns_len && local_len == 4 && !memcmp(local, "form", 4)) node->user = NULL;
    lxb_dom_element_attr_remove(lxb_dom_interface_element(node), attr);
    lxb_dom_attr_interface_destroy(attr);
    return 0;
}
void br_node_attributes(br_node *node, br_attr_cb cb, void *ctx) {
    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) return;
    for (lxb_dom_attr_t *attr = lxb_dom_interface_element(node)->first_attr; attr; attr = attr->next) {
        size_t name_len, value_len;
        const unsigned char *name = lxb_dom_attr_qualified_name(attr, &name_len);
        const unsigned char *value = lxb_dom_attr_value(attr, &value_len);
        size_t local_len = 0, ns_len = 0, prefix_len = 0;
        const unsigned char *local = lxb_dom_attr_local_name(attr, &local_len);
        const lxb_ns_data_t *ns = lxb_ns_data_by_id(node->owner_document->ns, attr->node.ns);
        const unsigned char *ns_name = NULL;
        if (ns && attr->node.ns != LXB_NS__UNDEF) {
            ns_name = lexbor_hash_entry_str(&ns->entry); ns_len = ns->entry.length;
        }
        const unsigned char *prefix = NULL;
        if (attr->node.prefix) {
            const lxb_ns_prefix_data_t *data = lxb_ns_prefix_data_by_id(node->owner_document->prefix, attr->node.prefix);
            if (data) { prefix = lexbor_hash_entry_str(&data->entry); prefix_len = data->entry.length; }
        }
        const unsigned char *colon = attr->node.prefix ? memchr(name, ':', name_len) : NULL;
        if (colon) { local = colon + 1; local_len = name_len - (size_t)(local - name); }
        else if (!attr->node.prefix) { local = name; local_len = name_len; }
        cb(name, name_len, local, local_len, ns_name, ns_len, prefix, prefix_len, value, value_len, ctx);
    }
}
struct query_context { br_node_cb callback; void *context; };
static lxb_status_t selected(br_node *node, lxb_css_selector_specificity_t specificity, void *ctx) {
    (void)specificity;
    struct query_context *query = ctx; query->callback(node, query->context); return LXB_STATUS_OK;
}
int br_select(br_node *root, const unsigned char *selector, size_t len, bool matches, br_node_cb cb, void *ctx) {
    int result = -1;
    lxb_css_parser_t *parser = lxb_css_parser_create();
    lxb_selectors_t *selectors = lxb_selectors_create();
    if (!parser || !selectors) goto done;
    if (lxb_css_parser_init(parser, NULL) != LXB_STATUS_OK || lxb_selectors_init(selectors) != LXB_STATUS_OK) goto done;
    lxb_css_selector_list_t *list = lxb_css_selectors_parse(parser, selector, len);
    if (!list || parser->status != LXB_STATUS_OK) goto done;
    lxb_selectors_opt_set(selectors, LXB_SELECTORS_OPT_MATCH_FIRST);
    selectors->scope = root;
    if (root->type == LXB_DOM_NODE_TYPE_DOCUMENT) {
        for (br_node *child = root->first_child; child; child = child->next)
            if (child->type == LXB_DOM_NODE_TYPE_ELEMENT) { selectors->scope = child; break; }
    }
    struct query_context context = {cb, ctx};
    lxb_status_t status = matches ? lxb_selectors_match_node(selectors, root, list, selected, &context) : lxb_selectors_find(selectors, root, list, selected, &context);
    result = status == LXB_STATUS_OK ? 0 : -1;
done:
    if (parser) { if (parser->memory) lxb_css_memory_destroy(parser->memory, true); lxb_css_parser_destroy(parser, true); }
    if (selectors) lxb_selectors_destroy(selectors, true);
    return result;
}
int br_declarations(const unsigned char *css, size_t len, br_decl_cb callback, void *ctx) {
    int result = -1;
    lxb_css_parser_t *parser = lxb_css_parser_create();
    lxb_css_memory_t *memory = lxb_css_memory_create();
    if (!parser || !memory) goto done;
    if (lxb_css_parser_init(parser, NULL) != LXB_STATUS_OK || lxb_css_memory_init(memory, 64) != LXB_STATUS_OK) goto done;
    lxb_css_parser_memory_set(parser, memory);
    lxb_css_rule_declaration_list_t *list = lxb_css_declaration_list_parse(parser, css, len);
    if (!list) goto done;
    for (lxb_css_rule_t *rule = list->first; rule; rule = rule->next) {
        if (rule->type != LXB_CSS_RULE_DECLARATION) continue;
        lxb_css_rule_declaration_t *declaration = lxb_css_rule_declaration(rule);
        if (declaration->type == LXB_CSS_PROPERTY__UNDEF) continue;
        br_bytes name = {0}, value = {0};
        lxb_status_t status = lxb_css_rule_declaration_serialize_name(declaration, append_bytes, &name);
        if (status == LXB_STATUS_OK) status = lxb_css_property_serialize(declaration->u.user, declaration->type, append_bytes, &value);
        bool known = declaration->type != LXB_CSS_PROPERTY__CUSTOM ||
            (name.len >= 2 && name.data[0] == '-' && name.data[1] == '-');
        if (status == LXB_STATUS_OK && known) callback(name.data, name.len, value.data, value.len, declaration->important, ctx);
        br_bytes_free(&name); br_bytes_free(&value);
        if (status != LXB_STATUS_OK) goto done;
    }
    result = 0;
done:
    if (memory) lxb_css_memory_destroy(memory, true);
    if (parser) lxb_css_parser_destroy(parser, true);
    return result;
}

/* Copy the parser's rule tree across the safe boundary. No parser pointers or
 * partially owned rule objects escape the synchronous call. */
static int stylesheet_rules(lxb_css_rule_list_t *list, const unsigned char *css, size_t len, unsigned depth, br_rule_cb cb, void *ctx) {
    if (!list) return 0;
    for (lxb_css_rule_t *rule = list->first; rule; rule = rule->next) {
        br_bytes prelude = {0}, body = {0};
        unsigned type = 0; lxb_css_rule_list_t *children = NULL;
        lxb_status_t status = LXB_STATUS_OK;
        if (rule->type == LXB_CSS_RULE_STYLE) {
            lxb_css_rule_style_t *style = lxb_css_rule_style(rule);
            type = 1;
            status = lxb_css_selector_serialize_list_chain(style->selector, append_bytes, &prelude);
            if (status == LXB_STATUS_OK && style->declarations) status = lxb_css_rule_declaration_list_serialize(style->declarations, append_bytes, &body);
            children = style->child;
        } else if (rule->type == LXB_CSS_RULE_AT_RULE) {
            lxb_css_rule_at_t *at = lxb_css_rule_at(rule);
            br_bytes name = {0};
            status = lxb_css_rule_at_serialize_name(at, append_bytes, &name);
            if (status == LXB_STATUS_OK) {
                if (name.len == 5 && !memcmp(name.data,"media",5)) type = 4;
                else if (name.len == 8 && !memcmp(name.data,"supports",8)) type = 12;
                else if (name.len == 9 && !memcmp(name.data,"font-face",9)) type = 5;
                else if (name.len == 6 && !memcmp(name.data,"import",6)) type = 3;
                else if (name.len == 9 && !memcmp(name.data,"namespace",9)) type = 10;
            }
            br_bytes_free(&name);
            if (at->prelude_begin <= at->prelude_end && at->prelude_end <= len)
                status = append_bytes(css+at->prelude_begin,at->prelude_end-at->prelude_begin,&prelude);
            if (at->type == LXB_CSS_AT_RULE_MEDIA) children = at->u.media->block;
            else if (at->type == LXB_CSS_AT_RULE_FONT_FACE) children = at->u.font_face->block;
            else if (at->type == LXB_CSS_AT_RULE__CUSTOM) children = at->u.custom->block;
            else if (at->type == LXB_CSS_AT_RULE__UNDEF) children = at->u.undef->block;
            if (type == 5 && children) { status = lxb_css_rule_list_serialize(children, append_bytes, &body); children = NULL; }
        }
        if (status != LXB_STATUS_OK) { br_bytes_free(&prelude); br_bytes_free(&body); return -1; }
        if (type) cb(depth,type,prelude.data,prelude.len,body.data,body.len,ctx);
        br_bytes_free(&prelude); br_bytes_free(&body);
        if (type && stylesheet_rules(children,css,len,depth+1,cb,ctx)) return -1;
    }
    return 0;
}
int br_stylesheet(const unsigned char *css, size_t len, br_rule_cb cb, void *ctx) {
    int result = -1;
    lxb_css_parser_t *parser = lxb_css_parser_create();
    lxb_css_stylesheet_t *sheet = lxb_css_stylesheet_create(NULL);
    if (!parser || !sheet) goto done;
    if (lxb_css_parser_init(parser,NULL) != LXB_STATUS_OK) goto done;
    if (lxb_css_stylesheet_parse(sheet,parser,css,len) != LXB_STATUS_OK) goto done;
    if (sheet->root && sheet->root->type == LXB_CSS_RULE_LIST)
        result = stylesheet_rules(lxb_css_rule_list(sheet->root),css,len,0,cb,ctx);
    else result = 0;
done:
    if (sheet) lxb_css_stylesheet_destroy(sheet,true);
    if (parser) lxb_css_parser_destroy(parser,true);
    return result;
}

br_parser *br_writer_new(br_node *root) {
    if (root->type != LXB_DOM_NODE_TYPE_DOCUMENT || root->first_child) return NULL;
    lxb_html_document_t *doc = lxb_html_interface_document(root);
    doc->head = NULL; doc->body = NULL;
    doc->dom_document.element = NULL; doc->dom_document.doctype = NULL;
    return br_parser_new(root,true);
}
int br_writer_feed(br_parser *parser, const unsigned char *data, size_t len, size_t *offset, br_node **script) {
    lxb_dom_document_t *doc = lxb_dom_document_owner(&parser->html->tree->document->dom_document);
    const lxb_dom_document_mutation_cb_t *previous = doc->mutation; void *user = doc->user;
    doc->mutation = &parser_callbacks; doc->user = parser;
    *script = NULL; parser->script = NULL; int result = 0;
    while (*offset < len) {
        const unsigned char *start = data+*offset, *end = memchr(start,'>',len-*offset);
        size_t size = end ? (size_t)(end+1-start) : len-*offset;
        if (lxb_html_parse_chunk_process(parser->html,start,size) != LXB_STATUS_OK) { result = -1; break; }
        *offset += size;
        if (parser->script) { *script = parser->script; result = 1; break; }
    }
    doc->mutation = previous; doc->user = user;
    return parser->mutation_failed ? -1 : result;
}
