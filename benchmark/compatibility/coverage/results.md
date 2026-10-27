# DOM coverage probe results

Generated 2026-10-22T14:53:50.674Z. Targeted checks; not an overall conformance score.

Runtime releases and source-review checkouts are recorded separately in report.json.

| Probe | Area | Brimp core | jsdom | happy-dom | Brimp + bundled plugins | Peer-confirmed gap |
| --- | --- | --- | --- | --- | --- | --- |
| document-adopt | document | matched | matched | different | matched |  |
| document-import | document | matched | matched | matched | matched |  |
| attribute-create-attach | attributes | matched | matched | matched | matched |  |
| attribute-namespace-map | attributes | matched | matched | matched | matched |  |
| namespace-query | document | matched | matched | different | matched |  |
| namespace-lookup | document | matched | matched | throws | matched |  |
| xml-parser | xml | throws | matched | matched | throws | yes |
| xml-serializer | xml | throws | matched | matched | throws | yes |
| xml-document-cdata | xml | throws | matched | throws | throws | yes |
| collection-live-identity | collections | matched | matched | matched | matched |  |
| collection-named-properties | collections | matched | matched | matched | matched |  |
| collection-namespace-name | collections | matched | matched | different | matched |  |
| treewalker-navigation | traversal | matched | matched | matched | matched |  |
| treewalker-reject-subtree | traversal | matched | matched | matched | matched |  |
| iterator-removal | traversal | matched | matched | throws | matched |  |
| range-partial-extract | range | matched | matched | matched | matched |  |
| range-replace-child | range | matched | matched | different | matched |  |
| range-replace-with | range | matched | matched | different | matched |  |
| range-invalid-replacement-atomic | range | matched | matched | different | matched |  |
| range-text-split-normalize | range | matched | different | matched | matched |  |
| form-required-validity | forms | matched | matched | matched | matched |  |
| form-custom-validity | forms | matched | matched | matched | matched |  |
| input-number-step | forms | matched | matched | matched | matched |  |
| input-selection-replace | forms | matched | matched | different | matched |  |
| form-associated-controls | forms | matched | matched | matched | matched |  |
| formdata-submitter | forms | matched | matched | matched | matched |  |
| formdata-disabled-fieldset | forms | matched | matched | different | matched |  |
| form-labels | forms | matched | matched | matched | matched |  |
| form-reset-dirty-value | forms | matched | matched | matched | matched |  |
| template-owner-document | document | matched | matched | different | matched |  |
| custom-element-upgrade | custom-elements | matched | matched | different | matched |  |
| custom-element-when-defined | custom-elements | matched | matched | different | matched |  |
| customized-built-in | custom-elements | matched | matched | matched | matched |  |
| element-internals | custom-elements | matched | matched | throws | matched |  |
| shadow-slot-assignment | shadow | matched | matched | different | matched |  |
| mutation-observer-batch | events | matched | matched | matched | matched |  |
| event-abort-listener | events | matched | matched | matched | matched |  |
| selector-has | selectors | matched | matched | matched | matched |  |
| css-inline-priority | css | matched | matched | matched | matched |  |
| stylesheet-rule-editing | cssom | matched | matched | matched | matched |  |
| stylesheet-adoption | cssom | matched | matched | matched | matched |  |
| computed-style-cascade | computed-style | throws | matched | matched | throws | yes |
| svg-animated-length | svg | throws | throws | matched | throws | yes |
| svg-class-name | svg | different | matched | different | different | yes |
| storage-basic | plugin | throws | matched | matched | matched | yes |
| geometry-matrix | plugin | throws | throws | matched | matched | yes |
| file-reader-text | files | matched | matched | matched | matched |  |
| document-write | document | matched | matched | matched | matched |  |
| mutation-observer-after-task | events | matched | matched | matched | matched |  |
| mutation-observer-take-records | events | matched | matched | matched | matched |  |
| css-inline-basic | css | matched | matched | matched | matched |  |
| css-inline-parsed-priority | css | matched | matched | matched | matched |  |
| microtask-order | events | matched | matched | matched | matched |  |

An exception is recorded with its name/message in report.json. Surface candidates include inherited prototype members; instance properties, descriptors and semantics need investigation. Plugin rows are intentional default-core omissions, not missing plugin implementations.
