/*
 * Copyright (C) 2023 The Chromium Authors.
 * Copyright (C) 2023 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *    * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *    * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *    * Neither the name of Google LLC nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Safari's DOMParser keeps standards mode when WebKit's restricted fragment
// parser accepts the input. Recognize that input language without building a
// second DOM. Lexbor still constructs every node and decodes character references.
// Rules: WebKit HTMLDocumentParserFastPath.cpp; measured against Safari 27.0.
function __safariDOMParserStandardsMode(source) {
  if (source.includes('\0')) return false;
  const phrasing = new Set([
    'b',
    'br',
    'button',
    'i',
    'input',
    'label',
    'select',
    'span',
    'strong',
  ]);
  const flow = new Set(['a', 'div', 'footer', 'li', 'ol', 'p', 'ul']);
  const stack = [{ tag: 'body', phrasing: false }];
  const whitespace = (c) => c !== undefined && /[\t\n\f\r ]/.test(c);
  const letter = (c) => c !== undefined && /[A-Za-z]/.test(c);
  const attributeChar = (c) => c !== undefined && /[A-Za-z0-9_-]/.test(c);
  let cursor = 0;
  const skipSpace = () => {
    while (whitespace(source[cursor])) cursor++;
  };
  // Document::setMarkupUnsafe discards leading ASCII whitespace first.
  skipSpace();
  while (cursor < source.length) {
    const parent = stack[stack.length - 1];
    if (source[cursor] !== '<') {
      let end = source.indexOf('<', cursor);
      if (end === -1) end = source.length;
      const text = source.slice(cursor, end);
      // WebKit caches newline + spaces without applying its text limit.
      if (text.length >= 65536 && !/^\n *$/.test(text)) {
        const length = /[&\r]/.test(text)
          ? __callHost('htmlDecodedTextLength', null, text)
          : text.length;
        if (length >= 65536) return false;
      }
      cursor = end;
      continue;
    }
    cursor++;
    const closing = source[cursor] === '/';
    if (closing) cursor++;
    const start = cursor;
    while (letter(source[cursor])) cursor++;
    const tag = source.slice(start, cursor).toLowerCase();
    if (!tag) return false;
    if (closing) {
      if (stack.length === 1 || tag !== parent.tag) return false;
      skipSpace();
      if (source[cursor++] !== '>') return false;
      stack.pop();
      continue;
    }
    if (stack.length === 512) return false;
    if (!whitespace(source[cursor]) && source[cursor] !== '>' && source[cursor] !== '/')
      return false;
    const listParent = parent.tag === 'ol' || parent.tag === 'ul';
    if (listParent) {
      if (tag !== 'li') return false;
    } else if (parent.tag === 'select') {
      if (tag !== 'option') return false;
    } else {
      if (parent.tag === 'option' || (!phrasing.has(tag) && !flow.has(tag))) return false;
      if (parent.phrasing && !phrasing.has(tag) && tag !== 'a') return false;
      if (tag === 'li' && stack.some((entry) => entry.tag === 'li')) return false;
    }
    if (tag === 'a' && stack.some((entry) => entry.tag === 'a')) return false;
    skipSpace();
    while (true) {
      if (source[cursor] === '>') {
        cursor++;
        break;
      }
      if (source[cursor] === '/') {
        cursor++;
        skipSpace();
        if (source[cursor++] !== '>') return false;
        break;
      }
      const attributeStart = cursor;
      while (attributeChar(source[cursor])) cursor++;
      const name = source.slice(attributeStart, cursor).toLowerCase();
      if (!name) return false;
      if (name.length > 2 && name.startsWith('on')) {
        // Safari accepts a terminal bare on* attribute when scanning
        // stops directly at '>' or '/>'; an assigned value bails out.
        if (source[cursor] !== '>' && source[cursor] !== '/') return false;
        continue;
      }
      skipSpace();
      if (source[cursor] === '=') {
        cursor++;
        skipSpace();
        const quote = source[cursor];
        if (quote === '"' || quote === "'") {
          const end = source.indexOf(quote, cursor + 1);
          if (end === -1) return false;
          cursor = end + 1;
        } else {
          while (attributeChar(source[cursor])) cursor++;
          if (!whitespace(source[cursor]) && source[cursor] !== '>') return false;
        }
        skipSpace();
      }
    }
    if (tag !== 'br' && tag !== 'input') {
      stack.push({
        tag,
        phrasing:
          tag === 'a' ? parent.phrasing : tag === 'p' || (phrasing.has(tag) && tag !== 'select'),
      });
    }
  }
  return stack.length === 1;
}
