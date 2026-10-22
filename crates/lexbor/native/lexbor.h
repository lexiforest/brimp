#ifndef BRIMP_LEXBOR_H
#define BRIMP_LEXBOR_H
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct br_document br_document;
typedef struct br_parser br_parser;
typedef struct lxb_dom_node br_node;
typedef struct { unsigned char *data; size_t len; size_t capacity; } br_bytes;
typedef void (*br_node_cb)(br_node *, void *);
typedef void (*br_attr_cb)(const unsigned char *, size_t, const unsigned char *, size_t,
    const unsigned char *, size_t, const unsigned char *, size_t,
    const unsigned char *, size_t, void *);
typedef void (*br_decl_cb)(const unsigned char *, size_t, const unsigned char *, size_t, bool, void *);

br_document *br_document_new(void);
void br_document_free(br_document *);
br_node *br_document_root(br_document *);
br_parser *br_parser_new(br_node *, bool);
void br_parser_free(br_parser *);
/* 0: done; 1: script boundary; -1: parser/allocation error. Input stays live. */
int br_parser_resume(br_parser *, const unsigned char *, size_t, size_t *, br_node **);
br_node *br_fragment(br_document *, br_node *, const unsigned char *, size_t);

unsigned br_node_type(br_node *);
br_node *br_node_parent(br_node *);
br_node *br_node_first(br_node *);
br_node *br_node_next(br_node *);
br_node *br_node_template_content(br_node *);
int br_node_name(br_node *, unsigned, br_bytes *);
int br_node_text(br_node *, br_bytes *);
int br_node_set_text(br_node *, const unsigned char *, size_t);
int br_node_serialize(br_node *, bool, br_bytes *);
br_node *br_node_create(br_document *, unsigned, const unsigned char *, size_t, const unsigned char *, size_t, const unsigned char *, size_t);
br_node *br_node_clone(br_node *, bool);
void br_node_detach(br_node *);
void br_node_clear(br_node *);
/* Validates cycles and reference parent before detaching anything. */
int br_node_insert(br_node *, br_node *, br_node *);
int br_node_attribute(br_node *, const unsigned char *, size_t, br_bytes *);
int br_node_set_attribute(br_node *, const unsigned char *, size_t, const unsigned char *, size_t);
int br_node_remove_attribute(br_node *, const unsigned char *, size_t);
void br_node_attributes(br_node *, br_attr_cb, void *);
int br_select(br_node *, const unsigned char *, size_t, bool, br_node_cb, void *);
int br_declarations(const unsigned char *, size_t, br_decl_cb, void *);
void br_bytes_free(br_bytes *);
#endif

typedef struct {br_node *target, *node, *previous, *next; bool removed; bool character_data; const unsigned char *old_data; size_t old_length;} br_mutation;
size_t br_parser_mutation_count(br_parser *);
bool br_parser_mutation_at(br_parser *, size_t, br_mutation *);
void br_parser_mutations_clear(br_parser *);
