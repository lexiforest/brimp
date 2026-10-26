#include "lexbor.h"
#include "lexbor/html/html.h"
#include "lexbor/html/encoding.h"
#include <string.h>

/* Safari scans tokens, rather than raw occurrences of <meta>. The tokenizer
 * handles comments, references, duplicate attributes and raw-text elements.
 * Head declarations can be arbitrarily late; outside the head, stop after a
 * complete token at/after byte 1024. This is deliberately not Lexbor's raw
 * byte prescan, which also sees markup-looking strings inside scripts.
 * Reference: WebKit Source/WebCore/html/parser/HTMLMetaCharsetParser.cpp. */
typedef struct {
    br_charset_cb accept;
    void *context;
    const unsigned char *input;
    size_t consumed;
    bool in_head, done;
} charset_scan;

static bool attribute_is(lxb_html_token_attr_t *attr, const char *name) {
    size_t length;
    const unsigned char *data = lxb_html_token_attr_name(attr, &length);
    return length == strlen(name) && !memcmp(data, name, length);
}

static lxb_html_token_t *charset_token(lxb_html_tokenizer_t *tkz, lxb_html_token_t *token, void *context) {
    charset_scan *scan = context;
    bool closing = token->type & LXB_HTML_TOKEN_TYPE_CLOSE;
    if (token->tag_id >= LXB_TAG__BEGIN) {
        if (!closing) {
            /* Safari's charset tokenizer recognizes declarations in noscript. */
            lxb_html_tokenizer_set_state_by_tag(tkz, false, token->tag_id, LXB_NS_HTML);
            if (token->tag_id == LXB_TAG_META) {
                const unsigned char *label = NULL;
                size_t length = 0;
                bool pragma = false, direct = false;
                for (lxb_html_token_attr_t *attr = token->attr_first; attr; attr = attr->next) {
                    if (attribute_is(attr, "charset")) {
                        label = attr->value; length = attr->value_size; direct = true;
                        break;
                    }
                    if (attribute_is(attr, "http-equiv")) {
                        pragma = attr->value_size == 12 && lexbor_str_data_ncasecmp(attr->value,
                            (const unsigned char *)"content-type", 12);
                    } else if (attribute_is(attr, "content") && attr->value_size) {
                        const unsigned char *end;
                        label = lxb_html_encoding_content(attr->value, attr->value + attr->value_size, &end);
                        length = label ? (size_t)(end - label) : 0;
                    }
                }
                if (label && (direct || pragma) && scan->accept(label, length, scan->context)) {
                    scan->done = true;
                    return NULL;
                }
            }
        }
        switch (token->tag_id) {
            case LXB_TAG_SCRIPT: case LXB_TAG_NOSCRIPT: case LXB_TAG_STYLE:
            case LXB_TAG_LINK: case LXB_TAG_META: case LXB_TAG_OBJECT:
            case LXB_TAG_TITLE: case LXB_TAG_BASE:
                break;
            case LXB_TAG_HTML: case LXB_TAG_HEAD:
                if (!closing) break;
                /* fall through */
            default: scan->in_head = false;
        }
    }
    size_t consumed = scan->consumed;
    /* Text is emitted upon seeing the next tag; its end excludes that tag. */
    if (token->tag_id == LXB_TAG__TEXT && token->end) consumed = (size_t)(token->end - scan->input);
    if (!scan->in_head && consumed >= 1024) { scan->done = true; return NULL; }
    return token;
}

int br_html_meta_charset(const unsigned char *input, size_t length, br_charset_cb accept, void *context) {
    lxb_html_tokenizer_t *tkz = lxb_html_tokenizer_create();
    if (!tkz) return -1;
    charset_scan scan = {.accept=accept, .context=context, .input=input, .in_head=true};
    lxb_status_t status = lxb_html_tokenizer_init(tkz);
    if (status == LXB_STATUS_OK) {
        lxb_html_tokenizer_callback_token_done_set(tkz, charset_token, &scan);
        status = lxb_html_tokenizer_begin(tkz);
    }
    /* Single-byte chunks expose the consumed position without relying on
     * tokenizer internals or mistaking a tag-name end for the token end. */
    while (status == LXB_STATUS_OK && !scan.done && scan.consumed < length) {
        size_t offset = scan.consumed++;
        status = lxb_html_tokenizer_chunk(tkz, input + offset, 1);
    }
    if (status == LXB_STATUS_OK && !scan.done) status = lxb_html_tokenizer_end(tkz);
    lxb_html_tokenizer_destroy(tkz);
    return status == LXB_STATUS_OK || scan.done ? 0 : -1;
}
