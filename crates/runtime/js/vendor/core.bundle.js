(() => {
  var __create = Object.create;
  var __defProp = Object.defineProperty;
  var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __getProtoOf = Object.getPrototypeOf;
  var __hasOwnProp = Object.prototype.hasOwnProperty;
  var __commonJS = (cb, mod) => function __require() {
    try {
      return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
    } catch (e2) {
      throw mod = 0, e2;
    }
  };
  var __export = (target, all) => {
    for (var name2 in all)
      __defProp(target, name2, { get: all[name2], enumerable: true });
  };
  var __copyProps = (to2, from, except, desc) => {
    if (from && typeof from === "object" || typeof from === "function") {
      for (let key of __getOwnPropNames(from))
        if (!__hasOwnProp.call(to2, key) && key !== except)
          __defProp(to2, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
    }
    return to2;
  };
  var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
    // If the importer is in node compatibility mode or this is not an ESM
    // file that has been converted to a CommonJS file using a Babel-
    // compatible transform (i.e. "__esModule" has not been set), then set
    // "default" to the CommonJS "module.exports" for node compatibility.
    isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
    mod
  ));

  // node_modules/core-js-pure/modules/es.error.to-string.js
  var require_es_error_to_string = __commonJS({
    "node_modules/core-js-pure/modules/es.error.to-string.js"() {
    }
  });

  // node_modules/core-js-pure/internals/fails.js
  var require_fails = __commonJS({
    "node_modules/core-js-pure/internals/fails.js"(exports, module) {
      "use strict";
      module.exports = function(exec) {
        try {
          return !!exec();
        } catch (error) {
          return true;
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/function-bind-native.js
  var require_function_bind_native = __commonJS({
    "node_modules/core-js-pure/internals/function-bind-native.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      module.exports = !fails(function() {
        var test = function() {
        }.bind();
        return typeof test != "function" || test.hasOwnProperty("prototype");
      });
    }
  });

  // node_modules/core-js-pure/internals/function-uncurry-this.js
  var require_function_uncurry_this = __commonJS({
    "node_modules/core-js-pure/internals/function-uncurry-this.js"(exports, module) {
      "use strict";
      var NATIVE_BIND = require_function_bind_native();
      var FunctionPrototype = Function.prototype;
      var call = FunctionPrototype.call;
      var uncurryThisWithBind = NATIVE_BIND && FunctionPrototype.bind.bind(call, call);
      module.exports = NATIVE_BIND ? uncurryThisWithBind : function(fn2) {
        return function() {
          return call.apply(fn2, arguments);
        };
      };
    }
  });

  // node_modules/core-js-pure/internals/classof-raw.js
  var require_classof_raw = __commonJS({
    "node_modules/core-js-pure/internals/classof-raw.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var toString = uncurryThis({}.toString);
      var stringSlice = uncurryThis("".slice);
      module.exports = function(it2) {
        return stringSlice(toString(it2), 8, -1);
      };
    }
  });

  // node_modules/core-js-pure/internals/indexed-object.js
  var require_indexed_object = __commonJS({
    "node_modules/core-js-pure/internals/indexed-object.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var fails = require_fails();
      var classof = require_classof_raw();
      var $Object = Object;
      var split = uncurryThis("".split);
      module.exports = fails(function() {
        return !$Object("z").propertyIsEnumerable(0);
      }) ? function(it2) {
        return classof(it2) === "String" ? split(it2, "") : $Object(it2);
      } : $Object;
    }
  });

  // node_modules/core-js-pure/internals/is-null-or-undefined.js
  var require_is_null_or_undefined = __commonJS({
    "node_modules/core-js-pure/internals/is-null-or-undefined.js"(exports, module) {
      "use strict";
      module.exports = function(it2) {
        return it2 === null || it2 === void 0;
      };
    }
  });

  // node_modules/core-js-pure/internals/require-object-coercible.js
  var require_require_object_coercible = __commonJS({
    "node_modules/core-js-pure/internals/require-object-coercible.js"(exports, module) {
      "use strict";
      var isNullOrUndefined = require_is_null_or_undefined();
      var $TypeError = TypeError;
      module.exports = function(it2) {
        if (isNullOrUndefined(it2)) throw new $TypeError("Can't call method on " + it2);
        return it2;
      };
    }
  });

  // node_modules/core-js-pure/internals/to-indexed-object.js
  var require_to_indexed_object = __commonJS({
    "node_modules/core-js-pure/internals/to-indexed-object.js"(exports, module) {
      "use strict";
      var IndexedObject = require_indexed_object();
      var requireObjectCoercible = require_require_object_coercible();
      module.exports = function(it2) {
        return IndexedObject(requireObjectCoercible(it2));
      };
    }
  });

  // node_modules/core-js-pure/internals/add-to-unscopables.js
  var require_add_to_unscopables = __commonJS({
    "node_modules/core-js-pure/internals/add-to-unscopables.js"(exports, module) {
      "use strict";
      module.exports = function() {
      };
    }
  });

  // node_modules/core-js-pure/internals/iterators.js
  var require_iterators = __commonJS({
    "node_modules/core-js-pure/internals/iterators.js"(exports, module) {
      "use strict";
      module.exports = Object.create ? /* @__PURE__ */ Object.create(null) : {};
    }
  });

  // node_modules/core-js-pure/internals/global-this.js
  var require_global_this = __commonJS({
    "node_modules/core-js-pure/internals/global-this.js"(exports, module) {
      "use strict";
      var check = function(it2) {
        return it2 && it2.Math === Math && it2;
      };
      module.exports = // eslint-disable-next-line es/no-global-this -- safe
      check(typeof globalThis == "object" && globalThis) || check(typeof window == "object" && window) || // eslint-disable-next-line no-restricted-globals -- safe
      check(typeof self == "object" && self) || check(typeof global == "object" && global) || check(typeof exports == "object" && exports) || // eslint-disable-next-line no-new-func -- fallback
      /* @__PURE__ */ (function() {
        return this;
      })() || Function("return this")();
    }
  });

  // node_modules/core-js-pure/internals/is-callable.js
  var require_is_callable = __commonJS({
    "node_modules/core-js-pure/internals/is-callable.js"(exports, module) {
      "use strict";
      var documentAll = typeof document == "object" && document.all;
      module.exports = typeof documentAll == "undefined" && documentAll !== void 0 ? function(argument) {
        return typeof argument == "function" || argument === documentAll;
      } : function(argument) {
        return typeof argument == "function";
      };
    }
  });

  // node_modules/core-js-pure/internals/weak-map-basic-detection.js
  var require_weak_map_basic_detection = __commonJS({
    "node_modules/core-js-pure/internals/weak-map-basic-detection.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var isCallable = require_is_callable();
      var WeakMap2 = globalThis2.WeakMap;
      module.exports = isCallable(WeakMap2) && /native code/.test(String(WeakMap2));
    }
  });

  // node_modules/core-js-pure/internals/is-object.js
  var require_is_object = __commonJS({
    "node_modules/core-js-pure/internals/is-object.js"(exports, module) {
      "use strict";
      var isCallable = require_is_callable();
      module.exports = function(it2) {
        return typeof it2 == "object" ? it2 !== null : isCallable(it2);
      };
    }
  });

  // node_modules/core-js-pure/internals/descriptors.js
  var require_descriptors = __commonJS({
    "node_modules/core-js-pure/internals/descriptors.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      module.exports = !fails(function() {
        return Object.defineProperty({}, 1, { get: function() {
          return 7;
        } })[1] !== 7;
      });
    }
  });

  // node_modules/core-js-pure/internals/document-create-element.js
  var require_document_create_element = __commonJS({
    "node_modules/core-js-pure/internals/document-create-element.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var isObject = require_is_object();
      var document2 = globalThis2.document;
      var EXISTS = isObject(document2) && isObject(document2.createElement);
      module.exports = function(it2) {
        return EXISTS ? document2.createElement(it2) : {};
      };
    }
  });

  // node_modules/core-js-pure/internals/ie8-dom-define.js
  var require_ie8_dom_define = __commonJS({
    "node_modules/core-js-pure/internals/ie8-dom-define.js"(exports, module) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var fails = require_fails();
      var createElement = require_document_create_element();
      module.exports = !DESCRIPTORS && !fails(function() {
        return Object.defineProperty(createElement("div"), "a", {
          get: function() {
            return 7;
          }
        }).a !== 7;
      });
    }
  });

  // node_modules/core-js-pure/internals/v8-prototype-define-bug.js
  var require_v8_prototype_define_bug = __commonJS({
    "node_modules/core-js-pure/internals/v8-prototype-define-bug.js"(exports, module) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var fails = require_fails();
      module.exports = DESCRIPTORS && fails(function() {
        return Object.defineProperty(function() {
        }, "prototype", {
          value: 42,
          writable: false
        }).prototype !== 42;
      });
    }
  });

  // node_modules/core-js-pure/internals/an-object.js
  var require_an_object = __commonJS({
    "node_modules/core-js-pure/internals/an-object.js"(exports, module) {
      "use strict";
      var isObject = require_is_object();
      var $String = String;
      var $TypeError = TypeError;
      module.exports = function(argument) {
        if (isObject(argument)) return argument;
        throw new $TypeError($String(argument) + " is not an object");
      };
    }
  });

  // node_modules/core-js-pure/internals/function-call.js
  var require_function_call = __commonJS({
    "node_modules/core-js-pure/internals/function-call.js"(exports, module) {
      "use strict";
      var NATIVE_BIND = require_function_bind_native();
      var call = Function.prototype.call;
      module.exports = NATIVE_BIND ? call.bind(call) : function() {
        return call.apply(call, arguments);
      };
    }
  });

  // node_modules/core-js-pure/internals/path.js
  var require_path = __commonJS({
    "node_modules/core-js-pure/internals/path.js"(exports, module) {
      "use strict";
      module.exports = {};
    }
  });

  // node_modules/core-js-pure/internals/get-built-in.js
  var require_get_built_in = __commonJS({
    "node_modules/core-js-pure/internals/get-built-in.js"(exports, module) {
      "use strict";
      var path = require_path();
      var globalThis2 = require_global_this();
      var isCallable = require_is_callable();
      var aFunction = function(variable) {
        return isCallable(variable) ? variable : void 0;
      };
      module.exports = function(namespace, method) {
        return arguments.length < 2 ? aFunction(path[namespace]) || aFunction(globalThis2[namespace]) : path[namespace] && path[namespace][method] || globalThis2[namespace] && globalThis2[namespace][method];
      };
    }
  });

  // node_modules/core-js-pure/internals/object-is-prototype-of.js
  var require_object_is_prototype_of = __commonJS({
    "node_modules/core-js-pure/internals/object-is-prototype-of.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      module.exports = uncurryThis({}.isPrototypeOf);
    }
  });

  // node_modules/core-js-pure/internals/environment-user-agent.js
  var require_environment_user_agent = __commonJS({
    "node_modules/core-js-pure/internals/environment-user-agent.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var navigator = globalThis2.navigator;
      var userAgent = navigator && navigator.userAgent;
      module.exports = userAgent ? String(userAgent) : "";
    }
  });

  // node_modules/core-js-pure/internals/environment-v8-version.js
  var require_environment_v8_version = __commonJS({
    "node_modules/core-js-pure/internals/environment-v8-version.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var userAgent = require_environment_user_agent();
      var process = globalThis2.process;
      var Deno2 = globalThis2.Deno;
      var versions = process && process.versions || Deno2 && Deno2.version;
      var v8 = versions && versions.v8;
      var match;
      var version;
      if (v8) {
        match = v8.split(".");
        version = match[0] > 0 && match[0] < 4 ? 1 : +(match[0] + match[1]);
      }
      if (!version && userAgent) {
        match = userAgent.match(/Edge\/(\d+)/);
        if (!match || match[1] >= 74) {
          match = userAgent.match(/Chrome\/(\d+)/);
          if (match) version = +match[1];
        }
      }
      module.exports = version;
    }
  });

  // node_modules/core-js-pure/internals/symbol-constructor-detection.js
  var require_symbol_constructor_detection = __commonJS({
    "node_modules/core-js-pure/internals/symbol-constructor-detection.js"(exports, module) {
      "use strict";
      var V8_VERSION = require_environment_v8_version();
      var fails = require_fails();
      var globalThis2 = require_global_this();
      var $String = globalThis2.String;
      module.exports = !!Object.getOwnPropertySymbols && !fails(function() {
        var symbol = /* @__PURE__ */ Symbol("symbol detection");
        return !$String(symbol) || !(Object(symbol) instanceof Symbol) || // Chrome 38-40 symbols are not inherited from DOM collections prototypes to instances
        !Symbol.sham && V8_VERSION && V8_VERSION < 41;
      });
    }
  });

  // node_modules/core-js-pure/internals/use-symbol-as-uid.js
  var require_use_symbol_as_uid = __commonJS({
    "node_modules/core-js-pure/internals/use-symbol-as-uid.js"(exports, module) {
      "use strict";
      var NATIVE_SYMBOL = require_symbol_constructor_detection();
      module.exports = NATIVE_SYMBOL && !Symbol.sham && typeof Symbol.iterator == "symbol";
    }
  });

  // node_modules/core-js-pure/internals/is-symbol.js
  var require_is_symbol = __commonJS({
    "node_modules/core-js-pure/internals/is-symbol.js"(exports, module) {
      "use strict";
      var getBuiltIn = require_get_built_in();
      var isCallable = require_is_callable();
      var isPrototypeOf = require_object_is_prototype_of();
      var USE_SYMBOL_AS_UID = require_use_symbol_as_uid();
      var $Object = Object;
      module.exports = USE_SYMBOL_AS_UID ? function(it2) {
        return typeof it2 == "symbol";
      } : function(it2) {
        var $Symbol = getBuiltIn("Symbol");
        return isCallable($Symbol) && isPrototypeOf($Symbol.prototype, $Object(it2));
      };
    }
  });

  // node_modules/core-js-pure/internals/try-to-string.js
  var require_try_to_string = __commonJS({
    "node_modules/core-js-pure/internals/try-to-string.js"(exports, module) {
      "use strict";
      var $String = String;
      module.exports = function(argument) {
        try {
          return $String(argument);
        } catch (error) {
          return "Object";
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/a-callable.js
  var require_a_callable = __commonJS({
    "node_modules/core-js-pure/internals/a-callable.js"(exports, module) {
      "use strict";
      var isCallable = require_is_callable();
      var tryToString = require_try_to_string();
      var $TypeError = TypeError;
      module.exports = function(argument) {
        if (isCallable(argument)) return argument;
        throw new $TypeError(tryToString(argument) + " is not a function");
      };
    }
  });

  // node_modules/core-js-pure/internals/get-method.js
  var require_get_method = __commonJS({
    "node_modules/core-js-pure/internals/get-method.js"(exports, module) {
      "use strict";
      var aCallable = require_a_callable();
      var isNullOrUndefined = require_is_null_or_undefined();
      module.exports = function(V2, P2) {
        var func = V2[P2];
        return isNullOrUndefined(func) ? void 0 : aCallable(func);
      };
    }
  });

  // node_modules/core-js-pure/internals/ordinary-to-primitive.js
  var require_ordinary_to_primitive = __commonJS({
    "node_modules/core-js-pure/internals/ordinary-to-primitive.js"(exports, module) {
      "use strict";
      var call = require_function_call();
      var isCallable = require_is_callable();
      var isObject = require_is_object();
      var $TypeError = TypeError;
      module.exports = function(input, pref) {
        var fn2, val;
        if (pref === "string" && isCallable(fn2 = input.toString) && !isObject(val = call(fn2, input))) return val;
        if (isCallable(fn2 = input.valueOf) && !isObject(val = call(fn2, input))) return val;
        if (pref !== "string" && isCallable(fn2 = input.toString) && !isObject(val = call(fn2, input))) return val;
        throw new $TypeError("Can't convert object to primitive value");
      };
    }
  });

  // node_modules/core-js-pure/internals/shared-store.js
  var require_shared_store = __commonJS({
    "node_modules/core-js-pure/internals/shared-store.js"(exports, module) {
      module.exports = {};
    }
  });

  // node_modules/core-js-pure/internals/shared.js
  var require_shared = __commonJS({
    "node_modules/core-js-pure/internals/shared.js"(exports, module) {
      "use strict";
      var store = require_shared_store();
      var create = Object.create || Object;
      module.exports = function(key, value) {
        return store[key] || (store[key] = value || create(null));
      };
    }
  });

  // node_modules/core-js-pure/internals/to-object.js
  var require_to_object = __commonJS({
    "node_modules/core-js-pure/internals/to-object.js"(exports, module) {
      "use strict";
      var requireObjectCoercible = require_require_object_coercible();
      var $Object = Object;
      module.exports = function(argument) {
        return $Object(requireObjectCoercible(argument));
      };
    }
  });

  // node_modules/core-js-pure/internals/has-own-property.js
  var require_has_own_property = __commonJS({
    "node_modules/core-js-pure/internals/has-own-property.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var toObject = require_to_object();
      var hasOwnProperty = uncurryThis({}.hasOwnProperty);
      module.exports = Object.hasOwn || function hasOwn(it2, key) {
        return hasOwnProperty(toObject(it2), key);
      };
    }
  });

  // node_modules/core-js-pure/internals/uid.js
  var require_uid = __commonJS({
    "node_modules/core-js-pure/internals/uid.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var id = 0;
      var postfix = Math.random();
      var toString = uncurryThis(1.1.toString);
      module.exports = function(key) {
        return "Symbol(" + (key === void 0 ? "" : key) + ")_" + toString(++id + postfix, 36);
      };
    }
  });

  // node_modules/core-js-pure/internals/well-known-symbol.js
  var require_well_known_symbol = __commonJS({
    "node_modules/core-js-pure/internals/well-known-symbol.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var shared = require_shared();
      var hasOwn = require_has_own_property();
      var uid = require_uid();
      var NATIVE_SYMBOL = require_symbol_constructor_detection();
      var USE_SYMBOL_AS_UID = require_use_symbol_as_uid();
      var Symbol2 = globalThis2.Symbol;
      var WellKnownSymbolsStore = shared("wks");
      var createWellKnownSymbol = USE_SYMBOL_AS_UID ? Symbol2["for"] || Symbol2 : Symbol2 && Symbol2.withoutSetter || uid;
      module.exports = function(name2) {
        if (!hasOwn(WellKnownSymbolsStore, name2)) {
          WellKnownSymbolsStore[name2] = NATIVE_SYMBOL && hasOwn(Symbol2, name2) ? Symbol2[name2] : createWellKnownSymbol("Symbol." + name2);
        }
        return WellKnownSymbolsStore[name2];
      };
    }
  });

  // node_modules/core-js-pure/internals/to-primitive.js
  var require_to_primitive = __commonJS({
    "node_modules/core-js-pure/internals/to-primitive.js"(exports, module) {
      "use strict";
      var call = require_function_call();
      var isObject = require_is_object();
      var isSymbol = require_is_symbol();
      var getMethod = require_get_method();
      var ordinaryToPrimitive = require_ordinary_to_primitive();
      var wellKnownSymbol = require_well_known_symbol();
      var $TypeError = TypeError;
      var TO_PRIMITIVE = wellKnownSymbol("toPrimitive");
      module.exports = function(input, pref) {
        if (!isObject(input) || isSymbol(input)) return input;
        var exoticToPrim = getMethod(input, TO_PRIMITIVE);
        var result;
        if (exoticToPrim) {
          if (pref === void 0) pref = "default";
          result = call(exoticToPrim, input, pref);
          if (!isObject(result) || isSymbol(result)) return result;
          throw new $TypeError("Can't convert object to primitive value");
        }
        if (pref === void 0) pref = "number";
        return ordinaryToPrimitive(input, pref);
      };
    }
  });

  // node_modules/core-js-pure/internals/to-property-key.js
  var require_to_property_key = __commonJS({
    "node_modules/core-js-pure/internals/to-property-key.js"(exports, module) {
      "use strict";
      var toPrimitive = require_to_primitive();
      var isSymbol = require_is_symbol();
      module.exports = function(argument) {
        var key = toPrimitive(argument, "string");
        return isSymbol(key) ? key : key + "";
      };
    }
  });

  // node_modules/core-js-pure/internals/object-define-property.js
  var require_object_define_property = __commonJS({
    "node_modules/core-js-pure/internals/object-define-property.js"(exports) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var IE8_DOM_DEFINE = require_ie8_dom_define();
      var V8_PROTOTYPE_DEFINE_BUG = require_v8_prototype_define_bug();
      var anObject = require_an_object();
      var toPropertyKey = require_to_property_key();
      var $TypeError = TypeError;
      var $defineProperty = Object.defineProperty;
      var $getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
      var ENUMERABLE = "enumerable";
      var CONFIGURABLE = "configurable";
      var WRITABLE = "writable";
      exports.f = DESCRIPTORS ? V8_PROTOTYPE_DEFINE_BUG ? function defineProperty(O2, P2, Attributes) {
        anObject(O2);
        P2 = toPropertyKey(P2);
        anObject(Attributes);
        if (typeof O2 === "function" && P2 === "prototype" && "value" in Attributes && WRITABLE in Attributes && !Attributes[WRITABLE]) {
          var current = $getOwnPropertyDescriptor(O2, P2);
          if (current && current[WRITABLE]) {
            O2[P2] = Attributes.value;
            Attributes = {
              configurable: CONFIGURABLE in Attributes ? Attributes[CONFIGURABLE] : current[CONFIGURABLE],
              enumerable: ENUMERABLE in Attributes ? Attributes[ENUMERABLE] : current[ENUMERABLE],
              writable: false
            };
          }
        }
        return $defineProperty(O2, P2, Attributes);
      } : $defineProperty : function defineProperty(O2, P2, Attributes) {
        anObject(O2);
        P2 = toPropertyKey(P2);
        anObject(Attributes);
        if (IE8_DOM_DEFINE) try {
          return $defineProperty(O2, P2, Attributes);
        } catch (error) {
        }
        if ("get" in Attributes || "set" in Attributes) throw new $TypeError("Accessors not supported");
        if ("value" in Attributes) O2[P2] = Attributes.value;
        return O2;
      };
    }
  });

  // node_modules/core-js-pure/internals/create-property-descriptor.js
  var require_create_property_descriptor = __commonJS({
    "node_modules/core-js-pure/internals/create-property-descriptor.js"(exports, module) {
      "use strict";
      module.exports = function(bitmap, value) {
        return {
          enumerable: !(bitmap & 1),
          configurable: !(bitmap & 2),
          writable: !(bitmap & 4),
          value
        };
      };
    }
  });

  // node_modules/core-js-pure/internals/create-non-enumerable-property.js
  var require_create_non_enumerable_property = __commonJS({
    "node_modules/core-js-pure/internals/create-non-enumerable-property.js"(exports, module) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var definePropertyModule = require_object_define_property();
      var createPropertyDescriptor = require_create_property_descriptor();
      module.exports = DESCRIPTORS ? function(object, key, value) {
        return definePropertyModule.f(object, key, createPropertyDescriptor(1, value));
      } : function(object, key, value) {
        object[key] = value;
        return object;
      };
    }
  });

  // node_modules/core-js-pure/internals/shared-key.js
  var require_shared_key = __commonJS({
    "node_modules/core-js-pure/internals/shared-key.js"(exports, module) {
      "use strict";
      var shared = require_shared();
      var uid = require_uid();
      var keys = shared("keys");
      module.exports = function(key) {
        return keys[key] || (keys[key] = uid(key));
      };
    }
  });

  // node_modules/core-js-pure/internals/hidden-keys.js
  var require_hidden_keys = __commonJS({
    "node_modules/core-js-pure/internals/hidden-keys.js"(exports, module) {
      "use strict";
      module.exports = {};
    }
  });

  // node_modules/core-js-pure/internals/internal-state.js
  var require_internal_state = __commonJS({
    "node_modules/core-js-pure/internals/internal-state.js"(exports, module) {
      "use strict";
      var NATIVE_WEAK_MAP = require_weak_map_basic_detection();
      var globalThis2 = require_global_this();
      var isObject = require_is_object();
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      var hasOwn = require_has_own_property();
      var shared = require_shared_store();
      var sharedKey = require_shared_key();
      var hiddenKeys = require_hidden_keys();
      var OBJECT_ALREADY_INITIALIZED = "Object already initialized";
      var TypeError2 = globalThis2.TypeError;
      var WeakMap2 = globalThis2.WeakMap;
      var set;
      var get;
      var has;
      var enforce = function(it2) {
        return has(it2) ? get(it2) : set(it2, {});
      };
      var getterFor = function(TYPE) {
        return function(it2) {
          var state;
          if (!isObject(it2) || (state = get(it2)).type !== TYPE) {
            throw new TypeError2("Incompatible receiver, " + TYPE + " required");
          }
          return state;
        };
      };
      if (NATIVE_WEAK_MAP || shared.state) {
        store = shared.state || (shared.state = new WeakMap2());
        store.get = store.get;
        store.has = store.has;
        store.set = store.set;
        set = function(it2, metadata) {
          if (store.has(it2)) throw new TypeError2(OBJECT_ALREADY_INITIALIZED);
          metadata.facade = it2;
          store.set(it2, metadata);
          return metadata;
        };
        get = function(it2) {
          return store.get(it2) || {};
        };
        has = function(it2) {
          return store.has(it2);
        };
      } else {
        STATE = sharedKey("state");
        hiddenKeys[STATE] = true;
        set = function(it2, metadata) {
          if (hasOwn(it2, STATE)) throw new TypeError2(OBJECT_ALREADY_INITIALIZED);
          metadata.facade = it2;
          createNonEnumerableProperty(it2, STATE, metadata);
          return metadata;
        };
        get = function(it2) {
          return hasOwn(it2, STATE) ? it2[STATE] : {};
        };
        has = function(it2) {
          return hasOwn(it2, STATE);
        };
      }
      var store;
      var STATE;
      module.exports = {
        set,
        get,
        has,
        enforce,
        getterFor
      };
    }
  });

  // node_modules/core-js-pure/internals/function-apply.js
  var require_function_apply = __commonJS({
    "node_modules/core-js-pure/internals/function-apply.js"(exports, module) {
      "use strict";
      var NATIVE_BIND = require_function_bind_native();
      var FunctionPrototype = Function.prototype;
      var apply = FunctionPrototype.apply;
      var call = FunctionPrototype.call;
      module.exports = typeof Reflect == "object" && Reflect.apply || (NATIVE_BIND ? call.bind(apply) : function() {
        return call.apply(apply, arguments);
      });
    }
  });

  // node_modules/core-js-pure/internals/function-uncurry-this-clause.js
  var require_function_uncurry_this_clause = __commonJS({
    "node_modules/core-js-pure/internals/function-uncurry-this-clause.js"(exports, module) {
      "use strict";
      var classofRaw = require_classof_raw();
      var uncurryThis = require_function_uncurry_this();
      module.exports = function(fn2) {
        if (classofRaw(fn2) === "Function") return uncurryThis(fn2);
      };
    }
  });

  // node_modules/core-js-pure/internals/object-property-is-enumerable.js
  var require_object_property_is_enumerable = __commonJS({
    "node_modules/core-js-pure/internals/object-property-is-enumerable.js"(exports) {
      "use strict";
      var $propertyIsEnumerable = {}.propertyIsEnumerable;
      var getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
      var NASHORN_BUG = getOwnPropertyDescriptor && !$propertyIsEnumerable.call({ 1: 2 }, 1);
      exports.f = NASHORN_BUG ? function propertyIsEnumerable(V2) {
        var descriptor = getOwnPropertyDescriptor(this, V2);
        return !!descriptor && descriptor.enumerable;
      } : $propertyIsEnumerable;
    }
  });

  // node_modules/core-js-pure/internals/object-get-own-property-descriptor.js
  var require_object_get_own_property_descriptor = __commonJS({
    "node_modules/core-js-pure/internals/object-get-own-property-descriptor.js"(exports) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var call = require_function_call();
      var propertyIsEnumerableModule = require_object_property_is_enumerable();
      var createPropertyDescriptor = require_create_property_descriptor();
      var toIndexedObject = require_to_indexed_object();
      var toPropertyKey = require_to_property_key();
      var hasOwn = require_has_own_property();
      var IE8_DOM_DEFINE = require_ie8_dom_define();
      var $getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
      exports.f = DESCRIPTORS ? $getOwnPropertyDescriptor : function getOwnPropertyDescriptor(O2, P2) {
        O2 = toIndexedObject(O2);
        P2 = toPropertyKey(P2);
        if (IE8_DOM_DEFINE) try {
          return $getOwnPropertyDescriptor(O2, P2);
        } catch (error) {
        }
        if (hasOwn(O2, P2)) return createPropertyDescriptor(!call(propertyIsEnumerableModule.f, O2, P2), O2[P2]);
      };
    }
  });

  // node_modules/core-js-pure/internals/is-forced.js
  var require_is_forced = __commonJS({
    "node_modules/core-js-pure/internals/is-forced.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      var isCallable = require_is_callable();
      var replacement = /#|\.prototype\./;
      var isForced = function(feature, detection) {
        var value = data[normalize(feature)];
        return value === POLYFILL ? true : value === NATIVE ? false : isCallable(detection) ? fails(detection) : !!detection;
      };
      var normalize = isForced.normalize = function(string) {
        return String(string).replace(replacement, ".").toLowerCase();
      };
      var data = isForced.data = {};
      var NATIVE = isForced.NATIVE = "N";
      var POLYFILL = isForced.POLYFILL = "P";
      module.exports = isForced;
    }
  });

  // node_modules/core-js-pure/internals/function-bind-context.js
  var require_function_bind_context = __commonJS({
    "node_modules/core-js-pure/internals/function-bind-context.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this_clause();
      var aCallable = require_a_callable();
      var NATIVE_BIND = require_function_bind_native();
      var bind = uncurryThis(uncurryThis.bind);
      module.exports = function(fn2, that) {
        aCallable(fn2);
        return that === void 0 ? fn2 : NATIVE_BIND ? bind(fn2, that) : function() {
          return fn2.apply(that, arguments);
        };
      };
    }
  });

  // node_modules/core-js-pure/internals/export.js
  var require_export = __commonJS({
    "node_modules/core-js-pure/internals/export.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var apply = require_function_apply();
      var uncurryThis = require_function_uncurry_this_clause();
      var isCallable = require_is_callable();
      var getOwnPropertyDescriptor = require_object_get_own_property_descriptor().f;
      var isForced = require_is_forced();
      var path = require_path();
      var bind = require_function_bind_context();
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      var hasOwn = require_has_own_property();
      require_shared_store();
      var wrapConstructor = function(NativeConstructor) {
        var Wrapper = function(a2, b2, c2) {
          if (this instanceof Wrapper) {
            switch (arguments.length) {
              case 0:
                return new NativeConstructor();
              case 1:
                return new NativeConstructor(a2);
              case 2:
                return new NativeConstructor(a2, b2);
            }
            return new NativeConstructor(a2, b2, c2);
          }
          return apply(NativeConstructor, this, arguments);
        };
        Wrapper.prototype = NativeConstructor.prototype;
        return Wrapper;
      };
      module.exports = function(options, source) {
        var TARGET = options.target;
        var GLOBAL = options.global;
        var STATIC = options.stat;
        var PROTO = options.proto;
        var nativeSource = GLOBAL ? globalThis2 : STATIC ? globalThis2[TARGET] : globalThis2[TARGET] && globalThis2[TARGET].prototype;
        var target = GLOBAL ? path : path[TARGET] || createNonEnumerableProperty(path, TARGET, {})[TARGET];
        var targetPrototype = target.prototype;
        var FORCED, USE_NATIVE, VIRTUAL_PROTOTYPE;
        var key, sourceProperty, targetProperty, nativeProperty, resultProperty, descriptor;
        for (key in source) {
          FORCED = isForced(GLOBAL ? key : TARGET + (STATIC ? "." : "#") + key, options.forced);
          USE_NATIVE = !FORCED && nativeSource && hasOwn(nativeSource, key);
          targetProperty = target[key];
          if (USE_NATIVE) if (options.dontCallGetSet) {
            descriptor = getOwnPropertyDescriptor(nativeSource, key);
            nativeProperty = descriptor && descriptor.value;
          } else nativeProperty = nativeSource[key];
          sourceProperty = USE_NATIVE && nativeProperty ? nativeProperty : source[key];
          if (!FORCED && !PROTO && typeof targetProperty == typeof sourceProperty) continue;
          if (options.bind && USE_NATIVE) resultProperty = bind(sourceProperty, globalThis2);
          else if (options.wrap && USE_NATIVE) resultProperty = wrapConstructor(sourceProperty);
          else if (PROTO && isCallable(sourceProperty)) resultProperty = uncurryThis(sourceProperty);
          else resultProperty = sourceProperty;
          if (options.sham || sourceProperty && sourceProperty.sham || targetProperty && targetProperty.sham) {
            createNonEnumerableProperty(resultProperty, "sham", true);
          }
          createNonEnumerableProperty(target, key, resultProperty);
          if (PROTO) {
            VIRTUAL_PROTOTYPE = TARGET + "Prototype";
            if (!hasOwn(path, VIRTUAL_PROTOTYPE)) {
              createNonEnumerableProperty(path, VIRTUAL_PROTOTYPE, {});
            }
            createNonEnumerableProperty(path[VIRTUAL_PROTOTYPE], key, sourceProperty);
            if (options.real && targetPrototype && (FORCED || !targetPrototype[key])) {
              createNonEnumerableProperty(targetPrototype, key, sourceProperty);
            }
          }
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/is-pure.js
  var require_is_pure = __commonJS({
    "node_modules/core-js-pure/internals/is-pure.js"(exports, module) {
      "use strict";
      module.exports = true;
    }
  });

  // node_modules/core-js-pure/internals/function-name.js
  var require_function_name = __commonJS({
    "node_modules/core-js-pure/internals/function-name.js"(exports, module) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var hasOwn = require_has_own_property();
      var FunctionPrototype = Function.prototype;
      var getDescriptor = DESCRIPTORS && Object.getOwnPropertyDescriptor;
      var EXISTS = hasOwn(FunctionPrototype, "name");
      var PROPER = EXISTS && function something() {
      }.name === "something";
      var CONFIGURABLE = EXISTS && (!DESCRIPTORS || DESCRIPTORS && getDescriptor(FunctionPrototype, "name").configurable);
      module.exports = {
        EXISTS,
        PROPER,
        CONFIGURABLE
      };
    }
  });

  // node_modules/core-js-pure/internals/math-trunc.js
  var require_math_trunc = __commonJS({
    "node_modules/core-js-pure/internals/math-trunc.js"(exports, module) {
      "use strict";
      var ceil = Math.ceil;
      var floor = Math.floor;
      module.exports = Math.trunc || function trunc(x2) {
        var n2 = +x2;
        return (n2 > 0 ? floor : ceil)(n2);
      };
    }
  });

  // node_modules/core-js-pure/internals/to-integer-or-infinity.js
  var require_to_integer_or_infinity = __commonJS({
    "node_modules/core-js-pure/internals/to-integer-or-infinity.js"(exports, module) {
      "use strict";
      var trunc = require_math_trunc();
      module.exports = function(argument) {
        var number = +argument;
        return number !== number || number === 0 ? 0 : trunc(number);
      };
    }
  });

  // node_modules/core-js-pure/internals/to-absolute-index.js
  var require_to_absolute_index = __commonJS({
    "node_modules/core-js-pure/internals/to-absolute-index.js"(exports, module) {
      "use strict";
      var toIntegerOrInfinity = require_to_integer_or_infinity();
      var max = Math.max;
      var min = Math.min;
      module.exports = function(index, length) {
        var integer = toIntegerOrInfinity(index);
        return integer < 0 ? max(integer + length, 0) : min(integer, length);
      };
    }
  });

  // node_modules/core-js-pure/internals/to-length.js
  var require_to_length = __commonJS({
    "node_modules/core-js-pure/internals/to-length.js"(exports, module) {
      "use strict";
      var toIntegerOrInfinity = require_to_integer_or_infinity();
      var min = Math.min;
      module.exports = function(argument) {
        var len = toIntegerOrInfinity(argument);
        return len > 0 ? min(len, 9007199254740991) : 0;
      };
    }
  });

  // node_modules/core-js-pure/internals/length-of-array-like.js
  var require_length_of_array_like = __commonJS({
    "node_modules/core-js-pure/internals/length-of-array-like.js"(exports, module) {
      "use strict";
      var toLength = require_to_length();
      module.exports = function(obj) {
        return toLength(obj.length);
      };
    }
  });

  // node_modules/core-js-pure/internals/array-includes.js
  var require_array_includes = __commonJS({
    "node_modules/core-js-pure/internals/array-includes.js"(exports, module) {
      "use strict";
      var toIndexedObject = require_to_indexed_object();
      var toAbsoluteIndex = require_to_absolute_index();
      var lengthOfArrayLike = require_length_of_array_like();
      var createMethod = function(IS_INCLUDES) {
        return function($this, el, fromIndex) {
          var O2 = toIndexedObject($this);
          var length = lengthOfArrayLike(O2);
          if (length === 0) return !IS_INCLUDES && -1;
          var index = toAbsoluteIndex(fromIndex, length);
          var value;
          if (IS_INCLUDES && el !== el) while (length > index) {
            value = O2[index++];
            if (value !== value) return true;
          }
          else for (; length > index; index++) {
            if ((IS_INCLUDES || index in O2) && O2[index] === el) return IS_INCLUDES || index || 0;
          }
          return !IS_INCLUDES && -1;
        };
      };
      module.exports = {
        // `Array.prototype.includes` method
        // https://tc39.es/ecma262/#sec-array.prototype.includes
        includes: createMethod(true),
        // `Array.prototype.indexOf` method
        // https://tc39.es/ecma262/#sec-array.prototype.indexof
        indexOf: createMethod(false)
      };
    }
  });

  // node_modules/core-js-pure/internals/object-keys-internal.js
  var require_object_keys_internal = __commonJS({
    "node_modules/core-js-pure/internals/object-keys-internal.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var hasOwn = require_has_own_property();
      var toIndexedObject = require_to_indexed_object();
      var indexOf = require_array_includes().indexOf;
      var hiddenKeys = require_hidden_keys();
      var push = uncurryThis([].push);
      module.exports = function(object, names) {
        var O2 = toIndexedObject(object);
        var i2 = 0;
        var result = [];
        var key;
        for (key in O2) !hasOwn(hiddenKeys, key) && hasOwn(O2, key) && push(result, key);
        while (names.length > i2) if (hasOwn(O2, key = names[i2++])) {
          ~indexOf(result, key) || push(result, key);
        }
        return result;
      };
    }
  });

  // node_modules/core-js-pure/internals/enum-bug-keys.js
  var require_enum_bug_keys = __commonJS({
    "node_modules/core-js-pure/internals/enum-bug-keys.js"(exports, module) {
      "use strict";
      module.exports = [
        "constructor",
        "hasOwnProperty",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toLocaleString",
        "toString",
        "valueOf"
      ];
    }
  });

  // node_modules/core-js-pure/internals/object-keys.js
  var require_object_keys = __commonJS({
    "node_modules/core-js-pure/internals/object-keys.js"(exports, module) {
      "use strict";
      var internalObjectKeys = require_object_keys_internal();
      var enumBugKeys = require_enum_bug_keys();
      module.exports = Object.keys || function keys(O2) {
        return internalObjectKeys(O2, enumBugKeys);
      };
    }
  });

  // node_modules/core-js-pure/internals/object-define-properties.js
  var require_object_define_properties = __commonJS({
    "node_modules/core-js-pure/internals/object-define-properties.js"(exports) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var V8_PROTOTYPE_DEFINE_BUG = require_v8_prototype_define_bug();
      var definePropertyModule = require_object_define_property();
      var anObject = require_an_object();
      var toIndexedObject = require_to_indexed_object();
      var objectKeys = require_object_keys();
      exports.f = DESCRIPTORS && !V8_PROTOTYPE_DEFINE_BUG ? Object.defineProperties : function defineProperties(O2, Properties) {
        anObject(O2);
        var props = toIndexedObject(Properties);
        var keys = objectKeys(Properties);
        var length = keys.length;
        var index = 0;
        var key;
        while (length > index) definePropertyModule.f(O2, key = keys[index++], props[key]);
        return O2;
      };
    }
  });

  // node_modules/core-js-pure/internals/html.js
  var require_html = __commonJS({
    "node_modules/core-js-pure/internals/html.js"(exports, module) {
      "use strict";
      var getBuiltIn = require_get_built_in();
      module.exports = getBuiltIn("document", "documentElement");
    }
  });

  // node_modules/core-js-pure/internals/object-create.js
  var require_object_create = __commonJS({
    "node_modules/core-js-pure/internals/object-create.js"(exports, module) {
      "use strict";
      var anObject = require_an_object();
      var definePropertiesModule = require_object_define_properties();
      var enumBugKeys = require_enum_bug_keys();
      var hiddenKeys = require_hidden_keys();
      var html = require_html();
      var documentCreateElement = require_document_create_element();
      var sharedKey = require_shared_key();
      var GT = ">";
      var LT = "<";
      var PROTOTYPE = "prototype";
      var SCRIPT = "script";
      var IE_PROTO = sharedKey("IE_PROTO");
      var EmptyConstructor = function() {
      };
      var scriptTag = function(content) {
        return LT + SCRIPT + GT + content + LT + "/" + SCRIPT + GT;
      };
      var NullProtoObjectViaActiveX = function(activeXDocument2) {
        activeXDocument2.write(scriptTag(""));
        activeXDocument2.close();
        var temp = activeXDocument2.parentWindow.Object;
        activeXDocument2 = null;
        return temp;
      };
      var NullProtoObjectViaIFrame = function() {
        var iframe = documentCreateElement("iframe");
        var JS = "java" + SCRIPT + ":";
        var iframeDocument;
        iframe.style.display = "none";
        html.appendChild(iframe);
        iframe.src = String(JS);
        iframeDocument = iframe.contentWindow.document;
        iframeDocument.open();
        iframeDocument.write(scriptTag("document.F=Object"));
        iframeDocument.close();
        return iframeDocument.F;
      };
      var activeXDocument;
      var NullProtoObject = function() {
        try {
          activeXDocument = new ActiveXObject("htmlfile");
        } catch (error) {
        }
        NullProtoObject = typeof document != "undefined" ? document.domain && activeXDocument ? NullProtoObjectViaActiveX(activeXDocument) : NullProtoObjectViaIFrame() : NullProtoObjectViaActiveX(activeXDocument);
        var length = enumBugKeys.length;
        while (length--) delete NullProtoObject[PROTOTYPE][enumBugKeys[length]];
        return NullProtoObject();
      };
      hiddenKeys[IE_PROTO] = true;
      module.exports = Object.create || function create(O2, Properties) {
        var result;
        if (O2 !== null) {
          EmptyConstructor[PROTOTYPE] = anObject(O2);
          result = new EmptyConstructor();
          EmptyConstructor[PROTOTYPE] = null;
          result[IE_PROTO] = O2;
        } else result = NullProtoObject();
        return Properties === void 0 ? result : definePropertiesModule.f(result, Properties);
      };
    }
  });

  // node_modules/core-js-pure/internals/correct-prototype-getter.js
  var require_correct_prototype_getter = __commonJS({
    "node_modules/core-js-pure/internals/correct-prototype-getter.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      module.exports = !fails(function() {
        function F2() {
        }
        F2.prototype.constructor = null;
        return Object.getPrototypeOf(new F2()) !== F2.prototype;
      });
    }
  });

  // node_modules/core-js-pure/internals/object-get-prototype-of.js
  var require_object_get_prototype_of = __commonJS({
    "node_modules/core-js-pure/internals/object-get-prototype-of.js"(exports, module) {
      "use strict";
      var hasOwn = require_has_own_property();
      var isCallable = require_is_callable();
      var toObject = require_to_object();
      var sharedKey = require_shared_key();
      var CORRECT_PROTOTYPE_GETTER = require_correct_prototype_getter();
      var IE_PROTO = sharedKey("IE_PROTO");
      var $Object = Object;
      var ObjectPrototype = $Object.prototype;
      module.exports = CORRECT_PROTOTYPE_GETTER ? $Object.getPrototypeOf : function(O2) {
        var object = toObject(O2);
        if (hasOwn(object, IE_PROTO)) return object[IE_PROTO];
        var constructor = object.constructor;
        if (isCallable(constructor) && object instanceof constructor) {
          return constructor.prototype;
        }
        return object instanceof $Object ? ObjectPrototype : null;
      };
    }
  });

  // node_modules/core-js-pure/internals/define-built-in.js
  var require_define_built_in = __commonJS({
    "node_modules/core-js-pure/internals/define-built-in.js"(exports, module) {
      "use strict";
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      module.exports = function(target, key, value, options) {
        if (options && options.enumerable) target[key] = value;
        else createNonEnumerableProperty(target, key, value);
        return target;
      };
    }
  });

  // node_modules/core-js-pure/internals/iterators-core.js
  var require_iterators_core = __commonJS({
    "node_modules/core-js-pure/internals/iterators-core.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      var isCallable = require_is_callable();
      var isObject = require_is_object();
      var create = require_object_create();
      var getPrototypeOf = require_object_get_prototype_of();
      var defineBuiltIn = require_define_built_in();
      var wellKnownSymbol = require_well_known_symbol();
      var IS_PURE = require_is_pure();
      var ITERATOR = wellKnownSymbol("iterator");
      var BUGGY_SAFARI_ITERATORS = false;
      var IteratorPrototype;
      var PrototypeOfArrayIteratorPrototype;
      var arrayIterator;
      if ([].keys) {
        arrayIterator = [].keys();
        if (!("next" in arrayIterator)) BUGGY_SAFARI_ITERATORS = true;
        else {
          PrototypeOfArrayIteratorPrototype = getPrototypeOf(getPrototypeOf(arrayIterator));
          if (PrototypeOfArrayIteratorPrototype !== Object.prototype) IteratorPrototype = PrototypeOfArrayIteratorPrototype;
        }
      }
      var NEW_ITERATOR_PROTOTYPE = !isObject(IteratorPrototype) || fails(function() {
        var test = {};
        return IteratorPrototype[ITERATOR].call(test) !== test;
      });
      if (NEW_ITERATOR_PROTOTYPE) IteratorPrototype = {};
      else if (IS_PURE) IteratorPrototype = create(IteratorPrototype);
      if (!isCallable(IteratorPrototype[ITERATOR])) {
        defineBuiltIn(IteratorPrototype, ITERATOR, function() {
          return this;
        });
      }
      module.exports = {
        IteratorPrototype,
        BUGGY_SAFARI_ITERATORS
      };
    }
  });

  // node_modules/core-js-pure/internals/to-string-tag-support.js
  var require_to_string_tag_support = __commonJS({
    "node_modules/core-js-pure/internals/to-string-tag-support.js"(exports, module) {
      "use strict";
      var wellKnownSymbol = require_well_known_symbol();
      var TO_STRING_TAG = wellKnownSymbol("toStringTag");
      var test = {};
      test[TO_STRING_TAG] = "z";
      module.exports = String(test) === "[object z]";
    }
  });

  // node_modules/core-js-pure/internals/classof.js
  var require_classof = __commonJS({
    "node_modules/core-js-pure/internals/classof.js"(exports, module) {
      "use strict";
      var TO_STRING_TAG_SUPPORT = require_to_string_tag_support();
      var isCallable = require_is_callable();
      var classofRaw = require_classof_raw();
      var wellKnownSymbol = require_well_known_symbol();
      var TO_STRING_TAG = wellKnownSymbol("toStringTag");
      var $Object = Object;
      var CORRECT_ARGUMENTS = classofRaw(/* @__PURE__ */ (function() {
        return arguments;
      })()) === "Arguments";
      var tryGet = function(it2, key) {
        try {
          return it2[key];
        } catch (error) {
        }
      };
      module.exports = TO_STRING_TAG_SUPPORT ? classofRaw : function(it2) {
        var O2, tag, result;
        return it2 === void 0 ? "Undefined" : it2 === null ? "Null" : typeof (tag = tryGet(O2 = $Object(it2), TO_STRING_TAG)) == "string" ? tag : CORRECT_ARGUMENTS ? classofRaw(O2) : (result = classofRaw(O2)) === "Object" && isCallable(O2.callee) ? "Arguments" : result;
      };
    }
  });

  // node_modules/core-js-pure/internals/object-to-string.js
  var require_object_to_string = __commonJS({
    "node_modules/core-js-pure/internals/object-to-string.js"(exports, module) {
      "use strict";
      var TO_STRING_TAG_SUPPORT = require_to_string_tag_support();
      var classof = require_classof();
      module.exports = TO_STRING_TAG_SUPPORT ? {}.toString : function toString() {
        return "[object " + classof(this) + "]";
      };
    }
  });

  // node_modules/core-js-pure/internals/set-to-string-tag.js
  var require_set_to_string_tag = __commonJS({
    "node_modules/core-js-pure/internals/set-to-string-tag.js"(exports, module) {
      "use strict";
      var TO_STRING_TAG_SUPPORT = require_to_string_tag_support();
      var defineProperty = require_object_define_property().f;
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      var hasOwn = require_has_own_property();
      var toString = require_object_to_string();
      var wellKnownSymbol = require_well_known_symbol();
      var TO_STRING_TAG = wellKnownSymbol("toStringTag");
      module.exports = function(it2, TAG, STATIC, SET_METHOD) {
        var target = STATIC ? it2 : it2 && it2.prototype;
        if (target) {
          if (!hasOwn(target, TO_STRING_TAG)) {
            defineProperty(target, TO_STRING_TAG, { configurable: true, value: TAG });
          }
          if (SET_METHOD && !TO_STRING_TAG_SUPPORT) {
            createNonEnumerableProperty(target, "toString", toString);
          }
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/iterator-create-constructor.js
  var require_iterator_create_constructor = __commonJS({
    "node_modules/core-js-pure/internals/iterator-create-constructor.js"(exports, module) {
      "use strict";
      var IteratorPrototype = require_iterators_core().IteratorPrototype;
      var create = require_object_create();
      var createPropertyDescriptor = require_create_property_descriptor();
      var setToStringTag = require_set_to_string_tag();
      var Iterators = require_iterators();
      var returnThis = function() {
        return this;
      };
      module.exports = function(IteratorConstructor, NAME, next, ENUMERABLE_NEXT) {
        var TO_STRING_TAG = NAME + " Iterator";
        IteratorConstructor.prototype = create(IteratorPrototype, { next: createPropertyDescriptor(+!ENUMERABLE_NEXT, next) });
        setToStringTag(IteratorConstructor, TO_STRING_TAG, false, true);
        Iterators[TO_STRING_TAG] = returnThis;
        return IteratorConstructor;
      };
    }
  });

  // node_modules/core-js-pure/internals/function-uncurry-this-accessor.js
  var require_function_uncurry_this_accessor = __commonJS({
    "node_modules/core-js-pure/internals/function-uncurry-this-accessor.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var aCallable = require_a_callable();
      module.exports = function(object, key, method) {
        try {
          return uncurryThis(aCallable(Object.getOwnPropertyDescriptor(object, key)[method]));
        } catch (error) {
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/is-possible-prototype.js
  var require_is_possible_prototype = __commonJS({
    "node_modules/core-js-pure/internals/is-possible-prototype.js"(exports, module) {
      "use strict";
      var isObject = require_is_object();
      module.exports = function(argument) {
        return isObject(argument) || argument === null;
      };
    }
  });

  // node_modules/core-js-pure/internals/a-possible-prototype.js
  var require_a_possible_prototype = __commonJS({
    "node_modules/core-js-pure/internals/a-possible-prototype.js"(exports, module) {
      "use strict";
      var isPossiblePrototype = require_is_possible_prototype();
      var $String = String;
      var $TypeError = TypeError;
      module.exports = function(argument) {
        if (isPossiblePrototype(argument)) return argument;
        throw new $TypeError("Can't set " + $String(argument) + " as a prototype");
      };
    }
  });

  // node_modules/core-js-pure/internals/object-set-prototype-of.js
  var require_object_set_prototype_of = __commonJS({
    "node_modules/core-js-pure/internals/object-set-prototype-of.js"(exports, module) {
      "use strict";
      var uncurryThisAccessor = require_function_uncurry_this_accessor();
      var isObject = require_is_object();
      var requireObjectCoercible = require_require_object_coercible();
      var aPossiblePrototype = require_a_possible_prototype();
      module.exports = Object.setPrototypeOf || ("__proto__" in {} ? (function() {
        var CORRECT_SETTER = false;
        var test = {};
        var setter;
        try {
          setter = uncurryThisAccessor(Object.prototype, "__proto__", "set");
          setter(test, []);
          CORRECT_SETTER = test instanceof Array;
        } catch (error) {
        }
        return function setPrototypeOf(O2, proto) {
          requireObjectCoercible(O2);
          aPossiblePrototype(proto);
          if (!isObject(O2)) return O2;
          if (CORRECT_SETTER) setter(O2, proto);
          else O2.__proto__ = proto;
          return O2;
        };
      })() : void 0);
    }
  });

  // node_modules/core-js-pure/internals/iterator-define.js
  var require_iterator_define = __commonJS({
    "node_modules/core-js-pure/internals/iterator-define.js"(exports, module) {
      "use strict";
      var $2 = require_export();
      var call = require_function_call();
      var IS_PURE = require_is_pure();
      var FunctionName = require_function_name();
      var isCallable = require_is_callable();
      var createIteratorConstructor = require_iterator_create_constructor();
      var getPrototypeOf = require_object_get_prototype_of();
      var setPrototypeOf = require_object_set_prototype_of();
      var setToStringTag = require_set_to_string_tag();
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      var defineBuiltIn = require_define_built_in();
      var wellKnownSymbol = require_well_known_symbol();
      var Iterators = require_iterators();
      var IteratorsCore = require_iterators_core();
      var PROPER_FUNCTION_NAME = FunctionName.PROPER;
      var CONFIGURABLE_FUNCTION_NAME = FunctionName.CONFIGURABLE;
      var IteratorPrototype = IteratorsCore.IteratorPrototype;
      var BUGGY_SAFARI_ITERATORS = IteratorsCore.BUGGY_SAFARI_ITERATORS;
      var ITERATOR = wellKnownSymbol("iterator");
      var KEYS = "keys";
      var VALUES = "values";
      var ENTRIES = "entries";
      var returnThis = function() {
        return this;
      };
      module.exports = function(Iterable, NAME, IteratorConstructor, next, DEFAULT, IS_SET, FORCED) {
        createIteratorConstructor(IteratorConstructor, NAME, next);
        var getIterationMethod = function(KIND) {
          if (KIND === DEFAULT && defaultIterator) return defaultIterator;
          if (!BUGGY_SAFARI_ITERATORS && KIND && KIND in IterablePrototype) return IterablePrototype[KIND];
          switch (KIND) {
            case KEYS:
              return function keys() {
                return new IteratorConstructor(this, KIND);
              };
            case VALUES:
              return function values() {
                return new IteratorConstructor(this, KIND);
              };
            case ENTRIES:
              return function entries() {
                return new IteratorConstructor(this, KIND);
              };
          }
          return function() {
            return new IteratorConstructor(this);
          };
        };
        var TO_STRING_TAG = NAME + " Iterator";
        var INCORRECT_VALUES_NAME = false;
        var IterablePrototype = Iterable.prototype;
        var nativeIterator = IterablePrototype[ITERATOR] || IterablePrototype["@@iterator"] || DEFAULT && IterablePrototype[DEFAULT];
        var defaultIterator = !BUGGY_SAFARI_ITERATORS && nativeIterator || getIterationMethod(DEFAULT);
        var anyNativeIterator = NAME === "Array" ? IterablePrototype.entries || nativeIterator : nativeIterator;
        var CurrentIteratorPrototype, methods, KEY;
        if (anyNativeIterator) {
          CurrentIteratorPrototype = getPrototypeOf(anyNativeIterator.call(new Iterable()));
          if (CurrentIteratorPrototype !== Object.prototype && CurrentIteratorPrototype.next) {
            if (!IS_PURE && getPrototypeOf(CurrentIteratorPrototype) !== IteratorPrototype) {
              if (setPrototypeOf) {
                setPrototypeOf(CurrentIteratorPrototype, IteratorPrototype);
              } else if (!isCallable(CurrentIteratorPrototype[ITERATOR])) {
                defineBuiltIn(CurrentIteratorPrototype, ITERATOR, returnThis);
              }
            }
            setToStringTag(CurrentIteratorPrototype, TO_STRING_TAG, true, true);
            if (IS_PURE) Iterators[TO_STRING_TAG] = returnThis;
          }
        }
        if (PROPER_FUNCTION_NAME && DEFAULT === VALUES && nativeIterator && nativeIterator.name !== VALUES) {
          if (!IS_PURE && CONFIGURABLE_FUNCTION_NAME) {
            createNonEnumerableProperty(IterablePrototype, "name", VALUES);
          } else {
            INCORRECT_VALUES_NAME = true;
            defaultIterator = function values() {
              return call(nativeIterator, this);
            };
          }
        }
        if (DEFAULT) {
          methods = {
            values: getIterationMethod(VALUES),
            keys: IS_SET ? defaultIterator : getIterationMethod(KEYS),
            entries: getIterationMethod(ENTRIES)
          };
          if (FORCED) for (KEY in methods) {
            if (BUGGY_SAFARI_ITERATORS || INCORRECT_VALUES_NAME || !(KEY in IterablePrototype)) {
              defineBuiltIn(IterablePrototype, KEY, methods[KEY]);
            }
          }
          else $2({ target: NAME, proto: true, forced: BUGGY_SAFARI_ITERATORS || INCORRECT_VALUES_NAME }, methods);
        }
        if ((!IS_PURE || FORCED) && IterablePrototype[ITERATOR] !== defaultIterator) {
          defineBuiltIn(IterablePrototype, ITERATOR, defaultIterator, { name: DEFAULT });
        }
        Iterators[NAME] = defaultIterator;
        return methods;
      };
    }
  });

  // node_modules/core-js-pure/internals/create-iter-result-object.js
  var require_create_iter_result_object = __commonJS({
    "node_modules/core-js-pure/internals/create-iter-result-object.js"(exports, module) {
      "use strict";
      module.exports = function(value, done) {
        return { value, done };
      };
    }
  });

  // node_modules/core-js-pure/modules/es.array.iterator.js
  var require_es_array_iterator = __commonJS({
    "node_modules/core-js-pure/modules/es.array.iterator.js"(exports, module) {
      "use strict";
      var toIndexedObject = require_to_indexed_object();
      var addToUnscopables = require_add_to_unscopables();
      var Iterators = require_iterators();
      var InternalStateModule = require_internal_state();
      var defineProperty = require_object_define_property().f;
      var defineIterator = require_iterator_define();
      var createIterResultObject = require_create_iter_result_object();
      var IS_PURE = require_is_pure();
      var DESCRIPTORS = require_descriptors();
      var ARRAY_ITERATOR = "Array Iterator";
      var setInternalState = InternalStateModule.set;
      var getInternalState = InternalStateModule.getterFor(ARRAY_ITERATOR);
      module.exports = defineIterator(Array, "Array", function(iterated, kind) {
        setInternalState(this, {
          type: ARRAY_ITERATOR,
          target: toIndexedObject(iterated),
          // target
          index: 0,
          // next index
          kind
          // kind
        });
      }, function() {
        var state = getInternalState(this);
        var target = state.target;
        var index = state.index++;
        if (!target || index >= target.length) {
          state.target = null;
          return createIterResultObject(void 0, true);
        }
        switch (state.kind) {
          case "keys":
            return createIterResultObject(index, false);
          case "values":
            return createIterResultObject(target[index], false);
        }
        return createIterResultObject([index, target[index]], false);
      }, "values");
      var values = Iterators.Arguments = Iterators.Array;
      addToUnscopables("keys");
      addToUnscopables("values");
      addToUnscopables("entries");
      if (!IS_PURE && DESCRIPTORS && values.name !== "values") try {
        defineProperty(values, "name", { value: "values" });
      } catch (error) {
      }
    }
  });

  // node_modules/core-js-pure/modules/es.object.keys.js
  var require_es_object_keys = __commonJS({
    "node_modules/core-js-pure/modules/es.object.keys.js"() {
      "use strict";
      var $2 = require_export();
      var toObject = require_to_object();
      var nativeKeys = require_object_keys();
      var fails = require_fails();
      var FAILS_ON_PRIMITIVES = fails(function() {
        nativeKeys(1);
      });
      $2({ target: "Object", stat: true, forced: FAILS_ON_PRIMITIVES }, {
        keys: function keys(it2) {
          return nativeKeys(toObject(it2));
        }
      });
    }
  });

  // node_modules/core-js-pure/modules/es.object.to-string.js
  var require_es_object_to_string = __commonJS({
    "node_modules/core-js-pure/modules/es.object.to-string.js"() {
    }
  });

  // node_modules/core-js-pure/internals/object-get-own-property-names.js
  var require_object_get_own_property_names = __commonJS({
    "node_modules/core-js-pure/internals/object-get-own-property-names.js"(exports) {
      "use strict";
      var internalObjectKeys = require_object_keys_internal();
      var enumBugKeys = require_enum_bug_keys();
      var hiddenKeys = enumBugKeys.concat("length", "prototype");
      exports.f = Object.getOwnPropertyNames || function getOwnPropertyNames(O2) {
        return internalObjectKeys(O2, hiddenKeys);
      };
    }
  });

  // node_modules/core-js-pure/internals/array-slice.js
  var require_array_slice = __commonJS({
    "node_modules/core-js-pure/internals/array-slice.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      module.exports = uncurryThis([].slice);
    }
  });

  // node_modules/core-js-pure/internals/object-get-own-property-names-external.js
  var require_object_get_own_property_names_external = __commonJS({
    "node_modules/core-js-pure/internals/object-get-own-property-names-external.js"(exports, module) {
      "use strict";
      var classof = require_classof_raw();
      var toIndexedObject = require_to_indexed_object();
      var $getOwnPropertyNames = require_object_get_own_property_names().f;
      var arraySlice = require_array_slice();
      var windowNames = typeof window == "object" && window && Object.getOwnPropertyNames ? Object.getOwnPropertyNames(window) : [];
      var getWindowNames = function(it2) {
        try {
          return $getOwnPropertyNames(it2);
        } catch (error) {
          return arraySlice(windowNames);
        }
      };
      module.exports.f = function getOwnPropertyNames(it2) {
        return windowNames && classof(it2) === "Window" ? getWindowNames(it2) : $getOwnPropertyNames(toIndexedObject(it2));
      };
    }
  });

  // node_modules/core-js-pure/internals/array-buffer-non-extensible.js
  var require_array_buffer_non_extensible = __commonJS({
    "node_modules/core-js-pure/internals/array-buffer-non-extensible.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      module.exports = fails(function() {
        if (typeof ArrayBuffer == "function") {
          var buffer = new ArrayBuffer(8);
          if (Object.isExtensible(buffer)) Object.defineProperty(buffer, "a", { value: 8 });
        }
      });
    }
  });

  // node_modules/core-js-pure/internals/object-is-extensible.js
  var require_object_is_extensible = __commonJS({
    "node_modules/core-js-pure/internals/object-is-extensible.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      var isObject = require_is_object();
      var classof = require_classof_raw();
      var ARRAY_BUFFER_NON_EXTENSIBLE = require_array_buffer_non_extensible();
      var $isExtensible = Object.isExtensible;
      var FAILS_ON_PRIMITIVES = fails(function() {
        $isExtensible(1);
      });
      module.exports = FAILS_ON_PRIMITIVES || ARRAY_BUFFER_NON_EXTENSIBLE ? function isExtensible(it2) {
        if (!isObject(it2)) return false;
        if (ARRAY_BUFFER_NON_EXTENSIBLE && classof(it2) === "ArrayBuffer") return false;
        return $isExtensible ? $isExtensible(it2) : true;
      } : $isExtensible;
    }
  });

  // node_modules/core-js-pure/internals/freezing.js
  var require_freezing = __commonJS({
    "node_modules/core-js-pure/internals/freezing.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      module.exports = !fails(function() {
        return Object.isExtensible(Object.preventExtensions({}));
      });
    }
  });

  // node_modules/core-js-pure/internals/internal-metadata.js
  var require_internal_metadata = __commonJS({
    "node_modules/core-js-pure/internals/internal-metadata.js"(exports, module) {
      "use strict";
      var $2 = require_export();
      var uncurryThis = require_function_uncurry_this();
      var hiddenKeys = require_hidden_keys();
      var isObject = require_is_object();
      var hasOwn = require_has_own_property();
      var defineProperty = require_object_define_property().f;
      var getOwnPropertyNamesModule = require_object_get_own_property_names();
      var getOwnPropertyNamesExternalModule = require_object_get_own_property_names_external();
      var isExtensible = require_object_is_extensible();
      var uid = require_uid();
      var FREEZING = require_freezing();
      var REQUIRED = false;
      var METADATA = uid("meta");
      var id = 0;
      var setMetadata = function(it2) {
        defineProperty(it2, METADATA, { value: {
          objectID: "O" + id++,
          // object ID
          weakData: {}
          // weak collections IDs
        } });
      };
      var fastKey = function(it2, create) {
        if (!isObject(it2)) return typeof it2 == "symbol" ? it2 : (typeof it2 == "string" ? "S" : "P") + it2;
        if (!hasOwn(it2, METADATA)) {
          if (!isExtensible(it2)) return "F";
          if (!create) return "E";
          setMetadata(it2);
        }
        return it2[METADATA].objectID;
      };
      var getWeakData = function(it2, create) {
        if (!hasOwn(it2, METADATA)) {
          if (!isExtensible(it2)) return true;
          if (!create) return false;
          setMetadata(it2);
        }
        return it2[METADATA].weakData;
      };
      var onFreeze = function(it2) {
        if (FREEZING && REQUIRED && isExtensible(it2) && !hasOwn(it2, METADATA)) setMetadata(it2);
        return it2;
      };
      var enable = function() {
        meta.enable = function() {
        };
        REQUIRED = true;
        var getOwnPropertyNames = getOwnPropertyNamesModule.f;
        var splice = uncurryThis([].splice);
        var test = {};
        test[METADATA] = 1;
        if (getOwnPropertyNames(test).length) {
          getOwnPropertyNamesModule.f = function(it2) {
            var result = getOwnPropertyNames(it2);
            for (var i2 = 0, length = result.length; i2 < length; i2++) {
              if (result[i2] === METADATA) {
                splice(result, i2, 1);
                break;
              }
            }
            return result;
          };
          $2({ target: "Object", stat: true, forced: true }, {
            getOwnPropertyNames: getOwnPropertyNamesExternalModule.f
          });
        }
      };
      var meta = module.exports = {
        enable,
        fastKey,
        getWeakData,
        onFreeze
      };
      hiddenKeys[METADATA] = true;
    }
  });

  // node_modules/core-js-pure/internals/is-array-iterator-method.js
  var require_is_array_iterator_method = __commonJS({
    "node_modules/core-js-pure/internals/is-array-iterator-method.js"(exports, module) {
      "use strict";
      var wellKnownSymbol = require_well_known_symbol();
      var Iterators = require_iterators();
      var ITERATOR = wellKnownSymbol("iterator");
      var ArrayPrototype = Array.prototype;
      module.exports = function(it2) {
        return it2 !== void 0 && (Iterators.Array === it2 || ArrayPrototype[ITERATOR] === it2);
      };
    }
  });

  // node_modules/core-js-pure/internals/get-internal-iterator.js
  var require_get_internal_iterator = __commonJS({
    "node_modules/core-js-pure/internals/get-internal-iterator.js"(exports, module) {
      "use strict";
      var classof = require_classof();
      var hasOwn = require_has_own_property();
      var Iterators = require_iterators();
      module.exports = function(it2) {
        var klass = classof(it2);
        if (klass === "Arguments") klass = "Array";
        return hasOwn(Iterators, klass) ? Iterators[klass] : void 0;
      };
    }
  });

  // node_modules/core-js-pure/internals/get-iterator-method-internal.js
  var require_get_iterator_method_internal = __commonJS({
    "node_modules/core-js-pure/internals/get-iterator-method-internal.js"(exports, module) {
      "use strict";
      var getMethod = require_get_method();
      var isNullOrUndefined = require_is_null_or_undefined();
      var getInternalIterator = require_get_internal_iterator();
      var wellKnownSymbol = require_well_known_symbol();
      var ITERATOR = wellKnownSymbol("iterator");
      module.exports = function(it2) {
        if (!isNullOrUndefined(it2)) return getMethod(it2, ITERATOR) || getMethod(it2, "@@iterator") || getInternalIterator(it2);
      };
    }
  });

  // node_modules/core-js-pure/internals/get-iterator-internal.js
  var require_get_iterator_internal = __commonJS({
    "node_modules/core-js-pure/internals/get-iterator-internal.js"(exports, module) {
      "use strict";
      var call = require_function_call();
      var isCallable = require_is_callable();
      var anObject = require_an_object();
      var tryToString = require_try_to_string();
      var getIteratorMethod = require_get_iterator_method_internal();
      var $TypeError = TypeError;
      module.exports = function(argument, usingIterator) {
        var iteratorMethod = arguments.length < 2 ? getIteratorMethod(argument) : usingIterator;
        if (isCallable(iteratorMethod)) return anObject(call(iteratorMethod, argument));
        throw new $TypeError(tryToString(argument) + " is not iterable");
      };
    }
  });

  // node_modules/core-js-pure/internals/iterator-close.js
  var require_iterator_close = __commonJS({
    "node_modules/core-js-pure/internals/iterator-close.js"(exports, module) {
      "use strict";
      var call = require_function_call();
      var anObject = require_an_object();
      var getMethod = require_get_method();
      module.exports = function(iterator, kind, value) {
        var innerResult, innerError;
        anObject(iterator);
        try {
          innerResult = getMethod(iterator, "return");
          if (!innerResult) {
            if (kind === "throw") throw value;
            return value;
          }
          innerResult = call(innerResult, iterator);
        } catch (error) {
          innerError = true;
          innerResult = error;
        }
        if (kind === "throw") throw value;
        if (innerError) throw innerResult;
        anObject(innerResult);
        return value;
      };
    }
  });

  // node_modules/core-js-pure/internals/iterate.js
  var require_iterate = __commonJS({
    "node_modules/core-js-pure/internals/iterate.js"(exports, module) {
      "use strict";
      var bind = require_function_bind_context();
      var call = require_function_call();
      var anObject = require_an_object();
      var tryToString = require_try_to_string();
      var isArrayIteratorMethod = require_is_array_iterator_method();
      var lengthOfArrayLike = require_length_of_array_like();
      var isPrototypeOf = require_object_is_prototype_of();
      var getIterator = require_get_iterator_internal();
      var getIteratorMethod = require_get_iterator_method_internal();
      var iteratorClose = require_iterator_close();
      var $TypeError = TypeError;
      var Result = function(stopped, result) {
        this.stopped = stopped;
        this.result = result;
      };
      var ResultPrototype = Result.prototype;
      module.exports = function(iterable, unboundFunction, options) {
        var that = options && options.that;
        var AS_ENTRIES = !!(options && options.AS_ENTRIES);
        var IS_RECORD = !!(options && options.IS_RECORD);
        var IS_ITERATOR = !!(options && options.IS_ITERATOR);
        var INTERRUPTED = !!(options && options.INTERRUPTED);
        var fn2 = bind(unboundFunction, that);
        var iterator, iterFn, index, length, result, next, step;
        var stop = function(condition) {
          var $iterator = iterator;
          iterator = void 0;
          if ($iterator) iteratorClose($iterator, "normal");
          return new Result(true, condition);
        };
        var callFn = function(value2) {
          if (AS_ENTRIES) {
            anObject(value2);
            return INTERRUPTED ? fn2(value2[0], value2[1], stop) : fn2(value2[0], value2[1]);
          }
          return INTERRUPTED ? fn2(value2, stop) : fn2(value2);
        };
        if (IS_RECORD) {
          iterator = iterable.iterator;
        } else if (IS_ITERATOR) {
          iterator = iterable;
        } else {
          iterFn = getIteratorMethod(iterable);
          if (!iterFn) throw new $TypeError(tryToString(iterable) + " is not iterable");
          if (isArrayIteratorMethod(iterFn)) {
            for (index = 0, length = lengthOfArrayLike(iterable); length > index; index++) {
              result = callFn(iterable[index]);
              if (result && isPrototypeOf(ResultPrototype, result)) return result;
            }
            return new Result(false);
          }
          iterator = getIterator(iterable, iterFn);
        }
        next = IS_RECORD ? iterable.next : iterator.next;
        while (!(step = call(next, iterator)).done) {
          var value = step.value;
          try {
            result = callFn(value);
          } catch (error) {
            if (iterator) iteratorClose(iterator, "throw", error);
            else throw error;
          }
          if (typeof result == "object" && result && isPrototypeOf(ResultPrototype, result)) return result;
        }
        return new Result(false);
      };
    }
  });

  // node_modules/core-js-pure/internals/an-instance.js
  var require_an_instance = __commonJS({
    "node_modules/core-js-pure/internals/an-instance.js"(exports, module) {
      "use strict";
      var isPrototypeOf = require_object_is_prototype_of();
      var $TypeError = TypeError;
      module.exports = function(it2, Prototype) {
        if (isPrototypeOf(Prototype, it2)) return it2;
        throw new $TypeError("Incorrect invocation");
      };
    }
  });

  // node_modules/core-js-pure/internals/is-array.js
  var require_is_array = __commonJS({
    "node_modules/core-js-pure/internals/is-array.js"(exports, module) {
      "use strict";
      var classof = require_classof_raw();
      module.exports = Array.isArray || function isArray(argument) {
        return classof(argument) === "Array";
      };
    }
  });

  // node_modules/core-js-pure/internals/inspect-source.js
  var require_inspect_source = __commonJS({
    "node_modules/core-js-pure/internals/inspect-source.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var isCallable = require_is_callable();
      var store = require_shared_store();
      var functionToString = uncurryThis(Function.toString);
      if (!isCallable(store.inspectSource)) {
        store.inspectSource = function(it2) {
          return functionToString(it2);
        };
      }
      module.exports = store.inspectSource;
    }
  });

  // node_modules/core-js-pure/internals/is-constructor.js
  var require_is_constructor = __commonJS({
    "node_modules/core-js-pure/internals/is-constructor.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var fails = require_fails();
      var isCallable = require_is_callable();
      var classof = require_classof();
      var getBuiltIn = require_get_built_in();
      var inspectSource = require_inspect_source();
      var noop = function() {
      };
      var construct = getBuiltIn("Reflect", "construct");
      var constructorRegExp = /^\s*(?:class|function)\b/;
      var exec = uncurryThis(constructorRegExp.exec);
      var INCORRECT_TO_STRING = !constructorRegExp.test(noop);
      var isConstructorModern = function isConstructor(argument) {
        if (!isCallable(argument)) return false;
        try {
          construct(noop, [], argument);
          return true;
        } catch (error) {
          return false;
        }
      };
      var isConstructorLegacy = function isConstructor(argument) {
        if (!isCallable(argument)) return false;
        switch (classof(argument)) {
          case "AsyncFunction":
          case "GeneratorFunction":
          case "AsyncGeneratorFunction":
            return false;
        }
        try {
          return INCORRECT_TO_STRING || !!exec(constructorRegExp, inspectSource(argument));
        } catch (error) {
          return true;
        }
      };
      isConstructorLegacy.sham = true;
      module.exports = !construct || fails(function() {
        var called;
        return isConstructorModern(isConstructorModern.call) || !isConstructorModern(Object) || !isConstructorModern(function() {
          called = true;
        }) || called;
      }) ? isConstructorLegacy : isConstructorModern;
    }
  });

  // node_modules/core-js-pure/internals/array-species-constructor.js
  var require_array_species_constructor = __commonJS({
    "node_modules/core-js-pure/internals/array-species-constructor.js"(exports, module) {
      "use strict";
      var isArray = require_is_array();
      var isConstructor = require_is_constructor();
      var isObject = require_is_object();
      var wellKnownSymbol = require_well_known_symbol();
      var SPECIES = wellKnownSymbol("species");
      var $Array = Array;
      module.exports = function(originalArray) {
        var C2;
        if (isArray(originalArray)) {
          C2 = originalArray.constructor;
          if (isConstructor(C2) && (C2 === $Array || isArray(C2.prototype))) C2 = void 0;
          else if (isObject(C2)) {
            C2 = C2[SPECIES];
            if (C2 === null) C2 = void 0;
          }
        }
        return C2 === void 0 ? $Array : C2;
      };
    }
  });

  // node_modules/core-js-pure/internals/array-species-create.js
  var require_array_species_create = __commonJS({
    "node_modules/core-js-pure/internals/array-species-create.js"(exports, module) {
      "use strict";
      var arraySpeciesConstructor = require_array_species_constructor();
      module.exports = function(originalArray, length) {
        return new (arraySpeciesConstructor(originalArray))(length === 0 ? 0 : length);
      };
    }
  });

  // node_modules/core-js-pure/internals/create-property.js
  var require_create_property = __commonJS({
    "node_modules/core-js-pure/internals/create-property.js"(exports, module) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var definePropertyModule = require_object_define_property();
      var createPropertyDescriptor = require_create_property_descriptor();
      module.exports = function(object, key, value) {
        if (DESCRIPTORS) definePropertyModule.f(object, key, createPropertyDescriptor(0, value));
        else object[key] = value;
      };
    }
  });

  // node_modules/core-js-pure/internals/array-iteration.js
  var require_array_iteration = __commonJS({
    "node_modules/core-js-pure/internals/array-iteration.js"(exports, module) {
      "use strict";
      var bind = require_function_bind_context();
      var IndexedObject = require_indexed_object();
      var toObject = require_to_object();
      var lengthOfArrayLike = require_length_of_array_like();
      var arraySpeciesCreate = require_array_species_create();
      var createProperty = require_create_property();
      var createMethod = function(TYPE) {
        var IS_MAP = TYPE === 1;
        var IS_FILTER = TYPE === 2;
        var IS_SOME = TYPE === 3;
        var IS_EVERY = TYPE === 4;
        var IS_FIND_INDEX = TYPE === 6;
        var IS_FILTER_REJECT = TYPE === 7;
        var NO_HOLES = TYPE === 5 || IS_FIND_INDEX;
        return function($this, callbackfn, that) {
          var O2 = toObject($this);
          var self2 = IndexedObject(O2);
          var length = lengthOfArrayLike(self2);
          var boundFunction = bind(callbackfn, that);
          var index = 0;
          var resIndex = 0;
          var target = IS_MAP ? arraySpeciesCreate($this, length) : IS_FILTER || IS_FILTER_REJECT ? arraySpeciesCreate($this, 0) : void 0;
          var value, result;
          for (; length > index; index++) if (NO_HOLES || index in self2) {
            value = self2[index];
            result = boundFunction(value, index, O2);
            if (TYPE) {
              if (IS_MAP) createProperty(target, index, result);
              else if (result) switch (TYPE) {
                case 3:
                  return true;
                // some
                case 5:
                  return value;
                // find
                case 6:
                  return index;
                // findIndex
                case 2:
                  createProperty(target, resIndex++, value);
              }
              else switch (TYPE) {
                case 4:
                  return false;
                // every
                case 7:
                  createProperty(target, resIndex++, value);
              }
            }
          }
          return IS_FIND_INDEX ? -1 : IS_SOME || IS_EVERY ? IS_EVERY : target;
        };
      };
      module.exports = {
        // `Array.prototype.forEach` method
        // https://tc39.es/ecma262/#sec-array.prototype.foreach
        forEach: createMethod(0),
        // `Array.prototype.map` method
        // https://tc39.es/ecma262/#sec-array.prototype.map
        map: createMethod(1),
        // `Array.prototype.filter` method
        // https://tc39.es/ecma262/#sec-array.prototype.filter
        filter: createMethod(2),
        // `Array.prototype.some` method
        // https://tc39.es/ecma262/#sec-array.prototype.some
        some: createMethod(3),
        // `Array.prototype.every` method
        // https://tc39.es/ecma262/#sec-array.prototype.every
        every: createMethod(4),
        // `Array.prototype.find` method
        // https://tc39.es/ecma262/#sec-array.prototype.find
        find: createMethod(5),
        // `Array.prototype.findIndex` method
        // https://tc39.es/ecma262/#sec-array.prototype.findIndex
        findIndex: createMethod(6),
        // `Array.prototype.filterReject` method
        // https://github.com/tc39/proposal-array-filtering
        filterReject: createMethod(7)
      };
    }
  });

  // node_modules/core-js-pure/internals/collection.js
  var require_collection = __commonJS({
    "node_modules/core-js-pure/internals/collection.js"(exports, module) {
      "use strict";
      var $2 = require_export();
      var globalThis2 = require_global_this();
      var InternalMetadataModule = require_internal_metadata();
      var call = require_function_call();
      var fails = require_fails();
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      var iterate = require_iterate();
      var anInstance = require_an_instance();
      var isCallable = require_is_callable();
      var isObject = require_is_object();
      var isNullOrUndefined = require_is_null_or_undefined();
      var setToStringTag = require_set_to_string_tag();
      var defineProperty = require_object_define_property().f;
      var forEach = require_array_iteration().forEach;
      var DESCRIPTORS = require_descriptors();
      var InternalStateModule = require_internal_state();
      var setInternalState = InternalStateModule.set;
      var internalStateGetterFor = InternalStateModule.getterFor;
      module.exports = function(CONSTRUCTOR_NAME, wrapper, common) {
        var IS_MAP = CONSTRUCTOR_NAME.indexOf("Map") !== -1;
        var IS_WEAK = CONSTRUCTOR_NAME.indexOf("Weak") !== -1;
        var ADDER = IS_MAP ? "set" : "add";
        var NativeConstructor = globalThis2[CONSTRUCTOR_NAME];
        var NativePrototype = NativeConstructor && NativeConstructor.prototype;
        var exported = {};
        var Constructor;
        if (!DESCRIPTORS || !isCallable(NativeConstructor) || !(IS_WEAK || NativePrototype.forEach && !fails(function() {
          new NativeConstructor().entries().next();
        }))) {
          Constructor = common.getConstructor(wrapper, CONSTRUCTOR_NAME, IS_MAP, ADDER);
          InternalMetadataModule.enable();
        } else {
          Constructor = wrapper(function(target, iterable) {
            setInternalState(anInstance(target, Prototype), {
              type: CONSTRUCTOR_NAME,
              collection: new NativeConstructor()
            });
            if (!isNullOrUndefined(iterable)) iterate(iterable, target[ADDER], { that: target, AS_ENTRIES: IS_MAP });
          });
          var Prototype = Constructor.prototype;
          var getInternalState = internalStateGetterFor(CONSTRUCTOR_NAME);
          forEach(["add", "clear", "delete", "forEach", "get", "has", "set", "keys", "values", "entries"], function(KEY) {
            var IS_ADDER = KEY === "add" || KEY === "set";
            if (KEY in NativePrototype && !(IS_WEAK && KEY === "clear")) {
              createNonEnumerableProperty(Prototype, KEY, function(a2, b2) {
                var that = this;
                var collection = getInternalState(that).collection;
                if (!IS_ADDER && IS_WEAK && !isObject(a2)) return KEY === "get" ? void 0 : false;
                var result = collection[KEY](KEY === "forEach" ? function(value, key) {
                  call(a2, b2, value, key, that);
                } : a2 === 0 ? 0 : a2, b2);
                return IS_ADDER ? that : result;
              });
            }
          });
          IS_WEAK || defineProperty(Prototype, "size", {
            configurable: true,
            get: function() {
              return getInternalState(this).collection.size;
            }
          });
        }
        setToStringTag(Constructor, CONSTRUCTOR_NAME, false, true);
        exported[CONSTRUCTOR_NAME] = Constructor;
        $2({ global: true, forced: true }, exported);
        if (!IS_WEAK) common.setStrong(Constructor, CONSTRUCTOR_NAME, IS_MAP);
        return Constructor;
      };
    }
  });

  // node_modules/core-js-pure/internals/define-built-in-accessor.js
  var require_define_built_in_accessor = __commonJS({
    "node_modules/core-js-pure/internals/define-built-in-accessor.js"(exports, module) {
      "use strict";
      var defineProperty = require_object_define_property();
      module.exports = function(target, name2, descriptor) {
        return defineProperty.f(target, name2, descriptor);
      };
    }
  });

  // node_modules/core-js-pure/internals/define-built-ins.js
  var require_define_built_ins = __commonJS({
    "node_modules/core-js-pure/internals/define-built-ins.js"(exports, module) {
      "use strict";
      var defineBuiltIn = require_define_built_in();
      module.exports = function(target, src, options) {
        for (var key in src) {
          if (options && options.unsafe && target[key]) target[key] = src[key];
          else defineBuiltIn(target, key, src[key], options);
        }
        return target;
      };
    }
  });

  // node_modules/core-js-pure/internals/set-species.js
  var require_set_species = __commonJS({
    "node_modules/core-js-pure/internals/set-species.js"(exports, module) {
      "use strict";
      var getBuiltIn = require_get_built_in();
      var defineBuiltInAccessor = require_define_built_in_accessor();
      var wellKnownSymbol = require_well_known_symbol();
      var DESCRIPTORS = require_descriptors();
      var SPECIES = wellKnownSymbol("species");
      module.exports = function(CONSTRUCTOR_NAME) {
        var Constructor = getBuiltIn(CONSTRUCTOR_NAME);
        if (DESCRIPTORS && Constructor && !Constructor[SPECIES]) {
          defineBuiltInAccessor(Constructor, SPECIES, {
            configurable: true,
            get: function() {
              return this;
            }
          });
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/collection-strong.js
  var require_collection_strong = __commonJS({
    "node_modules/core-js-pure/internals/collection-strong.js"(exports, module) {
      "use strict";
      var create = require_object_create();
      var defineBuiltInAccessor = require_define_built_in_accessor();
      var defineBuiltIns = require_define_built_ins();
      var bind = require_function_bind_context();
      var anInstance = require_an_instance();
      var isNullOrUndefined = require_is_null_or_undefined();
      var iterate = require_iterate();
      var defineIterator = require_iterator_define();
      var createIterResultObject = require_create_iter_result_object();
      var setSpecies = require_set_species();
      var DESCRIPTORS = require_descriptors();
      var fastKey = require_internal_metadata().fastKey;
      var InternalStateModule = require_internal_state();
      var setInternalState = InternalStateModule.set;
      var internalStateGetterFor = InternalStateModule.getterFor;
      module.exports = {
        getConstructor: function(wrapper, CONSTRUCTOR_NAME, IS_MAP, ADDER) {
          var Constructor = wrapper(function(that, iterable) {
            anInstance(that, Prototype);
            setInternalState(that, {
              type: CONSTRUCTOR_NAME,
              index: create(null),
              first: null,
              last: null,
              size: 0
            });
            if (!DESCRIPTORS) that.size = 0;
            if (!isNullOrUndefined(iterable)) iterate(iterable, that[ADDER], { that, AS_ENTRIES: IS_MAP });
          });
          var Prototype = Constructor.prototype;
          var getInternalState = internalStateGetterFor(CONSTRUCTOR_NAME);
          var define = function(that, key, value) {
            var state = getInternalState(that);
            var entry = getEntry(that, key);
            var previous, index;
            if (entry) {
              entry.value = value;
            } else {
              state.last = entry = {
                index: index = fastKey(key, true),
                key,
                value,
                previous: previous = state.last,
                next: null,
                removed: false
              };
              if (!state.first) state.first = entry;
              if (previous) previous.next = entry;
              if (DESCRIPTORS) state.size++;
              else that.size++;
              if (index !== "F") state.index[index] = entry;
            }
            return that;
          };
          var getEntry = function(that, key) {
            var state = getInternalState(that);
            var index = fastKey(key);
            var entry;
            if (index !== "F") return state.index[index];
            for (entry = state.first; entry; entry = entry.next) {
              if (entry.key === key) return entry;
            }
          };
          defineBuiltIns(Prototype, {
            // `{ Map, Set }.prototype.clear()` methods
            // https://tc39.es/ecma262/#sec-map.prototype.clear
            // https://tc39.es/ecma262/#sec-set.prototype.clear
            clear: function clear() {
              var that = this;
              var state = getInternalState(that);
              var entry = state.first;
              while (entry) {
                entry.removed = true;
                if (entry.previous) entry.previous = entry.previous.next = null;
                entry = entry.next;
              }
              state.first = state.last = null;
              state.index = create(null);
              if (DESCRIPTORS) state.size = 0;
              else that.size = 0;
            },
            // `{ Map, Set }.prototype.delete(key)` methods
            // https://tc39.es/ecma262/#sec-map.prototype.delete
            // https://tc39.es/ecma262/#sec-set.prototype.delete
            "delete": function(key) {
              var that = this;
              var state = getInternalState(that);
              var entry = getEntry(that, key);
              if (entry) {
                var next = entry.next;
                var prev = entry.previous;
                delete state.index[entry.index];
                entry.removed = true;
                if (prev) prev.next = next;
                if (next) next.previous = prev;
                if (state.first === entry) state.first = next;
                if (state.last === entry) state.last = prev;
                if (DESCRIPTORS) state.size--;
                else that.size--;
              }
              return !!entry;
            },
            // `{ Map, Set }.prototype.forEach(callbackfn, thisArg = undefined)` methods
            // https://tc39.es/ecma262/#sec-map.prototype.foreach
            // https://tc39.es/ecma262/#sec-set.prototype.foreach
            forEach: function forEach(callbackfn) {
              var state = getInternalState(this);
              var boundFunction = bind(callbackfn, arguments.length > 1 ? arguments[1] : void 0);
              var entry;
              while (entry = entry ? entry.next : state.first) {
                boundFunction(entry.value, entry.key, this);
                while (entry && entry.removed) entry = entry.previous;
              }
            },
            // `{ Map, Set}.prototype.has(key)` methods
            // https://tc39.es/ecma262/#sec-map.prototype.has
            // https://tc39.es/ecma262/#sec-set.prototype.has
            has: function has(key) {
              return !!getEntry(this, key);
            }
          });
          defineBuiltIns(Prototype, IS_MAP ? {
            // `Map.prototype.get(key)` method
            // https://tc39.es/ecma262/#sec-map.prototype.get
            get: function get(key) {
              var entry = getEntry(this, key);
              return entry && entry.value;
            },
            // `Map.prototype.set(key, value)` method
            // https://tc39.es/ecma262/#sec-map.prototype.set
            set: function set(key, value) {
              return define(this, key === 0 ? 0 : key, value);
            }
          } : {
            // `Set.prototype.add(value)` method
            // https://tc39.es/ecma262/#sec-set.prototype.add
            add: function add(value) {
              return define(this, value = value === 0 ? 0 : value, value);
            }
          });
          if (DESCRIPTORS) defineBuiltInAccessor(Prototype, "size", {
            configurable: true,
            get: function() {
              return getInternalState(this).size;
            }
          });
          return Constructor;
        },
        setStrong: function(Constructor, CONSTRUCTOR_NAME, IS_MAP) {
          var ITERATOR_NAME = CONSTRUCTOR_NAME + " Iterator";
          var getInternalCollectionState = internalStateGetterFor(CONSTRUCTOR_NAME);
          var getInternalIteratorState = internalStateGetterFor(ITERATOR_NAME);
          defineIterator(Constructor, CONSTRUCTOR_NAME, function(iterated, kind) {
            setInternalState(this, {
              type: ITERATOR_NAME,
              target: iterated,
              state: getInternalCollectionState(iterated),
              kind,
              last: null
            });
          }, function() {
            var state = getInternalIteratorState(this);
            var kind = state.kind;
            var entry = state.last;
            while (entry && entry.removed) entry = entry.previous;
            if (!state.target || !(state.last = entry = entry ? entry.next : state.state.first)) {
              state.target = null;
              return createIterResultObject(void 0, true);
            }
            if (kind === "keys") return createIterResultObject(entry.key, false);
            if (kind === "values") return createIterResultObject(entry.value, false);
            return createIterResultObject([entry.key, entry.value], false);
          }, IS_MAP ? "entries" : "values", !IS_MAP, true);
          setSpecies(CONSTRUCTOR_NAME);
        }
      };
    }
  });

  // node_modules/core-js-pure/modules/es.map.constructor.js
  var require_es_map_constructor = __commonJS({
    "node_modules/core-js-pure/modules/es.map.constructor.js"() {
      "use strict";
      var collection = require_collection();
      var collectionStrong = require_collection_strong();
      collection("Map", function(init) {
        return function Map() {
          return init(this, arguments.length ? arguments[0] : void 0);
        };
      }, collectionStrong);
    }
  });

  // node_modules/core-js-pure/modules/es.map.js
  var require_es_map = __commonJS({
    "node_modules/core-js-pure/modules/es.map.js"() {
      "use strict";
      require_es_map_constructor();
    }
  });

  // node_modules/core-js-pure/modules/es.set.constructor.js
  var require_es_set_constructor = __commonJS({
    "node_modules/core-js-pure/modules/es.set.constructor.js"() {
      "use strict";
      var collection = require_collection();
      var collectionStrong = require_collection_strong();
      collection("Set", function(init) {
        return function Set() {
          return init(this, arguments.length ? arguments[0] : void 0);
        };
      }, collectionStrong);
    }
  });

  // node_modules/core-js-pure/modules/es.set.js
  var require_es_set = __commonJS({
    "node_modules/core-js-pure/modules/es.set.js"() {
      "use strict";
      require_es_set_constructor();
    }
  });

  // node_modules/core-js-pure/internals/environment.js
  var require_environment = __commonJS({
    "node_modules/core-js-pure/internals/environment.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var userAgent = require_environment_user_agent();
      var classof = require_classof_raw();
      var userAgentStartsWith = function(string) {
        return userAgent.slice(0, string.length) === string;
      };
      module.exports = (function() {
        if (userAgentStartsWith("Bun/")) return "BUN";
        if (userAgentStartsWith("Cloudflare-Workers")) return "CLOUDFLARE";
        if (userAgentStartsWith("Deno/")) return "DENO";
        if (userAgentStartsWith("Node.js/")) return "NODE";
        if (globalThis2.Bun && typeof Bun.version == "string") return "BUN";
        if (globalThis2.Deno && typeof Deno.version == "object") return "DENO";
        if (classof(globalThis2.process) === "process") return "NODE";
        if (globalThis2.window && globalThis2.document) return "BROWSER";
        return "REST";
      })();
    }
  });

  // node_modules/core-js-pure/internals/environment-is-node.js
  var require_environment_is_node = __commonJS({
    "node_modules/core-js-pure/internals/environment-is-node.js"(exports, module) {
      "use strict";
      var ENVIRONMENT = require_environment();
      module.exports = ENVIRONMENT === "NODE";
    }
  });

  // node_modules/core-js-pure/internals/get-built-in-node-module.js
  var require_get_built_in_node_module = __commonJS({
    "node_modules/core-js-pure/internals/get-built-in-node-module.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var IS_NODE = require_environment_is_node();
      module.exports = function(name2) {
        if (IS_NODE) {
          try {
            return globalThis2.process.getBuiltinModule(name2);
          } catch (error) {
          }
          try {
            return Function('return require("' + name2 + '")')();
          } catch (error) {
          }
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/to-string.js
  var require_to_string = __commonJS({
    "node_modules/core-js-pure/internals/to-string.js"(exports, module) {
      "use strict";
      var classof = require_classof();
      var $String = String;
      module.exports = function(argument) {
        if (classof(argument) === "Symbol") throw new TypeError("Cannot convert a Symbol value to a string");
        return $String(argument);
      };
    }
  });

  // node_modules/core-js-pure/internals/normalize-string-argument.js
  var require_normalize_string_argument = __commonJS({
    "node_modules/core-js-pure/internals/normalize-string-argument.js"(exports, module) {
      "use strict";
      var toString = require_to_string();
      module.exports = function(argument, $default) {
        return argument === void 0 ? arguments.length < 2 ? "" : $default : toString(argument);
      };
    }
  });

  // node_modules/core-js-pure/internals/error-to-string.js
  var require_error_to_string = __commonJS({
    "node_modules/core-js-pure/internals/error-to-string.js"(exports, module) {
      "use strict";
      var DESCRIPTORS = require_descriptors();
      var fails = require_fails();
      var anObject = require_an_object();
      var normalizeStringArgument = require_normalize_string_argument();
      var nativeErrorToString = Error.prototype.toString;
      var INCORRECT_TO_STRING = fails(function() {
        if (DESCRIPTORS) {
          var object = Object.create(Object.defineProperty({}, "name", { get: function() {
            return this === object;
          } }));
          if (nativeErrorToString.call(object) !== "true") return true;
        }
        return nativeErrorToString.call({ message: 1, name: 2 }) !== "2: 1" || nativeErrorToString.call({}) !== "Error";
      });
      module.exports = INCORRECT_TO_STRING ? function toString() {
        var O2 = anObject(this);
        var name2 = normalizeStringArgument(O2.name, "Error");
        var message = normalizeStringArgument(O2.message);
        return !name2 ? message : !message ? name2 : name2 + ": " + message;
      } : nativeErrorToString;
    }
  });

  // node_modules/core-js-pure/internals/dom-exception-constants.js
  var require_dom_exception_constants = __commonJS({
    "node_modules/core-js-pure/internals/dom-exception-constants.js"(exports, module) {
      "use strict";
      module.exports = {
        IndexSizeError: { s: "INDEX_SIZE_ERR", c: 1, m: 1 },
        DOMStringSizeError: { s: "DOMSTRING_SIZE_ERR", c: 2, m: 0 },
        HierarchyRequestError: { s: "HIERARCHY_REQUEST_ERR", c: 3, m: 1 },
        WrongDocumentError: { s: "WRONG_DOCUMENT_ERR", c: 4, m: 1 },
        InvalidCharacterError: { s: "INVALID_CHARACTER_ERR", c: 5, m: 1 },
        NoDataAllowedError: { s: "NO_DATA_ALLOWED_ERR", c: 6, m: 0 },
        NoModificationAllowedError: { s: "NO_MODIFICATION_ALLOWED_ERR", c: 7, m: 1 },
        NotFoundError: { s: "NOT_FOUND_ERR", c: 8, m: 1 },
        NotSupportedError: { s: "NOT_SUPPORTED_ERR", c: 9, m: 1 },
        InUseAttributeError: { s: "INUSE_ATTRIBUTE_ERR", c: 10, m: 1 },
        InvalidStateError: { s: "INVALID_STATE_ERR", c: 11, m: 1 },
        SyntaxError: { s: "SYNTAX_ERR", c: 12, m: 1 },
        InvalidModificationError: { s: "INVALID_MODIFICATION_ERR", c: 13, m: 1 },
        NamespaceError: { s: "NAMESPACE_ERR", c: 14, m: 1 },
        InvalidAccessError: { s: "INVALID_ACCESS_ERR", c: 15, m: 1 },
        ValidationError: { s: "VALIDATION_ERR", c: 16, m: 0 },
        TypeMismatchError: { s: "TYPE_MISMATCH_ERR", c: 17, m: 1 },
        SecurityError: { s: "SECURITY_ERR", c: 18, m: 1 },
        NetworkError: { s: "NETWORK_ERR", c: 19, m: 1 },
        AbortError: { s: "ABORT_ERR", c: 20, m: 1 },
        URLMismatchError: { s: "URL_MISMATCH_ERR", c: 21, m: 1 },
        QuotaExceededError: { s: "QUOTA_EXCEEDED_ERR", c: 22, m: 1 },
        TimeoutError: { s: "TIMEOUT_ERR", c: 23, m: 1 },
        InvalidNodeTypeError: { s: "INVALID_NODE_TYPE_ERR", c: 24, m: 1 },
        DataCloneError: { s: "DATA_CLONE_ERR", c: 25, m: 1 }
      };
    }
  });

  // node_modules/core-js-pure/internals/error-stack-clear.js
  var require_error_stack_clear = __commonJS({
    "node_modules/core-js-pure/internals/error-stack-clear.js"(exports, module) {
      "use strict";
      var uncurryThis = require_function_uncurry_this();
      var $Error = Error;
      var replace = uncurryThis("".replace);
      var TEST = (function(arg) {
        return String(new $Error(arg).stack);
      })("zxcasd");
      var V8_OR_CHAKRA_STACK_ENTRY = /\n\s*at [^:]*:[^\n]*/;
      var IS_V8_OR_CHAKRA_STACK = V8_OR_CHAKRA_STACK_ENTRY.test(TEST);
      module.exports = function(stack, dropEntries) {
        if (IS_V8_OR_CHAKRA_STACK && typeof stack == "string" && !$Error.prepareStackTrace) {
          while (dropEntries--) stack = replace(stack, V8_OR_CHAKRA_STACK_ENTRY, "");
        }
        return stack;
      };
    }
  });

  // node_modules/core-js-pure/modules/web.dom-exception.constructor.js
  var require_web_dom_exception_constructor = __commonJS({
    "node_modules/core-js-pure/modules/web.dom-exception.constructor.js"() {
      "use strict";
      var $2 = require_export();
      var getBuiltIn = require_get_built_in();
      var getBuiltInNodeModule = require_get_built_in_node_module();
      var fails = require_fails();
      var create = require_object_create();
      var createPropertyDescriptor = require_create_property_descriptor();
      var defineProperty = require_object_define_property().f;
      var defineBuiltIn = require_define_built_in();
      var defineBuiltInAccessor = require_define_built_in_accessor();
      var hasOwn = require_has_own_property();
      var anInstance = require_an_instance();
      var anObject = require_an_object();
      var errorToString = require_error_to_string();
      var normalizeStringArgument = require_normalize_string_argument();
      var DOMExceptionConstants = require_dom_exception_constants();
      var clearErrorStack = require_error_stack_clear();
      var InternalStateModule = require_internal_state();
      var DESCRIPTORS = require_descriptors();
      var IS_PURE = require_is_pure();
      var DOM_EXCEPTION = "DOMException";
      var DATA_CLONE_ERR = "DATA_CLONE_ERR";
      var Error2 = getBuiltIn("Error");
      var NativeDOMException = getBuiltIn(DOM_EXCEPTION) || (function() {
        try {
          var MessageChannel = getBuiltIn("MessageChannel") || getBuiltInNodeModule("worker_threads").MessageChannel;
          new MessageChannel().port1.postMessage(/* @__PURE__ */ new WeakMap());
        } catch (error) {
          if (error.name === DATA_CLONE_ERR && error.code === 25) return error.constructor;
        }
      })();
      var NativeDOMExceptionPrototype = NativeDOMException && NativeDOMException.prototype;
      var ErrorPrototype = Error2.prototype;
      var setInternalState = InternalStateModule.set;
      var getInternalState = InternalStateModule.getterFor(DOM_EXCEPTION);
      var HAS_STACK = "stack" in new Error2(DOM_EXCEPTION);
      var codeFor = function(name2) {
        return hasOwn(DOMExceptionConstants, name2) && DOMExceptionConstants[name2].m ? DOMExceptionConstants[name2].c : 0;
      };
      var $DOMException = function DOMException() {
        anInstance(this, DOMExceptionPrototype);
        var argumentsLength = arguments.length;
        var message = normalizeStringArgument(argumentsLength < 1 ? void 0 : arguments[0]);
        var name2 = normalizeStringArgument(argumentsLength < 2 ? void 0 : arguments[1], "Error");
        var code = codeFor(name2);
        setInternalState(this, {
          type: DOM_EXCEPTION,
          name: name2,
          message,
          code
        });
        if (!DESCRIPTORS) {
          this.name = name2;
          this.message = message;
          this.code = code;
        }
        if (HAS_STACK) {
          var error = new Error2(message);
          error.name = DOM_EXCEPTION;
          defineProperty(this, "stack", createPropertyDescriptor(1, clearErrorStack(error.stack, 1)));
        }
      };
      var DOMExceptionPrototype = $DOMException.prototype = create(ErrorPrototype);
      var createGetterDescriptor = function(get) {
        return { enumerable: true, configurable: true, get };
      };
      var getterFor = function(key2) {
        return createGetterDescriptor(function() {
          return getInternalState(this)[key2];
        });
      };
      if (DESCRIPTORS) {
        defineBuiltInAccessor(DOMExceptionPrototype, "code", getterFor("code"));
        defineBuiltInAccessor(DOMExceptionPrototype, "message", getterFor("message"));
        defineBuiltInAccessor(DOMExceptionPrototype, "name", getterFor("name"));
      }
      defineProperty(DOMExceptionPrototype, "constructor", createPropertyDescriptor(1, $DOMException));
      var INCORRECT_CONSTRUCTOR = fails(function() {
        return !(new NativeDOMException() instanceof Error2);
      });
      var INCORRECT_TO_STRING = INCORRECT_CONSTRUCTOR || fails(function() {
        return ErrorPrototype.toString !== errorToString || String(new NativeDOMException(1, 2)) !== "2: 1";
      });
      var INCORRECT_CODE = INCORRECT_CONSTRUCTOR || fails(function() {
        return new NativeDOMException(1, "DataCloneError").code !== 25;
      });
      var MISSED_CONSTANTS = INCORRECT_CONSTRUCTOR || NativeDOMException[DATA_CLONE_ERR] !== 25 || NativeDOMExceptionPrototype[DATA_CLONE_ERR] !== 25;
      var FORCED_CONSTRUCTOR = IS_PURE ? INCORRECT_TO_STRING || INCORRECT_CODE || MISSED_CONSTANTS : INCORRECT_CONSTRUCTOR;
      $2({ global: true, constructor: true, forced: FORCED_CONSTRUCTOR }, {
        DOMException: FORCED_CONSTRUCTOR ? $DOMException : NativeDOMException
      });
      var PolyfilledDOMException = getBuiltIn(DOM_EXCEPTION);
      var PolyfilledDOMExceptionPrototype = PolyfilledDOMException.prototype;
      if (INCORRECT_TO_STRING && (IS_PURE || NativeDOMException === PolyfilledDOMException)) {
        defineBuiltIn(PolyfilledDOMExceptionPrototype, "toString", errorToString);
      }
      if (INCORRECT_CODE && DESCRIPTORS && NativeDOMException === PolyfilledDOMException) {
        defineBuiltInAccessor(PolyfilledDOMExceptionPrototype, "code", createGetterDescriptor(function() {
          return codeFor(anObject(this).name);
        }));
      }
      for (key in DOMExceptionConstants) if (hasOwn(DOMExceptionConstants, key)) {
        constant = DOMExceptionConstants[key];
        constantName = constant.s;
        descriptor = createPropertyDescriptor(6, constant.c);
        if (!hasOwn(PolyfilledDOMException, constantName)) {
          defineProperty(PolyfilledDOMException, constantName, descriptor);
        }
        if (!hasOwn(PolyfilledDOMExceptionPrototype, constantName)) {
          defineProperty(PolyfilledDOMExceptionPrototype, constantName, descriptor);
        }
      }
      var constant;
      var constantName;
      var descriptor;
      var key;
    }
  });

  // node_modules/core-js-pure/internals/inherit-if-required.js
  var require_inherit_if_required = __commonJS({
    "node_modules/core-js-pure/internals/inherit-if-required.js"(exports, module) {
      "use strict";
      var isCallable = require_is_callable();
      var isObject = require_is_object();
      var setPrototypeOf = require_object_set_prototype_of();
      module.exports = function($this, dummy, Wrapper) {
        var NewTarget, NewTargetPrototype;
        if (
          // it can work only with native `setPrototypeOf`
          setPrototypeOf && // we haven't completely correct pre-ES6 way for getting `new.target`, so use this
          isCallable(NewTarget = dummy.constructor) && NewTarget !== Wrapper && isObject(NewTargetPrototype = NewTarget.prototype) && NewTargetPrototype !== Wrapper.prototype
        ) setPrototypeOf($this, NewTargetPrototype);
        return $this;
      };
    }
  });

  // node_modules/core-js-pure/modules/web.dom-exception.stack.js
  var require_web_dom_exception_stack = __commonJS({
    "node_modules/core-js-pure/modules/web.dom-exception.stack.js"() {
      "use strict";
      var $2 = require_export();
      var globalThis2 = require_global_this();
      var getBuiltIn = require_get_built_in();
      var createPropertyDescriptor = require_create_property_descriptor();
      var defineProperty = require_object_define_property().f;
      var hasOwn = require_has_own_property();
      var anInstance = require_an_instance();
      var inheritIfRequired = require_inherit_if_required();
      var normalizeStringArgument = require_normalize_string_argument();
      var DOMExceptionConstants = require_dom_exception_constants();
      var clearErrorStack = require_error_stack_clear();
      var DESCRIPTORS = require_descriptors();
      var IS_PURE = require_is_pure();
      var DOM_EXCEPTION = "DOMException";
      var Error2 = getBuiltIn("Error");
      var NativeDOMException = getBuiltIn(DOM_EXCEPTION);
      var $DOMException = function DOMException() {
        anInstance(this, DOMExceptionPrototype);
        var argumentsLength = arguments.length;
        var message = normalizeStringArgument(argumentsLength < 1 ? void 0 : arguments[0]);
        var name2 = normalizeStringArgument(argumentsLength < 2 ? void 0 : arguments[1], "Error");
        var that = new NativeDOMException(message, name2);
        var error = new Error2(message);
        error.name = DOM_EXCEPTION;
        defineProperty(that, "stack", createPropertyDescriptor(1, clearErrorStack(error.stack, 1)));
        inheritIfRequired(that, this, $DOMException);
        return that;
      };
      var DOMExceptionPrototype = $DOMException.prototype = NativeDOMException.prototype;
      var ERROR_HAS_STACK = "stack" in new Error2(DOM_EXCEPTION);
      var DOM_EXCEPTION_HAS_STACK = "stack" in new NativeDOMException(1, 2);
      var descriptor = NativeDOMException && DESCRIPTORS && Object.getOwnPropertyDescriptor(globalThis2, DOM_EXCEPTION);
      var BUGGY_DESCRIPTOR = !!descriptor && !(descriptor.writable && descriptor.configurable);
      var FORCED_CONSTRUCTOR = ERROR_HAS_STACK && !BUGGY_DESCRIPTOR && !DOM_EXCEPTION_HAS_STACK;
      $2({ global: true, constructor: true, forced: IS_PURE || FORCED_CONSTRUCTOR }, {
        // TODO: fix export logic
        DOMException: FORCED_CONSTRUCTOR ? $DOMException : NativeDOMException
      });
      var PolyfilledDOMException = getBuiltIn(DOM_EXCEPTION);
      var PolyfilledDOMExceptionPrototype = PolyfilledDOMException.prototype;
      if (PolyfilledDOMExceptionPrototype.constructor !== PolyfilledDOMException) {
        if (!IS_PURE) {
          defineProperty(PolyfilledDOMExceptionPrototype, "constructor", createPropertyDescriptor(1, PolyfilledDOMException));
        }
        for (key in DOMExceptionConstants) if (hasOwn(DOMExceptionConstants, key)) {
          constant = DOMExceptionConstants[key];
          constantName = constant.s;
          if (!hasOwn(PolyfilledDOMException, constantName)) {
            defineProperty(PolyfilledDOMException, constantName, createPropertyDescriptor(6, constant.c));
          }
        }
      }
      var constant;
      var constantName;
      var key;
    }
  });

  // node_modules/core-js-pure/modules/web.dom-exception.to-string-tag.js
  var require_web_dom_exception_to_string_tag = __commonJS({
    "node_modules/core-js-pure/modules/web.dom-exception.to-string-tag.js"() {
      "use strict";
      var getBuiltIn = require_get_built_in();
      var setToStringTag = require_set_to_string_tag();
      var DOM_EXCEPTION = "DOMException";
      setToStringTag(getBuiltIn(DOM_EXCEPTION), DOM_EXCEPTION);
    }
  });

  // node_modules/core-js-pure/internals/validate-arguments-length.js
  var require_validate_arguments_length = __commonJS({
    "node_modules/core-js-pure/internals/validate-arguments-length.js"(exports, module) {
      "use strict";
      var $TypeError = TypeError;
      module.exports = function(passed, required) {
        if (passed < required) throw new $TypeError("Not enough arguments");
        return passed;
      };
    }
  });

  // node_modules/core-js-pure/internals/regexp-flags-detection.js
  var require_regexp_flags_detection = __commonJS({
    "node_modules/core-js-pure/internals/regexp-flags-detection.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var fails = require_fails();
      var RegExp2 = globalThis2.RegExp;
      var FLAGS_GETTER_IS_CORRECT = !fails(function() {
        var INDICES_SUPPORT = true;
        try {
          RegExp2(".", "d");
        } catch (error) {
          INDICES_SUPPORT = false;
        }
        var O2 = {};
        var calls = "";
        var expected = INDICES_SUPPORT ? "dgimsy" : "gimsy";
        var addGetter = function(key2, chr) {
          Object.defineProperty(O2, key2, { get: function() {
            calls += chr;
            return true;
          } });
        };
        var pairs = {
          dotAll: "s",
          global: "g",
          ignoreCase: "i",
          multiline: "m",
          sticky: "y"
        };
        if (INDICES_SUPPORT) pairs.hasIndices = "d";
        for (var key in pairs) addGetter(key, pairs[key]);
        var result = Object.getOwnPropertyDescriptor(RegExp2.prototype, "flags").get.call(O2);
        return result !== expected || calls !== expected;
      });
      module.exports = { correct: FLAGS_GETTER_IS_CORRECT };
    }
  });

  // node_modules/core-js-pure/internals/regexp-flags.js
  var require_regexp_flags = __commonJS({
    "node_modules/core-js-pure/internals/regexp-flags.js"(exports, module) {
      "use strict";
      var anObject = require_an_object();
      module.exports = function() {
        var that = anObject(this);
        var result = "";
        if (that.hasIndices) result += "d";
        if (that.global) result += "g";
        if (that.ignoreCase) result += "i";
        if (that.multiline) result += "m";
        if (that.dotAll) result += "s";
        if (that.unicode) result += "u";
        if (that.unicodeSets) result += "v";
        if (that.sticky) result += "y";
        return result;
      };
    }
  });

  // node_modules/core-js-pure/internals/regexp-get-flags.js
  var require_regexp_get_flags = __commonJS({
    "node_modules/core-js-pure/internals/regexp-get-flags.js"(exports, module) {
      "use strict";
      var call = require_function_call();
      var hasOwn = require_has_own_property();
      var isPrototypeOf = require_object_is_prototype_of();
      var regExpFlagsDetection = require_regexp_flags_detection();
      var regExpFlagsGetterImplementation = require_regexp_flags();
      var RegExpPrototype = RegExp.prototype;
      module.exports = regExpFlagsDetection.correct ? function(it2) {
        return it2.flags;
      } : function(it2) {
        return !regExpFlagsDetection.correct && isPrototypeOf(RegExpPrototype, it2) && !hasOwn(it2, "flags") ? call(regExpFlagsGetterImplementation, it2) : it2.flags;
      };
    }
  });

  // node_modules/core-js-pure/internals/caller.js
  var require_caller = __commonJS({
    "node_modules/core-js-pure/internals/caller.js"(exports, module) {
      "use strict";
      module.exports = function(methodName, numArgs) {
        return numArgs === 1 ? function(object, arg) {
          return object[methodName](arg);
        } : function(object, arg1, arg2) {
          return object[methodName](arg1, arg2);
        };
      };
    }
  });

  // node_modules/core-js-pure/internals/map-helpers.js
  var require_map_helpers = __commonJS({
    "node_modules/core-js-pure/internals/map-helpers.js"(exports, module) {
      "use strict";
      var getBuiltIn = require_get_built_in();
      var caller = require_caller();
      var Map = getBuiltIn("Map");
      module.exports = {
        Map,
        set: caller("set", 2),
        get: caller("get", 1),
        has: caller("has", 1),
        remove: caller("delete", 1),
        proto: Map.prototype
      };
    }
  });

  // node_modules/core-js-pure/internals/set-helpers.js
  var require_set_helpers = __commonJS({
    "node_modules/core-js-pure/internals/set-helpers.js"(exports, module) {
      "use strict";
      var getBuiltIn = require_get_built_in();
      var caller = require_caller();
      var Set = getBuiltIn("Set");
      var SetPrototype = Set.prototype;
      module.exports = {
        Set,
        add: caller("add", 1),
        has: caller("has", 1),
        remove: caller("delete", 1),
        proto: SetPrototype
      };
    }
  });

  // node_modules/core-js-pure/internals/iterate-simple.js
  var require_iterate_simple = __commonJS({
    "node_modules/core-js-pure/internals/iterate-simple.js"(exports, module) {
      "use strict";
      var call = require_function_call();
      module.exports = function(record, fn2, ITERATOR_INSTEAD_OF_RECORD) {
        var iterator = ITERATOR_INSTEAD_OF_RECORD ? record : record.iterator;
        var next = record.next;
        var step, result;
        while (!(step = call(next, iterator)).done) {
          result = fn2(step.value);
          if (result !== void 0) return result;
        }
      };
    }
  });

  // node_modules/core-js-pure/internals/set-iterate.js
  var require_set_iterate = __commonJS({
    "node_modules/core-js-pure/internals/set-iterate.js"(exports, module) {
      "use strict";
      var iterateSimple = require_iterate_simple();
      module.exports = function(set, fn2, interruptible) {
        return interruptible ? iterateSimple(set.keys(), fn2, true) : set.forEach(fn2);
      };
    }
  });

  // node_modules/core-js-pure/internals/structured-clone-proper-transfer.js
  var require_structured_clone_proper_transfer = __commonJS({
    "node_modules/core-js-pure/internals/structured-clone-proper-transfer.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var fails = require_fails();
      var V8 = require_environment_v8_version();
      var ENVIRONMENT = require_environment();
      var structuredClone3 = globalThis2.structuredClone;
      module.exports = !!structuredClone3 && !fails(function() {
        if (ENVIRONMENT === "DENO" && V8 > 92 || ENVIRONMENT === "NODE" && V8 > 94 || ENVIRONMENT === "BROWSER" && V8 > 97) return false;
        var buffer = new ArrayBuffer(8);
        var clone = structuredClone3(buffer, { transfer: [buffer] });
        return buffer.byteLength !== 0 || clone.byteLength !== 8;
      });
    }
  });

  // node_modules/core-js-pure/internals/detach-transferable.js
  var require_detach_transferable = __commonJS({
    "node_modules/core-js-pure/internals/detach-transferable.js"(exports, module) {
      "use strict";
      var globalThis2 = require_global_this();
      var getBuiltInNodeModule = require_get_built_in_node_module();
      var PROPER_STRUCTURED_CLONE_TRANSFER = require_structured_clone_proper_transfer();
      var structuredClone3 = globalThis2.structuredClone;
      var $ArrayBuffer = globalThis2.ArrayBuffer;
      var $MessageChannel = globalThis2.MessageChannel;
      var detach = false;
      var WorkerThreads;
      var channel;
      var buffer;
      var $detach;
      if (PROPER_STRUCTURED_CLONE_TRANSFER) {
        detach = function(transferable) {
          structuredClone3(transferable, { transfer: [transferable] });
        };
      } else if ($ArrayBuffer) try {
        if (!$MessageChannel) {
          WorkerThreads = getBuiltInNodeModule("worker_threads");
          if (WorkerThreads) $MessageChannel = WorkerThreads.MessageChannel;
        }
        if ($MessageChannel) {
          channel = new $MessageChannel();
          buffer = new $ArrayBuffer(2);
          $detach = function(transferable) {
            channel.port1.postMessage(null, [transferable]);
          };
          if (buffer.byteLength === 2) {
            $detach(buffer);
            if (buffer.byteLength === 0) detach = $detach;
          }
        }
      } catch (error) {
      }
      module.exports = detach;
    }
  });

  // node_modules/core-js-pure/internals/error-stack-installable.js
  var require_error_stack_installable = __commonJS({
    "node_modules/core-js-pure/internals/error-stack-installable.js"(exports, module) {
      "use strict";
      var fails = require_fails();
      var createPropertyDescriptor = require_create_property_descriptor();
      module.exports = !fails(function() {
        var error = new Error("a");
        if (!("stack" in error)) return true;
        Object.defineProperty(error, "stack", createPropertyDescriptor(1, 7));
        return error.stack !== 7;
      });
    }
  });

  // node_modules/core-js-pure/modules/web.structured-clone.js
  var require_web_structured_clone = __commonJS({
    "node_modules/core-js-pure/modules/web.structured-clone.js"() {
      "use strict";
      var IS_PURE = require_is_pure();
      var $2 = require_export();
      var globalThis2 = require_global_this();
      var getBuiltIn = require_get_built_in();
      var uncurryThis = require_function_uncurry_this();
      var fails = require_fails();
      var uid = require_uid();
      var isCallable = require_is_callable();
      var isConstructor = require_is_constructor();
      var isNullOrUndefined = require_is_null_or_undefined();
      var isObject = require_is_object();
      var isSymbol = require_is_symbol();
      var iterate = require_iterate();
      var anObject = require_an_object();
      var classof = require_classof();
      var hasOwn = require_has_own_property();
      var createProperty = require_create_property();
      var createNonEnumerableProperty = require_create_non_enumerable_property();
      var lengthOfArrayLike = require_length_of_array_like();
      var validateArgumentsLength = require_validate_arguments_length();
      var getRegExpFlags = require_regexp_get_flags();
      var MapHelpers = require_map_helpers();
      var SetHelpers = require_set_helpers();
      var setIterate = require_set_iterate();
      var detachTransferable = require_detach_transferable();
      var ERROR_STACK_INSTALLABLE = require_error_stack_installable();
      var PROPER_STRUCTURED_CLONE_TRANSFER = require_structured_clone_proper_transfer();
      var Object2 = globalThis2.Object;
      var Array2 = globalThis2.Array;
      var Date = globalThis2.Date;
      var Error2 = globalThis2.Error;
      var TypeError2 = globalThis2.TypeError;
      var PerformanceMark = globalThis2.PerformanceMark;
      var DOMException = getBuiltIn("DOMException");
      var Map = MapHelpers.Map;
      var mapHas = MapHelpers.has;
      var mapGet = MapHelpers.get;
      var mapSet = MapHelpers.set;
      var Set = SetHelpers.Set;
      var setAdd = SetHelpers.add;
      var setHas = SetHelpers.has;
      var objectKeys = getBuiltIn("Object", "keys");
      var push = uncurryThis([].push);
      var thisBooleanValue = uncurryThis(true.valueOf);
      var thisNumberValue = uncurryThis(1.1.valueOf);
      var thisStringValue = uncurryThis("".valueOf);
      var thisTimeValue = uncurryThis(Date.prototype.getTime);
      var PERFORMANCE_MARK = uid("structuredClone");
      var DATA_CLONE_ERROR = "DataCloneError";
      var TRANSFERRING = "Transferring";
      var checkBasicSemantic = function(structuredCloneImplementation) {
        return !fails(function() {
          var set1 = new globalThis2.Set([7]);
          var set2 = structuredCloneImplementation(set1);
          var number = structuredCloneImplementation(Object2(7));
          return set2 === set1 || !set2.has(7) || !isObject(number) || +number !== 7;
        }) && structuredCloneImplementation;
      };
      var checkErrorsCloning = function(structuredCloneImplementation, $Error) {
        return !fails(function() {
          var error = new $Error();
          var test = structuredCloneImplementation({ a: error, b: error });
          return !(test && test.a === test.b && test.a instanceof $Error && test.a.stack === error.stack);
        });
      };
      var checkNewErrorsCloningSemantic = function(structuredCloneImplementation) {
        return !fails(function() {
          var test = structuredCloneImplementation(new globalThis2.AggregateError([1], PERFORMANCE_MARK, { cause: 3 }));
          return test.name !== "AggregateError" || test.errors[0] !== 1 || test.message !== PERFORMANCE_MARK || test.cause !== 3;
        });
      };
      var nativeStructuredClone = globalThis2.structuredClone;
      var FORCED_REPLACEMENT = IS_PURE || !checkErrorsCloning(nativeStructuredClone, Error2) || !checkErrorsCloning(nativeStructuredClone, DOMException) || !checkNewErrorsCloningSemantic(nativeStructuredClone);
      var structuredCloneFromMark = !nativeStructuredClone && checkBasicSemantic(function(value) {
        return new PerformanceMark(PERFORMANCE_MARK, { detail: value }).detail;
      });
      var nativeRestrictedStructuredClone = checkBasicSemantic(nativeStructuredClone) || structuredCloneFromMark;
      var throwUncloneable = function(type) {
        throw new DOMException("Uncloneable type: " + type, DATA_CLONE_ERROR);
      };
      var throwUnpolyfillable = function(type, action) {
        throw new DOMException((action || "Cloning") + " of " + type + " cannot be properly polyfilled in this engine", DATA_CLONE_ERROR);
      };
      var tryNativeRestrictedStructuredClone = function(value, type) {
        if (!nativeRestrictedStructuredClone) throwUnpolyfillable(type);
        return nativeRestrictedStructuredClone(value);
      };
      var createDataTransfer = function() {
        var dataTransfer;
        try {
          dataTransfer = new globalThis2.DataTransfer();
        } catch (error) {
          try {
            dataTransfer = new globalThis2.ClipboardEvent("").clipboardData;
          } catch (error2) {
          }
        }
        return dataTransfer && dataTransfer.items && dataTransfer.files ? dataTransfer : null;
      };
      var cloneBuffer = function(value, map, $type) {
        if (mapHas(map, value)) return mapGet(map, value);
        var type = $type || classof(value);
        var clone, length, options, source, target, i2;
        if (type === "SharedArrayBuffer") {
          if (nativeRestrictedStructuredClone) clone = nativeRestrictedStructuredClone(value);
          else clone = value;
        } else {
          var DataView2 = globalThis2.DataView;
          if (!DataView2 && !isCallable(value.slice)) throwUnpolyfillable("ArrayBuffer");
          try {
            if (isCallable(value.slice) && !value.resizable) {
              clone = value.slice(0);
            } else {
              length = value.byteLength;
              options = "maxByteLength" in value ? { maxByteLength: value.maxByteLength } : void 0;
              clone = new ArrayBuffer(length, options);
              source = new DataView2(value);
              target = new DataView2(clone);
              for (i2 = 0; i2 < length; i2++) {
                target.setUint8(i2, source.getUint8(i2));
              }
            }
          } catch (error) {
            throw new DOMException("ArrayBuffer is detached", DATA_CLONE_ERROR);
          }
        }
        mapSet(map, value, clone);
        return clone;
      };
      var cloneView = function(value, type, offset, length, map) {
        var C2 = globalThis2[type];
        if (!isObject(C2)) throwUnpolyfillable(type);
        return new C2(cloneBuffer(value.buffer, map), offset, length);
      };
      var structuredCloneInternal = function(value, map) {
        if (isSymbol(value)) throwUncloneable("Symbol");
        if (!isObject(value)) return value;
        if (map) {
          if (mapHas(map, value)) return mapGet(map, value);
        } else map = new Map();
        var type = classof(value);
        var C2, name2, cloned, dataTransfer, i2, length, keys, key;
        switch (type) {
          case "Array":
            cloned = Array2(lengthOfArrayLike(value));
            break;
          case "Object":
            cloned = {};
            break;
          case "Map":
            cloned = new Map();
            break;
          case "Set":
            cloned = new Set();
            break;
          case "RegExp":
            cloned = new RegExp(value.source, getRegExpFlags(value));
            break;
          case "Error":
            name2 = value.name;
            switch (name2) {
              case "AggregateError":
                cloned = new (getBuiltIn(name2))([]);
                break;
              case "EvalError":
              case "RangeError":
              case "ReferenceError":
              case "SuppressedError":
              case "SyntaxError":
              case "TypeError":
              case "URIError":
                cloned = new (getBuiltIn(name2))();
                break;
              case "CompileError":
              case "LinkError":
              case "RuntimeError":
                cloned = new (getBuiltIn("WebAssembly", name2))();
                break;
              default:
                cloned = new Error2();
            }
            break;
          case "DOMException":
            cloned = new DOMException(value.message, value.name);
            break;
          case "ArrayBuffer":
          case "SharedArrayBuffer":
            cloned = cloneBuffer(value, map, type);
            break;
          case "DataView":
          case "Int8Array":
          case "Uint8Array":
          case "Uint8ClampedArray":
          case "Int16Array":
          case "Uint16Array":
          case "Int32Array":
          case "Uint32Array":
          case "Float16Array":
          case "Float32Array":
          case "Float64Array":
          case "BigInt64Array":
          case "BigUint64Array":
            length = type === "DataView" ? value.byteLength : value.length;
            cloned = cloneView(value, type, value.byteOffset, length, map);
            break;
          case "DOMQuad":
            try {
              cloned = new DOMQuad(
                structuredCloneInternal(value.p1, map),
                structuredCloneInternal(value.p2, map),
                structuredCloneInternal(value.p3, map),
                structuredCloneInternal(value.p4, map)
              );
            } catch (error) {
              cloned = tryNativeRestrictedStructuredClone(value, type);
            }
            break;
          case "File":
            if (nativeRestrictedStructuredClone) try {
              cloned = nativeRestrictedStructuredClone(value);
              if (classof(cloned) !== type) cloned = void 0;
            } catch (error) {
            }
            if (!cloned) try {
              cloned = new File([value], value.name, value);
            } catch (error) {
            }
            if (!cloned) throwUnpolyfillable(type);
            break;
          case "FileList":
            dataTransfer = createDataTransfer();
            if (dataTransfer) {
              for (i2 = 0, length = lengthOfArrayLike(value); i2 < length; i2++) {
                dataTransfer.items.add(structuredCloneInternal(value[i2], map));
              }
              cloned = dataTransfer.files;
            } else cloned = tryNativeRestrictedStructuredClone(value, type);
            break;
          case "ImageData":
            try {
              cloned = new ImageData(
                structuredCloneInternal(value.data, map),
                value.width,
                value.height,
                { colorSpace: value.colorSpace }
              );
            } catch (error) {
              cloned = tryNativeRestrictedStructuredClone(value, type);
            }
            break;
          default:
            if (nativeRestrictedStructuredClone) {
              cloned = nativeRestrictedStructuredClone(value);
            } else switch (type) {
              case "BigInt":
                cloned = Object2(value.valueOf());
                break;
              case "Boolean":
                cloned = Object2(thisBooleanValue(value));
                break;
              case "Number":
                cloned = Object2(thisNumberValue(value));
                break;
              case "String":
                cloned = Object2(thisStringValue(value));
                break;
              case "Date":
                cloned = new Date(thisTimeValue(value));
                break;
              case "Blob":
                try {
                  cloned = value.slice(0, value.size, value.type);
                } catch (error) {
                  throwUnpolyfillable(type);
                }
                break;
              case "DOMPoint":
              case "DOMPointReadOnly":
                C2 = globalThis2[type];
                try {
                  cloned = C2.fromPoint ? C2.fromPoint(value) : new C2(value.x, value.y, value.z, value.w);
                } catch (error) {
                  throwUnpolyfillable(type);
                }
                break;
              case "DOMRect":
              case "DOMRectReadOnly":
                C2 = globalThis2[type];
                try {
                  cloned = C2.fromRect ? C2.fromRect(value) : new C2(value.x, value.y, value.width, value.height);
                } catch (error) {
                  throwUnpolyfillable(type);
                }
                break;
              case "DOMMatrix":
              case "DOMMatrixReadOnly":
                C2 = globalThis2[type];
                try {
                  cloned = C2.fromMatrix ? C2.fromMatrix(value) : new C2(value);
                } catch (error) {
                  throwUnpolyfillable(type);
                }
                break;
              case "AudioData":
              case "VideoFrame":
                if (!isCallable(value.clone)) throwUnpolyfillable(type);
                try {
                  cloned = value.clone();
                } catch (error) {
                  throwUncloneable(type);
                }
                break;
              case "CropTarget":
              case "CryptoKey":
              case "FileSystemDirectoryHandle":
              case "FileSystemFileHandle":
              case "FileSystemHandle":
              case "GPUCompilationInfo":
              case "GPUCompilationMessage":
              case "ImageBitmap":
              case "RTCCertificate":
              case "WebAssembly.Module":
                throwUnpolyfillable(type);
              // break omitted
              default:
                throwUncloneable(type);
            }
        }
        mapSet(map, value, cloned);
        switch (type) {
          case "Array":
          case "Object":
            keys = objectKeys(value);
            for (i2 = 0, length = lengthOfArrayLike(keys); i2 < length; i2++) {
              key = keys[i2];
              createProperty(cloned, key, structuredCloneInternal(value[key], map));
            }
            break;
          case "Map":
            value.forEach(function(v2, k2) {
              mapSet(cloned, structuredCloneInternal(k2, map), structuredCloneInternal(v2, map));
            });
            break;
          case "Set":
            value.forEach(function(v2) {
              setAdd(cloned, structuredCloneInternal(v2, map));
            });
            break;
          case "Error":
            createNonEnumerableProperty(cloned, "message", structuredCloneInternal(value.message, map));
            if (hasOwn(value, "cause")) {
              createNonEnumerableProperty(cloned, "cause", structuredCloneInternal(value.cause, map));
            }
            if (name2 === "AggregateError") {
              cloned.errors = structuredCloneInternal(value.errors, map);
            } else if (name2 === "SuppressedError") {
              cloned.error = structuredCloneInternal(value.error, map);
              cloned.suppressed = structuredCloneInternal(value.suppressed, map);
            }
          // break omitted
          case "DOMException":
            if (ERROR_STACK_INSTALLABLE) {
              createNonEnumerableProperty(cloned, "stack", structuredCloneInternal(value.stack, map));
            }
        }
        return cloned;
      };
      var tryToTransfer = function(rawTransfer, map) {
        if (!isObject(rawTransfer)) throw new TypeError2("Transfer option cannot be converted to a sequence");
        var transfer = [];
        iterate(rawTransfer, function(value2) {
          push(transfer, anObject(value2));
        });
        var i2 = 0;
        var length = lengthOfArrayLike(transfer);
        var buffers = new Set();
        var value, type, C2, transferred, canvas, context;
        while (i2 < length) {
          value = transfer[i2++];
          type = classof(value);
          transferred = void 0;
          if (type === "ArrayBuffer" ? setHas(buffers, value) : mapHas(map, value)) {
            throw new DOMException("Duplicate transferable", DATA_CLONE_ERROR);
          }
          if (type === "ArrayBuffer") {
            setAdd(buffers, value);
            continue;
          }
          if (PROPER_STRUCTURED_CLONE_TRANSFER) {
            transferred = nativeStructuredClone(value, { transfer: [value] });
          } else switch (type) {
            case "ImageBitmap":
              C2 = globalThis2.OffscreenCanvas;
              if (!isConstructor(C2)) throwUnpolyfillable(type, TRANSFERRING);
              try {
                canvas = new C2(value.width, value.height);
                context = canvas.getContext("bitmaprenderer");
                context.transferFromImageBitmap(value);
                transferred = canvas.transferToImageBitmap();
              } catch (error) {
              }
              break;
            case "AudioData":
            case "VideoFrame":
              if (!isCallable(value.clone) || !isCallable(value.close)) throwUnpolyfillable(type, TRANSFERRING);
              try {
                transferred = value.clone();
                value.close();
              } catch (error) {
              }
              break;
            case "MediaSourceHandle":
            case "MessagePort":
            case "MIDIAccess":
            case "OffscreenCanvas":
            case "ReadableStream":
            case "RTCDataChannel":
            case "TransformStream":
            case "WebTransportReceiveStream":
            case "WebTransportSendStream":
            case "WritableStream":
              throwUnpolyfillable(type, TRANSFERRING);
          }
          if (transferred === void 0) throw new DOMException("This object cannot be transferred: " + type, DATA_CLONE_ERROR);
          mapSet(map, value, transferred);
        }
        return buffers;
      };
      var detachBuffers = function(buffers) {
        setIterate(buffers, function(buffer) {
          if (PROPER_STRUCTURED_CLONE_TRANSFER) {
            nativeStructuredClone(buffer, { transfer: [buffer] });
          } else if (isCallable(buffer.transfer)) {
            buffer.transfer();
          } else if (detachTransferable) {
            detachTransferable(buffer);
          } else {
            throwUnpolyfillable("ArrayBuffer", TRANSFERRING);
          }
        });
      };
      $2({ global: true, enumerable: true, sham: !PROPER_STRUCTURED_CLONE_TRANSFER, forced: FORCED_REPLACEMENT }, {
        structuredClone: function structuredClone3(value) {
          var options = validateArgumentsLength(arguments.length, 1) > 1 && !isNullOrUndefined(arguments[1]) ? anObject(arguments[1]) : void 0;
          var transfer = options ? options.transfer : void 0;
          var map, buffers;
          if (transfer !== void 0) {
            map = new Map();
            buffers = tryToTransfer(transfer, map);
          }
          var clone = structuredCloneInternal(value, map);
          if (buffers) detachBuffers(buffers);
          return clone;
        }
      });
    }
  });

  // node_modules/core-js-pure/stable/structured-clone.js
  var require_structured_clone = __commonJS({
    "node_modules/core-js-pure/stable/structured-clone.js"(exports, module) {
      "use strict";
      require_es_error_to_string();
      require_es_array_iterator();
      require_es_object_keys();
      require_es_object_to_string();
      require_es_map();
      require_es_set();
      require_web_dom_exception_constructor();
      require_web_dom_exception_stack();
      require_web_dom_exception_to_string_tag();
      require_web_structured_clone();
      var path = require_path();
      module.exports = path.structuredClone;
    }
  });

  // node_modules/core-js-pure/actual/structured-clone.js
  var require_structured_clone2 = __commonJS({
    "node_modules/core-js-pure/actual/structured-clone.js"(exports, module) {
      "use strict";
      var parent = require_structured_clone();
      module.exports = parent;
    }
  });

  // node_modules/xpath/xpath.js
  var require_xpath = __commonJS({
    "node_modules/xpath/xpath.js"(exports) {
      var xpath2 = typeof exports === "undefined" ? {} : exports;
      (function(exports2) {
        "use strict";
        var NAMESPACE_NODE_NODETYPE = "__namespace";
        var isNil = function(x2) {
          return x2 === null || x2 === void 0;
        };
        var isPositiveInteger = function(x2) {
          return typeof x2 === "number" && x2 >= 1 && Math.floor(x2) === x2;
        };
        var isValidNodeType = function(nodeType) {
          return nodeType === NAMESPACE_NODE_NODETYPE || Number.isInteger(nodeType) && nodeType >= 1 && nodeType <= 11;
        };
        var isNodeLike = function(value) {
          return value && isValidNodeType(value.nodeType) && typeof value.nodeName === "string";
        };
        function curry(func) {
          var slice = Array.prototype.slice, totalargs = func.length, partial = function(args, fn3) {
            return function() {
              return fn3.apply(this, args.concat(slice.call(arguments)));
            };
          }, fn2 = function() {
            var args = slice.call(arguments);
            return args.length < totalargs ? partial(args, fn2) : func.apply(this, slice.apply(arguments, [0, totalargs]));
          };
          return fn2;
        }
        var forEach = function(f2, xs) {
          for (var i2 = 0; i2 < xs.length; i2 += 1) {
            f2(xs[i2], i2, xs);
          }
        };
        var reduce = function(f2, seed, xs) {
          var acc = seed;
          forEach(function(x2, i2) {
            acc = f2(acc, x2, i2);
          }, xs);
          return acc;
        };
        var map = function(f2, xs) {
          var mapped = new Array(xs.length);
          forEach(function(x2, i2) {
            mapped[i2] = f2(x2);
          }, xs);
          return mapped;
        };
        var filter = function(f2, xs) {
          var filtered = [];
          forEach(function(x2, i2) {
            if (f2(x2, i2)) {
              filtered.push(x2);
            }
          }, xs);
          return filtered;
        };
        var includes = function(values, value) {
          for (var i2 = 0; i2 < values.length; i2 += 1) {
            if (values[i2] === value) {
              return true;
            }
          }
          return false;
        };
        function always(value) {
          return function() {
            return value;
          };
        }
        function toString(x2) {
          return x2.toString();
        }
        var join = function(s2, xs) {
          return xs.join(s2);
        };
        var wrap = function(pref, suf, str) {
          return pref + str + suf;
        };
        var prototypeConcat = Array.prototype.concat;
        var sortNodes = function(nodes, reverse) {
          var ns = new XNodeSet();
          ns.addArray(nodes);
          var sorted = ns.toArray();
          return reverse ? sorted.reverse() : sorted;
        };
        var MAX_ARGUMENT_LENGTH = 32767;
        function flatten(arr) {
          var result = [];
          for (var start = 0; start < arr.length; start += MAX_ARGUMENT_LENGTH) {
            var chunk = arr.slice(start, start + MAX_ARGUMENT_LENGTH);
            result = prototypeConcat.apply(result, chunk);
          }
          return result;
        }
        function assign(target, varArgs) {
          var to2 = Object(target);
          for (var index = 1; index < arguments.length; index++) {
            var nextSource = arguments[index];
            if (nextSource != null) {
              for (var nextKey in nextSource) {
                if (Object.prototype.hasOwnProperty.call(nextSource, nextKey)) {
                  to2[nextKey] = nextSource[nextKey];
                }
              }
            }
          }
          return to2;
        }
        var NodeTypes = {
          ELEMENT_NODE: 1,
          ATTRIBUTE_NODE: 2,
          TEXT_NODE: 3,
          CDATA_SECTION_NODE: 4,
          PROCESSING_INSTRUCTION_NODE: 7,
          COMMENT_NODE: 8,
          DOCUMENT_NODE: 9,
          DOCUMENT_TYPE_NODE: 10,
          DOCUMENT_FRAGMENT_NODE: 11,
          NAMESPACE_NODE: NAMESPACE_NODE_NODETYPE
        };
        XPathParser.prototype = new Object();
        XPathParser.prototype.constructor = XPathParser;
        XPathParser.superclass = Object.prototype;
        function XPathParser() {
          this.init();
        }
        XPathParser.prototype.init = function() {
          this.reduceActions = [];
          this.reduceActions[3] = function(rhs) {
            return new OrOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[5] = function(rhs) {
            return new AndOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[7] = function(rhs) {
            return new EqualsOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[8] = function(rhs) {
            return new NotEqualOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[10] = function(rhs) {
            return new LessThanOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[11] = function(rhs) {
            return new GreaterThanOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[12] = function(rhs) {
            return new LessThanOrEqualOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[13] = function(rhs) {
            return new GreaterThanOrEqualOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[15] = function(rhs) {
            return new PlusOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[16] = function(rhs) {
            return new MinusOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[18] = function(rhs) {
            return new MultiplyOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[19] = function(rhs) {
            return new DivOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[20] = function(rhs) {
            return new ModOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[22] = function(rhs) {
            return new UnaryMinusOperation(rhs[1]);
          };
          this.reduceActions[24] = function(rhs) {
            return new BarOperation(rhs[0], rhs[2]);
          };
          this.reduceActions[25] = function(rhs) {
            return new PathExpr(void 0, void 0, rhs[0]);
          };
          this.reduceActions[27] = function(rhs) {
            rhs[0].locationPath = rhs[2];
            return rhs[0];
          };
          this.reduceActions[28] = function(rhs) {
            rhs[0].locationPath = rhs[2];
            rhs[0].locationPath.steps.unshift(new Step(Step.DESCENDANTORSELF, NodeTest.nodeTest, []));
            return rhs[0];
          };
          this.reduceActions[29] = function(rhs) {
            return new PathExpr(rhs[0], [], void 0);
          };
          this.reduceActions[30] = function(rhs) {
            if (Utilities.instance_of(rhs[0], PathExpr)) {
              if (rhs[0].filterPredicates == void 0) {
                rhs[0].filterPredicates = [];
              }
              rhs[0].filterPredicates.push(rhs[1]);
              return rhs[0];
            } else {
              return new PathExpr(rhs[0], [rhs[1]], void 0);
            }
          };
          this.reduceActions[32] = function(rhs) {
            return rhs[1];
          };
          this.reduceActions[33] = function(rhs) {
            return new XString(rhs[0]);
          };
          this.reduceActions[34] = function(rhs) {
            return new XNumber(rhs[0]);
          };
          this.reduceActions[36] = function(rhs) {
            return new FunctionCall(rhs[0], []);
          };
          this.reduceActions[37] = function(rhs) {
            return new FunctionCall(rhs[0], rhs[2]);
          };
          this.reduceActions[38] = function(rhs) {
            return [rhs[0]];
          };
          this.reduceActions[39] = function(rhs) {
            rhs[2].unshift(rhs[0]);
            return rhs[2];
          };
          this.reduceActions[43] = function(rhs) {
            return new LocationPath(true, []);
          };
          this.reduceActions[44] = function(rhs) {
            rhs[1].absolute = true;
            return rhs[1];
          };
          this.reduceActions[46] = function(rhs) {
            return new LocationPath(false, [rhs[0]]);
          };
          this.reduceActions[47] = function(rhs) {
            rhs[0].steps.push(rhs[2]);
            return rhs[0];
          };
          this.reduceActions[49] = function(rhs) {
            return new Step(rhs[0], rhs[1], []);
          };
          this.reduceActions[50] = function(rhs) {
            return new Step(Step.CHILD, rhs[0], []);
          };
          this.reduceActions[51] = function(rhs) {
            return new Step(rhs[0], rhs[1], rhs[2]);
          };
          this.reduceActions[52] = function(rhs) {
            return new Step(Step.CHILD, rhs[0], rhs[1]);
          };
          this.reduceActions[54] = function(rhs) {
            return [rhs[0]];
          };
          this.reduceActions[55] = function(rhs) {
            rhs[1].unshift(rhs[0]);
            return rhs[1];
          };
          this.reduceActions[56] = function(rhs) {
            if (rhs[0] == "ancestor") {
              return Step.ANCESTOR;
            } else if (rhs[0] == "ancestor-or-self") {
              return Step.ANCESTORORSELF;
            } else if (rhs[0] == "attribute") {
              return Step.ATTRIBUTE;
            } else if (rhs[0] == "child") {
              return Step.CHILD;
            } else if (rhs[0] == "descendant") {
              return Step.DESCENDANT;
            } else if (rhs[0] == "descendant-or-self") {
              return Step.DESCENDANTORSELF;
            } else if (rhs[0] == "following") {
              return Step.FOLLOWING;
            } else if (rhs[0] == "following-sibling") {
              return Step.FOLLOWINGSIBLING;
            } else if (rhs[0] == "namespace") {
              return Step.NAMESPACE;
            } else if (rhs[0] == "parent") {
              return Step.PARENT;
            } else if (rhs[0] == "preceding") {
              return Step.PRECEDING;
            } else if (rhs[0] == "preceding-sibling") {
              return Step.PRECEDINGSIBLING;
            } else if (rhs[0] == "self") {
              return Step.SELF;
            }
            return -1;
          };
          this.reduceActions[57] = function(rhs) {
            return Step.ATTRIBUTE;
          };
          this.reduceActions[59] = function(rhs) {
            if (rhs[0] == "comment") {
              return NodeTest.commentTest;
            } else if (rhs[0] == "text") {
              return NodeTest.textTest;
            } else if (rhs[0] == "processing-instruction") {
              return NodeTest.anyPiTest;
            } else if (rhs[0] == "node") {
              return NodeTest.nodeTest;
            }
            return new NodeTest(-1, void 0);
          };
          this.reduceActions[60] = function(rhs) {
            return new NodeTest.PITest(rhs[2]);
          };
          this.reduceActions[61] = function(rhs) {
            return rhs[1];
          };
          this.reduceActions[63] = function(rhs) {
            rhs[1].absolute = true;
            rhs[1].steps.unshift(new Step(Step.DESCENDANTORSELF, NodeTest.nodeTest, []));
            return rhs[1];
          };
          this.reduceActions[64] = function(rhs) {
            rhs[0].steps.push(new Step(Step.DESCENDANTORSELF, NodeTest.nodeTest, []));
            rhs[0].steps.push(rhs[2]);
            return rhs[0];
          };
          this.reduceActions[65] = function(rhs) {
            return new Step(Step.SELF, NodeTest.nodeTest, []);
          };
          this.reduceActions[66] = function(rhs) {
            return new Step(Step.PARENT, NodeTest.nodeTest, []);
          };
          this.reduceActions[67] = function(rhs) {
            return new VariableReference(rhs[1]);
          };
          this.reduceActions[68] = function(rhs) {
            return NodeTest.nameTestAny;
          };
          this.reduceActions[69] = function(rhs) {
            return new NodeTest.NameTestPrefixAny(rhs[0].split(":")[0]);
          };
          this.reduceActions[70] = function(rhs) {
            return new NodeTest.NameTestQName(rhs[0]);
          };
        };
        XPathParser.actionTable = [
          " s s        sssssssss    s ss  s  ss",
          "                 s                  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "                rrrrr               ",
          " s s        sssssssss    s ss  s  ss",
          "rs  rrrrrrrr s  sssssrrrrrr  rrs rs ",
          " s s        sssssssss    s ss  s  ss",
          "                            s       ",
          "                            s       ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "  s                                 ",
          "                            s       ",
          " s           s  sssss          s  s ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "a                                   ",
          "r       s                    rr  r  ",
          "r      sr                    rr  r  ",
          "r   s  rr            s       rr  r  ",
          "r   rssrr            rss     rr  r  ",
          "r   rrrrr            rrrss   rr  r  ",
          "r   rrrrrsss         rrrrr   rr  r  ",
          "r   rrrrrrrr         rrrrr   rr  r  ",
          "r   rrrrrrrr         rrrrrs  rr  r  ",
          "r   rrrrrrrr         rrrrrr  rr  r  ",
          "r   rrrrrrrr         rrrrrr  rr  r  ",
          "r  srrrrrrrr         rrrrrrs rr sr  ",
          "r  srrrrrrrr         rrrrrrs rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r   rrrrrrrr         rrrrrr  rr  r  ",
          "r   rrrrrrrr         rrrrrr  rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "                sssss               ",
          "r  rrrrrrrrr         rrrrrrr rr sr  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "                             s      ",
          "r  srrrrrrrr         rrrrrrs rr  r  ",
          "r   rrrrrrrr         rrrrr   rr  r  ",
          "              s                     ",
          "                             s      ",
          "                rrrrr               ",
          " s s        sssssssss    s sss s  ss",
          "r  srrrrrrrr         rrrrrrs rr  r  ",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s s        sssssssss      ss  s  ss",
          " s s        sssssssss    s ss  s  ss",
          " s           s  sssss          s  s ",
          " s           s  sssss          s  s ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          " s           s  sssss          s  s ",
          " s           s  sssss          s  s ",
          "r  rrrrrrrrr         rrrrrrr rr sr  ",
          "r  rrrrrrrrr         rrrrrrr rr sr  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "                             s      ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "                             rr     ",
          "                             s      ",
          "                             rs     ",
          "r      sr                    rr  r  ",
          "r   s  rr            s       rr  r  ",
          "r   rssrr            rss     rr  r  ",
          "r   rssrr            rss     rr  r  ",
          "r   rrrrr            rrrss   rr  r  ",
          "r   rrrrr            rrrss   rr  r  ",
          "r   rrrrr            rrrss   rr  r  ",
          "r   rrrrr            rrrss   rr  r  ",
          "r   rrrrrsss         rrrrr   rr  r  ",
          "r   rrrrrsss         rrrrr   rr  r  ",
          "r   rrrrrrrr         rrrrr   rr  r  ",
          "r   rrrrrrrr         rrrrr   rr  r  ",
          "r   rrrrrrrr         rrrrr   rr  r  ",
          "r   rrrrrrrr         rrrrrr  rr  r  ",
          "                                 r  ",
          "                                 s  ",
          "r  srrrrrrrr         rrrrrrs rr  r  ",
          "r  srrrrrrrr         rrrrrrs rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr  r  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          " s s        sssssssss    s ss  s  ss",
          "r  rrrrrrrrr         rrrrrrr rr rr  ",
          "                             r      "
        ];
        XPathParser.actionTableNumber = [
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          "                 J                  ",
          "a  aaaaaaaaa         aaaaaaa aa  a  ",
          "                YYYYY               ",
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          `K1  KKKKKKKK .  +*)('KKKKKK  KK# K" `,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          "                            N       ",
          "                            O       ",
          "e  eeeeeeeee         eeeeeee ee ee  ",
          "f  fffffffff         fffffff ff ff  ",
          "d  ddddddddd         ddddddd dd dd  ",
          "B  BBBBBBBBB         BBBBBBB BB BB  ",
          "A  AAAAAAAAA         AAAAAAA AA AA  ",
          "  P                                 ",
          "                            Q       ",
          ` 1           .  +*)('          #  " `,
          "b  bbbbbbbbb         bbbbbbb bb  b  ",
          "                                    ",
          "!       S                    !!  !  ",
          '"      T"                    ""  "  ',
          "$   V  $$            U       $$  $  ",
          "&   &ZY&&            &XW     &&  &  ",
          ")   )))))            )))\\[   ))  )  ",
          ".   ....._^]         .....   ..  .  ",
          "1   11111111         11111   11  1  ",
          "5   55555555         55555`  55  5  ",
          "7   77777777         777777  77  7  ",
          "9   99999999         999999  99  9  ",
          ":  c::::::::         ::::::b :: a:  ",
          "I  fIIIIIIII         IIIIIIe II  I  ",
          "=  =========         ======= == ==  ",
          "?  ?????????         ??????? ?? ??  ",
          "C  CCCCCCCCC         CCCCCCC CC CC  ",
          "J   JJJJJJJJ         JJJJJJ  JJ  J  ",
          "M   MMMMMMMM         MMMMMM  MM  M  ",
          "N  NNNNNNNNN         NNNNNNN NN  N  ",
          "P  PPPPPPPPP         PPPPPPP PP  P  ",
          "                +*)('               ",
          "R  RRRRRRRRR         RRRRRRR RR aR  ",
          "U  UUUUUUUUU         UUUUUUU UU  U  ",
          "Z  ZZZZZZZZZ         ZZZZZZZ ZZ ZZ  ",
          "c  ccccccccc         ccccccc cc cc  ",
          "                             j      ",
          "L  fLLLLLLLL         LLLLLLe LL  L  ",
          "6   66666666         66666   66  6  ",
          "              k                     ",
          "                             l      ",
          "                XXXXX               ",
          ` 1 0        /.-,+*)('    & %$m #  "!`,
          "_  f________         ______e __  _  ",
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1 0        /.-,+*)('      %$  #  "!`,
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          ` 1           .  +*)('          #  " `,
          ` 1           .  +*)('          #  " `,
          ">  >>>>>>>>>         >>>>>>> >> >>  ",
          ` 1           .  +*)('          #  " `,
          ` 1           .  +*)('          #  " `,
          "Q  QQQQQQQQQ         QQQQQQQ QQ aQ  ",
          "V  VVVVVVVVV         VVVVVVV VV aV  ",
          "T  TTTTTTTTT         TTTTTTT TT  T  ",
          "@  @@@@@@@@@         @@@@@@@ @@ @@  ",
          "                             \x87      ",
          "[  [[[[[[[[[         [[[[[[[ [[ [[  ",
          "D  DDDDDDDDD         DDDDDDD DD DD  ",
          "                             HH     ",
          "                             \x88      ",
          "                             F\x89     ",
          "#      T#                    ##  #  ",
          "%   V  %%            U       %%  %  ",
          "'   'ZY''            'XW     ''  '  ",
          "(   (ZY((            (XW     ((  (  ",
          "+   +++++            +++\\[   ++  +  ",
          "*   *****            ***\\[   **  *  ",
          "-   -----            ---\\[   --  -  ",
          ",   ,,,,,            ,,,\\[   ,,  ,  ",
          "0   00000_^]         00000   00  0  ",
          "/   /////_^]         /////   //  /  ",
          "2   22222222         22222   22  2  ",
          "3   33333333         33333   33  3  ",
          "4   44444444         44444   44  4  ",
          "8   88888888         888888  88  8  ",
          "                                 ^  ",
          "                                 \x8A  ",
          ";  f;;;;;;;;         ;;;;;;e ;;  ;  ",
          "<  f<<<<<<<<         <<<<<<e <<  <  ",
          "O  OOOOOOOOO         OOOOOOO OO  O  ",
          "`  `````````         ``````` ``  `  ",
          "S  SSSSSSSSS         SSSSSSS SS  S  ",
          "W  WWWWWWWWW         WWWWWWW WW  W  ",
          "\\  \\\\\\\\\\\\\\\\\\         \\\\\\\\\\\\\\ \\\\ \\\\  ",
          "E  EEEEEEEEE         EEEEEEE EE EE  ",
          ` 1 0        /.-,+*)('    & %$  #  "!`,
          "]  ]]]]]]]]]         ]]]]]]] ]] ]]  ",
          "                             G      "
        ];
        XPathParser.gotoTable = [
          "3456789:;<=>?@ AB  CDEFGH IJ ",
          "                             ",
          "                             ",
          "                             ",
          "L456789:;<=>?@ AB  CDEFGH IJ ",
          "            M        EFGH IJ ",
          "       N;<=>?@ AB  CDEFGH IJ ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "            S        EFGH IJ ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "              e              ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                        h  J ",
          "              i          j   ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "o456789:;<=>?@ ABpqCDEFGH IJ ",
          "                             ",
          "  r6789:;<=>?@ AB  CDEFGH IJ ",
          "   s789:;<=>?@ AB  CDEFGH IJ ",
          "    t89:;<=>?@ AB  CDEFGH IJ ",
          "    u89:;<=>?@ AB  CDEFGH IJ ",
          "     v9:;<=>?@ AB  CDEFGH IJ ",
          "     w9:;<=>?@ AB  CDEFGH IJ ",
          "     x9:;<=>?@ AB  CDEFGH IJ ",
          "     y9:;<=>?@ AB  CDEFGH IJ ",
          "      z:;<=>?@ AB  CDEFGH IJ ",
          "      {:;<=>?@ AB  CDEFGH IJ ",
          "       |;<=>?@ AB  CDEFGH IJ ",
          "       };<=>?@ AB  CDEFGH IJ ",
          "       ~;<=>?@ AB  CDEFGH IJ ",
          "         \x7F=>?@ AB  CDEFGH IJ ",
          "\x80456789:;<=>?@ AB  CDEFGH IJ\x81",
          "            \x82        EFGH IJ ",
          "            \x83        EFGH IJ ",
          "                             ",
          "                     \x84 GH IJ ",
          "                     \x85 GH IJ ",
          "              i          \x86   ",
          "              i          \x87   ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "                             ",
          "o456789:;<=>?@ AB\x8CqCDEFGH IJ ",
          "                             ",
          "                             "
        ];
        XPathParser.productions = [
          [1, 1, 2],
          [2, 1, 3],
          [3, 1, 4],
          [3, 3, 3, -9, 4],
          [4, 1, 5],
          [4, 3, 4, -8, 5],
          [5, 1, 6],
          [5, 3, 5, -22, 6],
          [5, 3, 5, -5, 6],
          [6, 1, 7],
          [6, 3, 6, -23, 7],
          [6, 3, 6, -24, 7],
          [6, 3, 6, -6, 7],
          [6, 3, 6, -7, 7],
          [7, 1, 8],
          [7, 3, 7, -25, 8],
          [7, 3, 7, -26, 8],
          [8, 1, 9],
          [8, 3, 8, -12, 9],
          [8, 3, 8, -11, 9],
          [8, 3, 8, -10, 9],
          [9, 1, 10],
          [9, 2, -26, 9],
          [10, 1, 11],
          [10, 3, 10, -27, 11],
          [11, 1, 12],
          [11, 1, 13],
          [11, 3, 13, -28, 14],
          [11, 3, 13, -4, 14],
          [13, 1, 15],
          [13, 2, 13, 16],
          [15, 1, 17],
          [15, 3, -29, 2, -30],
          [15, 1, -15],
          [15, 1, -16],
          [15, 1, 18],
          [18, 3, -13, -29, -30],
          [18, 4, -13, -29, 19, -30],
          [19, 1, 20],
          [19, 3, 20, -31, 19],
          [20, 1, 2],
          [12, 1, 14],
          [12, 1, 21],
          [21, 1, -28],
          [21, 2, -28, 14],
          [21, 1, 22],
          [14, 1, 23],
          [14, 3, 14, -28, 23],
          [14, 1, 24],
          [23, 2, 25, 26],
          [23, 1, 26],
          [23, 3, 25, 26, 27],
          [23, 2, 26, 27],
          [23, 1, 28],
          [27, 1, 16],
          [27, 2, 16, 27],
          [25, 2, -14, -3],
          [25, 1, -32],
          [26, 1, 29],
          [26, 3, -20, -29, -30],
          [26, 4, -21, -29, -15, -30],
          [16, 3, -33, 30, -34],
          [30, 1, 2],
          [22, 2, -4, 14],
          [24, 3, 14, -4, 23],
          [28, 1, -35],
          [28, 1, -2],
          [17, 2, -36, -18],
          [29, 1, -17],
          [29, 1, -19],
          [29, 1, -18]
        ];
        XPathParser.DOUBLEDOT = 2;
        XPathParser.DOUBLECOLON = 3;
        XPathParser.DOUBLESLASH = 4;
        XPathParser.NOTEQUAL = 5;
        XPathParser.LESSTHANOREQUAL = 6;
        XPathParser.GREATERTHANOREQUAL = 7;
        XPathParser.AND = 8;
        XPathParser.OR = 9;
        XPathParser.MOD = 10;
        XPathParser.DIV = 11;
        XPathParser.MULTIPLYOPERATOR = 12;
        XPathParser.FUNCTIONNAME = 13;
        XPathParser.AXISNAME = 14;
        XPathParser.LITERAL = 15;
        XPathParser.NUMBER = 16;
        XPathParser.ASTERISKNAMETEST = 17;
        XPathParser.QNAME = 18;
        XPathParser.NCNAMECOLONASTERISK = 19;
        XPathParser.NODETYPE = 20;
        XPathParser.PROCESSINGINSTRUCTIONWITHLITERAL = 21;
        XPathParser.EQUALS = 22;
        XPathParser.LESSTHAN = 23;
        XPathParser.GREATERTHAN = 24;
        XPathParser.PLUS = 25;
        XPathParser.MINUS = 26;
        XPathParser.BAR = 27;
        XPathParser.SLASH = 28;
        XPathParser.LEFTPARENTHESIS = 29;
        XPathParser.RIGHTPARENTHESIS = 30;
        XPathParser.COMMA = 31;
        XPathParser.AT = 32;
        XPathParser.LEFTBRACKET = 33;
        XPathParser.RIGHTBRACKET = 34;
        XPathParser.DOT = 35;
        XPathParser.DOLLAR = 36;
        XPathParser.prototype.tokenize = function(s1) {
          var types = [];
          var values = [];
          var s2 = s1 + "\0";
          var pos = 0;
          var c2 = s2.charAt(pos++);
          while (1) {
            while (c2 == " " || c2 == "	" || c2 == "\r" || c2 == "\n") {
              c2 = s2.charAt(pos++);
            }
            if (c2 == "\0" || pos >= s2.length) {
              break;
            }
            if (c2 == "(") {
              types.push(XPathParser.LEFTPARENTHESIS);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == ")") {
              types.push(XPathParser.RIGHTPARENTHESIS);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "[") {
              types.push(XPathParser.LEFTBRACKET);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "]") {
              types.push(XPathParser.RIGHTBRACKET);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "@") {
              types.push(XPathParser.AT);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == ",") {
              types.push(XPathParser.COMMA);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "|") {
              types.push(XPathParser.BAR);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "+") {
              types.push(XPathParser.PLUS);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "-") {
              types.push(XPathParser.MINUS);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "=") {
              types.push(XPathParser.EQUALS);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "$") {
              types.push(XPathParser.DOLLAR);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == ".") {
              c2 = s2.charAt(pos++);
              if (c2 == ".") {
                types.push(XPathParser.DOUBLEDOT);
                values.push("..");
                c2 = s2.charAt(pos++);
                continue;
              }
              if (c2 >= "0" && c2 <= "9") {
                var number = "." + c2;
                c2 = s2.charAt(pos++);
                while (c2 >= "0" && c2 <= "9") {
                  number += c2;
                  c2 = s2.charAt(pos++);
                }
                types.push(XPathParser.NUMBER);
                values.push(number);
                continue;
              }
              types.push(XPathParser.DOT);
              values.push(".");
              continue;
            }
            if (c2 == "'" || c2 == '"') {
              var delimiter = c2;
              var literal = "";
              while (pos < s2.length && (c2 = s2.charAt(pos)) !== delimiter) {
                literal += c2;
                pos += 1;
              }
              if (c2 !== delimiter) {
                throw XPathException.fromMessage("Unterminated string literal: " + delimiter + literal);
              }
              pos += 1;
              types.push(XPathParser.LITERAL);
              values.push(literal);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 >= "0" && c2 <= "9") {
              var number = c2;
              c2 = s2.charAt(pos++);
              while (c2 >= "0" && c2 <= "9") {
                number += c2;
                c2 = s2.charAt(pos++);
              }
              if (c2 == ".") {
                if (s2.charAt(pos) >= "0" && s2.charAt(pos) <= "9") {
                  number += c2;
                  number += s2.charAt(pos++);
                  c2 = s2.charAt(pos++);
                  while (c2 >= "0" && c2 <= "9") {
                    number += c2;
                    c2 = s2.charAt(pos++);
                  }
                }
              }
              types.push(XPathParser.NUMBER);
              values.push(number);
              continue;
            }
            if (c2 == "*") {
              if (types.length > 0) {
                var last = types[types.length - 1];
                if (last != XPathParser.AT && last != XPathParser.DOUBLECOLON && last != XPathParser.LEFTPARENTHESIS && last != XPathParser.LEFTBRACKET && last != XPathParser.AND && last != XPathParser.OR && last != XPathParser.MOD && last != XPathParser.DIV && last != XPathParser.MULTIPLYOPERATOR && last != XPathParser.SLASH && last != XPathParser.DOUBLESLASH && last != XPathParser.BAR && last != XPathParser.PLUS && last != XPathParser.MINUS && last != XPathParser.EQUALS && last != XPathParser.NOTEQUAL && last != XPathParser.LESSTHAN && last != XPathParser.LESSTHANOREQUAL && last != XPathParser.GREATERTHAN && last != XPathParser.GREATERTHANOREQUAL) {
                  types.push(XPathParser.MULTIPLYOPERATOR);
                  values.push(c2);
                  c2 = s2.charAt(pos++);
                  continue;
                }
              }
              types.push(XPathParser.ASTERISKNAMETEST);
              values.push(c2);
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == ":") {
              if (s2.charAt(pos) == ":") {
                types.push(XPathParser.DOUBLECOLON);
                values.push("::");
                pos++;
                c2 = s2.charAt(pos++);
                continue;
              }
            }
            if (c2 == "/") {
              c2 = s2.charAt(pos++);
              if (c2 == "/") {
                types.push(XPathParser.DOUBLESLASH);
                values.push("//");
                c2 = s2.charAt(pos++);
                continue;
              }
              types.push(XPathParser.SLASH);
              values.push("/");
              continue;
            }
            if (c2 == "!") {
              if (s2.charAt(pos) == "=") {
                types.push(XPathParser.NOTEQUAL);
                values.push("!=");
                pos++;
                c2 = s2.charAt(pos++);
                continue;
              }
            }
            if (c2 == "<") {
              if (s2.charAt(pos) == "=") {
                types.push(XPathParser.LESSTHANOREQUAL);
                values.push("<=");
                pos++;
                c2 = s2.charAt(pos++);
                continue;
              }
              types.push(XPathParser.LESSTHAN);
              values.push("<");
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == ">") {
              if (s2.charAt(pos) == "=") {
                types.push(XPathParser.GREATERTHANOREQUAL);
                values.push(">=");
                pos++;
                c2 = s2.charAt(pos++);
                continue;
              }
              types.push(XPathParser.GREATERTHAN);
              values.push(">");
              c2 = s2.charAt(pos++);
              continue;
            }
            if (c2 == "_" || Utilities.isLetter(c2.charCodeAt(0))) {
              var name2 = c2;
              c2 = s2.charAt(pos++);
              while (Utilities.isNCNameChar(c2.charCodeAt(0))) {
                name2 += c2;
                c2 = s2.charAt(pos++);
              }
              if (types.length > 0) {
                var last = types[types.length - 1];
                if (last != XPathParser.AT && last != XPathParser.DOUBLECOLON && last != XPathParser.LEFTPARENTHESIS && last != XPathParser.LEFTBRACKET && last != XPathParser.AND && last != XPathParser.OR && last != XPathParser.MOD && last != XPathParser.DIV && last != XPathParser.MULTIPLYOPERATOR && last != XPathParser.SLASH && last != XPathParser.DOUBLESLASH && last != XPathParser.BAR && last != XPathParser.PLUS && last != XPathParser.MINUS && last != XPathParser.EQUALS && last != XPathParser.NOTEQUAL && last != XPathParser.LESSTHAN && last != XPathParser.LESSTHANOREQUAL && last != XPathParser.GREATERTHAN && last != XPathParser.GREATERTHANOREQUAL) {
                  if (name2 == "and") {
                    types.push(XPathParser.AND);
                    values.push(name2);
                    continue;
                  }
                  if (name2 == "or") {
                    types.push(XPathParser.OR);
                    values.push(name2);
                    continue;
                  }
                  if (name2 == "mod") {
                    types.push(XPathParser.MOD);
                    values.push(name2);
                    continue;
                  }
                  if (name2 == "div") {
                    types.push(XPathParser.DIV);
                    values.push(name2);
                    continue;
                  }
                }
              }
              if (c2 == ":") {
                if (s2.charAt(pos) == "*") {
                  types.push(XPathParser.NCNAMECOLONASTERISK);
                  values.push(name2 + ":*");
                  pos++;
                  c2 = s2.charAt(pos++);
                  continue;
                }
                if (s2.charAt(pos) == "_" || Utilities.isLetter(s2.charCodeAt(pos))) {
                  name2 += ":";
                  c2 = s2.charAt(pos++);
                  while (Utilities.isNCNameChar(c2.charCodeAt(0))) {
                    name2 += c2;
                    c2 = s2.charAt(pos++);
                  }
                  if (c2 == "(") {
                    types.push(XPathParser.FUNCTIONNAME);
                    values.push(name2);
                    continue;
                  }
                  types.push(XPathParser.QNAME);
                  values.push(name2);
                  continue;
                }
                if (s2.charAt(pos) == ":") {
                  types.push(XPathParser.AXISNAME);
                  values.push(name2);
                  continue;
                }
              }
              if (c2 == "(") {
                if (name2 == "comment" || name2 == "text" || name2 == "node") {
                  types.push(XPathParser.NODETYPE);
                  values.push(name2);
                  continue;
                }
                if (name2 == "processing-instruction") {
                  if (s2.charAt(pos) == ")") {
                    types.push(XPathParser.NODETYPE);
                  } else {
                    types.push(XPathParser.PROCESSINGINSTRUCTIONWITHLITERAL);
                  }
                  values.push(name2);
                  continue;
                }
                types.push(XPathParser.FUNCTIONNAME);
                values.push(name2);
                continue;
              }
              types.push(XPathParser.QNAME);
              values.push(name2);
              continue;
            }
            throw new Error("Unexpected character " + c2);
          }
          types.push(1);
          values.push("[EOF]");
          return [types, values];
        };
        XPathParser.SHIFT = "s";
        XPathParser.REDUCE = "r";
        XPathParser.ACCEPT = "a";
        XPathParser.prototype.parse = function(s2) {
          if (!s2) {
            throw new Error("XPath expression unspecified.");
          }
          if (typeof s2 !== "string") {
            throw new Error("XPath expression must be a string.");
          }
          var types;
          var values;
          var res = this.tokenize(s2);
          if (res == void 0) {
            return void 0;
          }
          types = res[0];
          values = res[1];
          var tokenPos = 0;
          var state = [];
          var tokenType = [];
          var tokenValue = [];
          var s2;
          var a2;
          var t2;
          state.push(0);
          tokenType.push(1);
          tokenValue.push("_S");
          a2 = types[tokenPos];
          t2 = values[tokenPos++];
          while (1) {
            s2 = state[state.length - 1];
            switch (XPathParser.actionTable[s2].charAt(a2 - 1)) {
              case XPathParser.SHIFT:
                tokenType.push(-a2);
                tokenValue.push(t2);
                state.push(XPathParser.actionTableNumber[s2].charCodeAt(a2 - 1) - 32);
                a2 = types[tokenPos];
                t2 = values[tokenPos++];
                break;
              case XPathParser.REDUCE:
                var num = XPathParser.productions[XPathParser.actionTableNumber[s2].charCodeAt(a2 - 1) - 32][1];
                var rhs = [];
                for (var i2 = 0; i2 < num; i2++) {
                  tokenType.pop();
                  rhs.unshift(tokenValue.pop());
                  state.pop();
                }
                var s_ = state[state.length - 1];
                tokenType.push(XPathParser.productions[XPathParser.actionTableNumber[s2].charCodeAt(a2 - 1) - 32][0]);
                if (this.reduceActions[XPathParser.actionTableNumber[s2].charCodeAt(a2 - 1) - 32] == void 0) {
                  tokenValue.push(rhs[0]);
                } else {
                  tokenValue.push(this.reduceActions[XPathParser.actionTableNumber[s2].charCodeAt(a2 - 1) - 32](rhs));
                }
                state.push(XPathParser.gotoTable[s_].charCodeAt(XPathParser.productions[XPathParser.actionTableNumber[s2].charCodeAt(a2 - 1) - 32][0] - 2) - 33);
                break;
              case XPathParser.ACCEPT:
                return new XPath(tokenValue.pop());
              default:
                throw new Error("XPath parse error");
            }
          }
        };
        XPath.prototype = new Object();
        XPath.prototype.constructor = XPath;
        XPath.superclass = Object.prototype;
        function XPath(e2) {
          this.expression = e2;
        }
        XPath.prototype.toString = function() {
          return this.expression.toString();
        };
        function setIfUnset(obj, prop, value) {
          if (!(prop in obj)) {
            obj[prop] = value;
          }
        }
        XPath.prototype.evaluate = function(c2) {
          var node = c2.expressionContextNode;
          if (!(isNil(node) || isNodeLike(node))) {
            throw new Error("Context node does not appear to be a valid DOM node.");
          }
          var size = isNil(c2.expressionContextSize) ? 1 : c2.expressionContextSize;
          var position = isNil(c2.expressionContextPosition) ? 1 : c2.expressionContextPosition;
          if (!isPositiveInteger(size) || !isPositiveInteger(position) || position > size) {
            throw new Error("Context position and size must be positive integers, with position no greater than size.");
          }
          c2.contextNode = c2.expressionContextNode;
          c2.contextSize = size;
          c2.contextPosition = position;
          if (c2.isHtml) {
            setIfUnset(c2, "caseInsensitive", true);
            setIfUnset(c2, "allowAnyNamespaceForNoPrefix", true);
          }
          setIfUnset(c2, "caseInsensitive", false);
          return this.expression.evaluate(c2);
        };
        XPath.XML_NAMESPACE_URI = "http://www.w3.org/XML/1998/namespace";
        XPath.XMLNS_NAMESPACE_URI = "http://www.w3.org/2000/xmlns/";
        Expression.prototype = new Object();
        Expression.prototype.constructor = Expression;
        Expression.superclass = Object.prototype;
        function Expression() {
        }
        Expression.prototype.init = function() {
        };
        Expression.prototype.toString = function() {
          return "<Expression>";
        };
        Expression.prototype.evaluate = function(c2) {
          throw new Error("Could not evaluate expression.");
        };
        UnaryOperation.prototype = new Expression();
        UnaryOperation.prototype.constructor = UnaryOperation;
        UnaryOperation.superclass = Expression.prototype;
        function UnaryOperation(rhs) {
          if (arguments.length > 0) {
            this.init(rhs);
          }
        }
        UnaryOperation.prototype.init = function(rhs) {
          this.rhs = rhs;
        };
        UnaryMinusOperation.prototype = new UnaryOperation();
        UnaryMinusOperation.prototype.constructor = UnaryMinusOperation;
        UnaryMinusOperation.superclass = UnaryOperation.prototype;
        function UnaryMinusOperation(rhs) {
          if (arguments.length > 0) {
            this.init(rhs);
          }
        }
        UnaryMinusOperation.prototype.init = function(rhs) {
          UnaryMinusOperation.superclass.init.call(this, rhs);
        };
        UnaryMinusOperation.prototype.evaluate = function(c2) {
          return this.rhs.evaluate(c2).number().negate();
        };
        UnaryMinusOperation.prototype.toString = function() {
          return "-" + this.rhs.toString();
        };
        BinaryOperation.prototype = new Expression();
        BinaryOperation.prototype.constructor = BinaryOperation;
        BinaryOperation.superclass = Expression.prototype;
        function BinaryOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        BinaryOperation.prototype.init = function(lhs, rhs) {
          this.lhs = lhs;
          this.rhs = rhs;
        };
        OrOperation.prototype = new BinaryOperation();
        OrOperation.prototype.constructor = OrOperation;
        OrOperation.superclass = BinaryOperation.prototype;
        function OrOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        OrOperation.prototype.init = function(lhs, rhs) {
          OrOperation.superclass.init.call(this, lhs, rhs);
        };
        OrOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " or " + this.rhs.toString() + ")";
        };
        OrOperation.prototype.evaluate = function(c2) {
          var b2 = this.lhs.evaluate(c2).bool();
          if (b2.booleanValue()) {
            return b2;
          }
          return this.rhs.evaluate(c2).bool();
        };
        AndOperation.prototype = new BinaryOperation();
        AndOperation.prototype.constructor = AndOperation;
        AndOperation.superclass = BinaryOperation.prototype;
        function AndOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        AndOperation.prototype.init = function(lhs, rhs) {
          AndOperation.superclass.init.call(this, lhs, rhs);
        };
        AndOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " and " + this.rhs.toString() + ")";
        };
        AndOperation.prototype.evaluate = function(c2) {
          var b2 = this.lhs.evaluate(c2).bool();
          if (!b2.booleanValue()) {
            return b2;
          }
          return this.rhs.evaluate(c2).bool();
        };
        EqualsOperation.prototype = new BinaryOperation();
        EqualsOperation.prototype.constructor = EqualsOperation;
        EqualsOperation.superclass = BinaryOperation.prototype;
        function EqualsOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        EqualsOperation.prototype.init = function(lhs, rhs) {
          EqualsOperation.superclass.init.call(this, lhs, rhs);
        };
        EqualsOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " = " + this.rhs.toString() + ")";
        };
        EqualsOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).equals(this.rhs.evaluate(c2));
        };
        NotEqualOperation.prototype = new BinaryOperation();
        NotEqualOperation.prototype.constructor = NotEqualOperation;
        NotEqualOperation.superclass = BinaryOperation.prototype;
        function NotEqualOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        NotEqualOperation.prototype.init = function(lhs, rhs) {
          NotEqualOperation.superclass.init.call(this, lhs, rhs);
        };
        NotEqualOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " != " + this.rhs.toString() + ")";
        };
        NotEqualOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).notequal(this.rhs.evaluate(c2));
        };
        LessThanOperation.prototype = new BinaryOperation();
        LessThanOperation.prototype.constructor = LessThanOperation;
        LessThanOperation.superclass = BinaryOperation.prototype;
        function LessThanOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        LessThanOperation.prototype.init = function(lhs, rhs) {
          LessThanOperation.superclass.init.call(this, lhs, rhs);
        };
        LessThanOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).lessthan(this.rhs.evaluate(c2));
        };
        LessThanOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " < " + this.rhs.toString() + ")";
        };
        GreaterThanOperation.prototype = new BinaryOperation();
        GreaterThanOperation.prototype.constructor = GreaterThanOperation;
        GreaterThanOperation.superclass = BinaryOperation.prototype;
        function GreaterThanOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        GreaterThanOperation.prototype.init = function(lhs, rhs) {
          GreaterThanOperation.superclass.init.call(this, lhs, rhs);
        };
        GreaterThanOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).greaterthan(this.rhs.evaluate(c2));
        };
        GreaterThanOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " > " + this.rhs.toString() + ")";
        };
        LessThanOrEqualOperation.prototype = new BinaryOperation();
        LessThanOrEqualOperation.prototype.constructor = LessThanOrEqualOperation;
        LessThanOrEqualOperation.superclass = BinaryOperation.prototype;
        function LessThanOrEqualOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        LessThanOrEqualOperation.prototype.init = function(lhs, rhs) {
          LessThanOrEqualOperation.superclass.init.call(this, lhs, rhs);
        };
        LessThanOrEqualOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).lessthanorequal(this.rhs.evaluate(c2));
        };
        LessThanOrEqualOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " <= " + this.rhs.toString() + ")";
        };
        GreaterThanOrEqualOperation.prototype = new BinaryOperation();
        GreaterThanOrEqualOperation.prototype.constructor = GreaterThanOrEqualOperation;
        GreaterThanOrEqualOperation.superclass = BinaryOperation.prototype;
        function GreaterThanOrEqualOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        GreaterThanOrEqualOperation.prototype.init = function(lhs, rhs) {
          GreaterThanOrEqualOperation.superclass.init.call(this, lhs, rhs);
        };
        GreaterThanOrEqualOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).greaterthanorequal(this.rhs.evaluate(c2));
        };
        GreaterThanOrEqualOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " >= " + this.rhs.toString() + ")";
        };
        PlusOperation.prototype = new BinaryOperation();
        PlusOperation.prototype.constructor = PlusOperation;
        PlusOperation.superclass = BinaryOperation.prototype;
        function PlusOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        PlusOperation.prototype.init = function(lhs, rhs) {
          PlusOperation.superclass.init.call(this, lhs, rhs);
        };
        PlusOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).number().plus(this.rhs.evaluate(c2).number());
        };
        PlusOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " + " + this.rhs.toString() + ")";
        };
        MinusOperation.prototype = new BinaryOperation();
        MinusOperation.prototype.constructor = MinusOperation;
        MinusOperation.superclass = BinaryOperation.prototype;
        function MinusOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        MinusOperation.prototype.init = function(lhs, rhs) {
          MinusOperation.superclass.init.call(this, lhs, rhs);
        };
        MinusOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).number().minus(this.rhs.evaluate(c2).number());
        };
        MinusOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " - " + this.rhs.toString() + ")";
        };
        MultiplyOperation.prototype = new BinaryOperation();
        MultiplyOperation.prototype.constructor = MultiplyOperation;
        MultiplyOperation.superclass = BinaryOperation.prototype;
        function MultiplyOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        MultiplyOperation.prototype.init = function(lhs, rhs) {
          MultiplyOperation.superclass.init.call(this, lhs, rhs);
        };
        MultiplyOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).number().multiply(this.rhs.evaluate(c2).number());
        };
        MultiplyOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " * " + this.rhs.toString() + ")";
        };
        DivOperation.prototype = new BinaryOperation();
        DivOperation.prototype.constructor = DivOperation;
        DivOperation.superclass = BinaryOperation.prototype;
        function DivOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        DivOperation.prototype.init = function(lhs, rhs) {
          DivOperation.superclass.init.call(this, lhs, rhs);
        };
        DivOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).number().div(this.rhs.evaluate(c2).number());
        };
        DivOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " div " + this.rhs.toString() + ")";
        };
        ModOperation.prototype = new BinaryOperation();
        ModOperation.prototype.constructor = ModOperation;
        ModOperation.superclass = BinaryOperation.prototype;
        function ModOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        ModOperation.prototype.init = function(lhs, rhs) {
          ModOperation.superclass.init.call(this, lhs, rhs);
        };
        ModOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).number().mod(this.rhs.evaluate(c2).number());
        };
        ModOperation.prototype.toString = function() {
          return "(" + this.lhs.toString() + " mod " + this.rhs.toString() + ")";
        };
        BarOperation.prototype = new BinaryOperation();
        BarOperation.prototype.constructor = BarOperation;
        BarOperation.superclass = BinaryOperation.prototype;
        function BarOperation(lhs, rhs) {
          if (arguments.length > 0) {
            this.init(lhs, rhs);
          }
        }
        BarOperation.prototype.init = function(lhs, rhs) {
          BarOperation.superclass.init.call(this, lhs, rhs);
        };
        BarOperation.prototype.evaluate = function(c2) {
          return this.lhs.evaluate(c2).nodeset().union(this.rhs.evaluate(c2).nodeset());
        };
        BarOperation.prototype.toString = function() {
          return map(toString, [this.lhs, this.rhs]).join(" | ");
        };
        PathExpr.prototype = new Expression();
        PathExpr.prototype.constructor = PathExpr;
        PathExpr.superclass = Expression.prototype;
        function PathExpr(filter2, filterPreds, locpath) {
          if (arguments.length > 0) {
            this.init(filter2, filterPreds, locpath);
          }
        }
        PathExpr.prototype.init = function(filter2, filterPreds, locpath) {
          PathExpr.superclass.init.call(this);
          this.filter = filter2;
          this.filterPredicates = filterPreds;
          this.locationPath = locpath;
        };
        function findRoot(node) {
          while (node && node.parentNode) {
            node = node.parentNode;
          }
          return node;
        }
        var applyPredicates = function(predicates, c2, nodes, reverse) {
          if (predicates.length === 0) {
            return nodes;
          }
          var ctx = c2.extend({});
          return reduce(
            function(inNodes, pred) {
              ctx.contextSize = inNodes.length;
              return filter(
                function(node, i2) {
                  ctx.contextNode = node;
                  ctx.contextPosition = i2 + 1;
                  return PathExpr.predicateMatches(pred, ctx);
                },
                inNodes
              );
            },
            sortNodes(nodes, reverse),
            predicates
          );
        };
        PathExpr.getRoot = function(xpc, nodes) {
          var firstNode = nodes[0];
          if (firstNode && firstNode.nodeType === NodeTypes.DOCUMENT_NODE) {
            return firstNode;
          }
          if (xpc.virtualRoot) {
            return xpc.virtualRoot;
          }
          if (!firstNode) {
            throw new Error("Context node not found when determining document root.");
          }
          var ownerDoc = firstNode.ownerDocument;
          if (ownerDoc) {
            return ownerDoc;
          }
          var n2 = firstNode;
          while (n2.parentNode != null) {
            n2 = n2.parentNode;
          }
          return n2;
        };
        var getPrefixForNamespaceNode = function(attrNode) {
          var nm = String(attrNode.name);
          if (nm === "xmlns") {
            return "";
          }
          if (nm.substring(0, 6) === "xmlns:") {
            return nm.substring(6, nm.length);
          }
          return null;
        };
        PathExpr.applyStep = function(step, xpc, node) {
          if (!node) {
            throw new Error("Context node not found when evaluating XPath step: " + step);
          }
          var newNodes = [];
          xpc.contextNode = node;
          switch (step.axis) {
            case Step.ANCESTOR:
              if (xpc.contextNode === xpc.virtualRoot) {
                break;
              }
              var m2;
              if (xpc.contextNode.nodeType == NodeTypes.ATTRIBUTE_NODE) {
                m2 = PathExpr.getOwnerElement(xpc.contextNode);
              } else {
                m2 = xpc.contextNode.parentNode;
              }
              while (m2 != null) {
                if (step.nodeTest.matches(m2, xpc)) {
                  newNodes.push(m2);
                }
                if (m2 === xpc.virtualRoot) {
                  break;
                }
                m2 = m2.parentNode;
              }
              break;
            case Step.ANCESTORORSELF:
              for (var m2 = xpc.contextNode; m2 != null; m2 = m2.nodeType == NodeTypes.ATTRIBUTE_NODE ? PathExpr.getOwnerElement(m2) : m2.parentNode) {
                if (step.nodeTest.matches(m2, xpc)) {
                  newNodes.push(m2);
                }
                if (m2 === xpc.virtualRoot) {
                  break;
                }
              }
              break;
            case Step.ATTRIBUTE:
              var nnm = xpc.contextNode.attributes;
              if (nnm != null) {
                for (var k2 = 0; k2 < nnm.length; k2++) {
                  var m2 = nnm.item(k2);
                  if (step.nodeTest.matches(m2, xpc)) {
                    newNodes.push(m2);
                  }
                }
              }
              break;
            case Step.CHILD:
              for (var m2 = xpc.contextNode.firstChild; m2 != null; m2 = m2.nextSibling) {
                if (step.nodeTest.matches(m2, xpc)) {
                  newNodes.push(m2);
                }
              }
              break;
            case Step.DESCENDANT:
              var st2 = [xpc.contextNode.firstChild];
              while (st2.length > 0) {
                for (var m2 = st2.pop(); m2 != null; ) {
                  if (step.nodeTest.matches(m2, xpc)) {
                    newNodes.push(m2);
                  }
                  if (m2.firstChild != null) {
                    st2.push(m2.nextSibling);
                    m2 = m2.firstChild;
                  } else {
                    m2 = m2.nextSibling;
                  }
                }
              }
              break;
            case Step.DESCENDANTORSELF:
              if (step.nodeTest.matches(xpc.contextNode, xpc)) {
                newNodes.push(xpc.contextNode);
              }
              var st2 = [xpc.contextNode.firstChild];
              while (st2.length > 0) {
                for (var m2 = st2.pop(); m2 != null; ) {
                  if (step.nodeTest.matches(m2, xpc)) {
                    newNodes.push(m2);
                  }
                  if (m2.firstChild != null) {
                    st2.push(m2.nextSibling);
                    m2 = m2.firstChild;
                  } else {
                    m2 = m2.nextSibling;
                  }
                }
              }
              break;
            case Step.FOLLOWING:
              if (xpc.contextNode === xpc.virtualRoot) {
                break;
              }
              var st2 = [];
              if (xpc.contextNode.firstChild != null) {
                st2.unshift(xpc.contextNode.firstChild);
              } else {
                st2.unshift(xpc.contextNode.nextSibling);
              }
              for (var m2 = xpc.contextNode.parentNode; m2 != null && m2.nodeType != NodeTypes.DOCUMENT_NODE && m2 !== xpc.virtualRoot; m2 = m2.parentNode) {
                st2.unshift(m2.nextSibling);
              }
              do {
                for (var m2 = st2.pop(); m2 != null; ) {
                  if (step.nodeTest.matches(m2, xpc)) {
                    newNodes.push(m2);
                  }
                  if (m2.firstChild != null) {
                    st2.push(m2.nextSibling);
                    m2 = m2.firstChild;
                  } else {
                    m2 = m2.nextSibling;
                  }
                }
              } while (st2.length > 0);
              break;
            case Step.FOLLOWINGSIBLING:
              if (xpc.contextNode === xpc.virtualRoot) {
                break;
              }
              for (var m2 = xpc.contextNode.nextSibling; m2 != null; m2 = m2.nextSibling) {
                if (step.nodeTest.matches(m2, xpc)) {
                  newNodes.push(m2);
                }
              }
              break;
            case Step.NAMESPACE:
              var nodes = {};
              if (xpc.contextNode.nodeType == NodeTypes.ELEMENT_NODE) {
                nodes["xml"] = new XPathNamespace("xml", null, XPath.XML_NAMESPACE_URI, xpc.contextNode);
                for (var m2 = xpc.contextNode; m2 != null && m2.nodeType == NodeTypes.ELEMENT_NODE; m2 = m2.parentNode) {
                  for (var k2 = 0; k2 < m2.attributes.length; k2++) {
                    var attr = m2.attributes.item(k2);
                    var pre = getPrefixForNamespaceNode(attr);
                    if (pre != null && nodes[pre] == void 0) {
                      nodes[pre] = new XPathNamespace(pre, attr, attr.value, xpc.contextNode);
                    }
                  }
                }
                for (var pre in nodes) {
                  var node = nodes[pre];
                  if (step.nodeTest.matches(node, xpc)) {
                    newNodes.push(node);
                  }
                }
              }
              break;
            case Step.PARENT:
              m2 = null;
              if (xpc.contextNode !== xpc.virtualRoot) {
                if (xpc.contextNode.nodeType == NodeTypes.ATTRIBUTE_NODE) {
                  m2 = PathExpr.getOwnerElement(xpc.contextNode);
                } else {
                  m2 = xpc.contextNode.parentNode;
                }
              }
              if (m2 != null && step.nodeTest.matches(m2, xpc)) {
                newNodes.push(m2);
              }
              break;
            case Step.PRECEDING:
              var st2;
              if (xpc.virtualRoot != null) {
                st2 = [xpc.virtualRoot];
              } else {
                st2 = [findRoot(xpc.contextNode)];
              }
              outer: while (st2.length > 0) {
                for (var m2 = st2.pop(); m2 != null; ) {
                  if (m2 == xpc.contextNode) {
                    break outer;
                  }
                  if (step.nodeTest.matches(m2, xpc)) {
                    newNodes.unshift(m2);
                  }
                  if (m2.firstChild != null) {
                    st2.push(m2.nextSibling);
                    m2 = m2.firstChild;
                  } else {
                    m2 = m2.nextSibling;
                  }
                }
              }
              break;
            case Step.PRECEDINGSIBLING:
              if (xpc.contextNode === xpc.virtualRoot) {
                break;
              }
              for (var m2 = xpc.contextNode.previousSibling; m2 != null; m2 = m2.previousSibling) {
                if (step.nodeTest.matches(m2, xpc)) {
                  newNodes.push(m2);
                }
              }
              break;
            case Step.SELF:
              if (step.nodeTest.matches(xpc.contextNode, xpc)) {
                newNodes.push(xpc.contextNode);
              }
              break;
            default:
          }
          return newNodes;
        };
        function applyStepWithPredicates(step, xpc, node) {
          return applyPredicates(
            step.predicates,
            xpc,
            PathExpr.applyStep(step, xpc, node),
            includes(REVERSE_AXES, step.axis)
          );
        }
        function applyStepToNodes(context, nodes, step) {
          return flatten(
            map(
              applyStepWithPredicates.bind(null, step, context),
              nodes
            )
          );
        }
        PathExpr.applySteps = function(steps, xpc, nodes) {
          return reduce(
            applyStepToNodes.bind(null, xpc),
            nodes,
            steps
          );
        };
        PathExpr.prototype.applyFilter = function(c2, xpc) {
          if (!this.filter) {
            return { nodes: [c2.contextNode] };
          }
          var ns = this.filter.evaluate(c2);
          if (!Utilities.instance_of(ns, XNodeSet)) {
            if (this.filterPredicates != null && this.filterPredicates.length > 0 || this.locationPath != null) {
              throw new Error("Path expression filter must evaluate to a nodeset if predicates or location path are used");
            }
            return { nonNodes: ns };
          }
          return {
            nodes: applyPredicates(
              this.filterPredicates || [],
              xpc,
              ns.toUnsortedArray(),
              false
              // reverse
            )
          };
        };
        PathExpr.applyLocationPath = function(locationPath, xpc, nodes) {
          if (!locationPath) {
            return nodes;
          }
          var startNodes = locationPath.absolute ? [PathExpr.getRoot(xpc, nodes)] : nodes;
          return PathExpr.applySteps(locationPath.steps, xpc, startNodes);
        };
        PathExpr.prototype.evaluate = function(c2) {
          var xpc = assign(new XPathContext(), c2);
          var filterResult = this.applyFilter(c2, xpc);
          if ("nonNodes" in filterResult) {
            return filterResult.nonNodes;
          }
          var ns = new XNodeSet();
          ns.addArray(PathExpr.applyLocationPath(this.locationPath, xpc, filterResult.nodes));
          return ns;
        };
        PathExpr.predicateMatches = function(pred, c2) {
          var res = pred.evaluate(c2);
          return Utilities.instance_of(res, XNumber) ? c2.contextPosition === res.numberValue() : res.booleanValue();
        };
        PathExpr.predicateString = function(predicate) {
          return wrap("[", "]", predicate.toString());
        };
        PathExpr.predicatesString = function(predicates) {
          return join(
            "",
            map(PathExpr.predicateString, predicates)
          );
        };
        PathExpr.prototype.toString = function() {
          if (this.filter != void 0) {
            var filterStr = toString(this.filter);
            if (Utilities.instance_of(this.filter, XString)) {
              return wrap("'", "'", filterStr);
            }
            if (this.filterPredicates != void 0 && this.filterPredicates.length) {
              return wrap("(", ")", filterStr) + PathExpr.predicatesString(this.filterPredicates);
            }
            if (this.locationPath != void 0) {
              return filterStr + (this.locationPath.absolute ? "" : "/") + toString(this.locationPath);
            }
            return filterStr;
          }
          return toString(this.locationPath);
        };
        PathExpr.getOwnerElement = function(n2) {
          if (n2.ownerElement) {
            return n2.ownerElement;
          }
          try {
            if (n2.selectSingleNode) {
              return n2.selectSingleNode("..");
            }
          } catch (e2) {
          }
          var doc = n2.nodeType == NodeTypes.DOCUMENT_NODE ? n2 : n2.ownerDocument;
          var elts = doc.getElementsByTagName("*");
          for (var i2 = 0; i2 < elts.length; i2++) {
            var elt = elts.item(i2);
            var nnm = elt.attributes;
            for (var j2 = 0; j2 < nnm.length; j2++) {
              var an = nnm.item(j2);
              if (an === n2) {
                return elt;
              }
            }
          }
          return null;
        };
        LocationPath.prototype = new Object();
        LocationPath.prototype.constructor = LocationPath;
        LocationPath.superclass = Object.prototype;
        function LocationPath(abs, steps) {
          if (arguments.length > 0) {
            this.init(abs, steps);
          }
        }
        LocationPath.prototype.init = function(abs, steps) {
          this.absolute = abs;
          this.steps = steps;
        };
        LocationPath.prototype.toString = function() {
          return (this.absolute ? "/" : "") + map(toString, this.steps).join("/");
        };
        Step.prototype = new Object();
        Step.prototype.constructor = Step;
        Step.superclass = Object.prototype;
        function Step(axis, nodetest, preds) {
          if (arguments.length > 0) {
            this.init(axis, nodetest, preds);
          }
        }
        Step.prototype.init = function(axis, nodetest, preds) {
          this.axis = axis;
          this.nodeTest = nodetest;
          this.predicates = preds;
        };
        Step.prototype.toString = function() {
          return Step.STEPNAMES[this.axis] + "::" + this.nodeTest.toString() + PathExpr.predicatesString(this.predicates);
        };
        Step.ANCESTOR = 0;
        Step.ANCESTORORSELF = 1;
        Step.ATTRIBUTE = 2;
        Step.CHILD = 3;
        Step.DESCENDANT = 4;
        Step.DESCENDANTORSELF = 5;
        Step.FOLLOWING = 6;
        Step.FOLLOWINGSIBLING = 7;
        Step.NAMESPACE = 8;
        Step.PARENT = 9;
        Step.PRECEDING = 10;
        Step.PRECEDINGSIBLING = 11;
        Step.SELF = 12;
        Step.STEPNAMES = reduce(function(acc, x2) {
          return acc[x2[0]] = x2[1], acc;
        }, {}, [
          [Step.ANCESTOR, "ancestor"],
          [Step.ANCESTORORSELF, "ancestor-or-self"],
          [Step.ATTRIBUTE, "attribute"],
          [Step.CHILD, "child"],
          [Step.DESCENDANT, "descendant"],
          [Step.DESCENDANTORSELF, "descendant-or-self"],
          [Step.FOLLOWING, "following"],
          [Step.FOLLOWINGSIBLING, "following-sibling"],
          [Step.NAMESPACE, "namespace"],
          [Step.PARENT, "parent"],
          [Step.PRECEDING, "preceding"],
          [Step.PRECEDINGSIBLING, "preceding-sibling"],
          [Step.SELF, "self"]
        ]);
        var REVERSE_AXES = [
          Step.ANCESTOR,
          Step.ANCESTORORSELF,
          Step.PARENT,
          Step.PRECEDING,
          Step.PRECEDINGSIBLING
        ];
        NodeTest.prototype = new Object();
        NodeTest.prototype.constructor = NodeTest;
        NodeTest.superclass = Object.prototype;
        function NodeTest(type, value) {
          if (arguments.length > 0) {
            this.init(type, value);
          }
        }
        NodeTest.prototype.init = function(type, value) {
          this.type = type;
          this.value = value;
        };
        NodeTest.prototype.toString = function() {
          return "<unknown nodetest type>";
        };
        NodeTest.prototype.matches = function(n2, xpc) {
          console.warn("unknown node test type");
        };
        NodeTest.NAMETESTANY = 0;
        NodeTest.NAMETESTPREFIXANY = 1;
        NodeTest.NAMETESTQNAME = 2;
        NodeTest.COMMENT = 3;
        NodeTest.TEXT = 4;
        NodeTest.PI = 5;
        NodeTest.NODE = 6;
        NodeTest.isNodeType = function(types) {
          return function(node) {
            return includes(types, node.nodeType);
          };
        };
        NodeTest.makeNodeTestType = function(type, members, ctor) {
          var newType = ctor || function() {
          };
          newType.prototype = new NodeTest(type);
          newType.prototype.constructor = newType;
          assign(newType.prototype, members);
          return newType;
        };
        NodeTest.makeNodeTypeTest = function(type, nodeTypes, stringVal) {
          return new (NodeTest.makeNodeTestType(type, {
            matches: NodeTest.isNodeType(nodeTypes),
            toString: always(stringVal)
          }))();
        };
        NodeTest.hasPrefix = function(node) {
          return node.prefix || (node.nodeName || node.tagName).indexOf(":") !== -1;
        };
        NodeTest.isElementOrAttribute = NodeTest.isNodeType([1, 2]);
        NodeTest.nameSpaceMatches = function(prefix, xpc, n2) {
          var nNamespace = n2.namespaceURI || "";
          if (!prefix) {
            return !nNamespace || xpc.allowAnyNamespaceForNoPrefix && !NodeTest.hasPrefix(n2);
          }
          var ns = xpc.namespaceResolver.getNamespace(prefix, xpc.expressionContextNode);
          if (ns == null) {
            throw new Error("Cannot resolve QName " + prefix);
          }
          return ns === nNamespace;
        };
        NodeTest.localNameMatches = function(localName, xpc, n2) {
          var nLocalName = n2.localName || n2.nodeName;
          return xpc.caseInsensitive ? localName.toLowerCase() === nLocalName.toLowerCase() : localName === nLocalName;
        };
        NodeTest.NameTestPrefixAny = NodeTest.makeNodeTestType(
          NodeTest.NAMETESTPREFIXANY,
          {
            matches: function(n2, xpc) {
              return NodeTest.isElementOrAttribute(n2) && NodeTest.nameSpaceMatches(this.prefix, xpc, n2);
            },
            toString: function() {
              return this.prefix + ":*";
            }
          },
          function NameTestPrefixAny(prefix) {
            this.prefix = prefix;
          }
        );
        NodeTest.NameTestQName = NodeTest.makeNodeTestType(
          NodeTest.NAMETESTQNAME,
          {
            matches: function(n2, xpc) {
              return NodeTest.isNodeType(
                [
                  NodeTypes.ELEMENT_NODE,
                  NodeTypes.ATTRIBUTE_NODE,
                  NodeTypes.NAMESPACE_NODE
                ]
              )(n2) && NodeTest.nameSpaceMatches(this.prefix, xpc, n2) && NodeTest.localNameMatches(this.localName, xpc, n2);
            },
            toString: function() {
              return this.name;
            }
          },
          function NameTestQName(name2) {
            var nameParts = name2.split(":");
            this.name = name2;
            this.prefix = nameParts.length > 1 ? nameParts[0] : null;
            this.localName = nameParts[nameParts.length > 1 ? 1 : 0];
          }
        );
        NodeTest.PITest = NodeTest.makeNodeTestType(NodeTest.PI, {
          matches: function(n2, xpc) {
            return NodeTest.isNodeType(
              [NodeTypes.PROCESSING_INSTRUCTION_NODE]
            )(n2) && (n2.target || n2.nodeName) === this.name;
          },
          toString: function() {
            return wrap('processing-instruction("', '")', this.name);
          }
        }, function(name2) {
          this.name = name2;
        });
        NodeTest.nameTestAny = NodeTest.makeNodeTypeTest(
          NodeTest.NAMETESTANY,
          [
            NodeTypes.ELEMENT_NODE,
            NodeTypes.ATTRIBUTE_NODE,
            NodeTypes.NAMESPACE_NODE
          ],
          "*"
        );
        NodeTest.textTest = NodeTest.makeNodeTypeTest(
          NodeTest.TEXT,
          [
            NodeTypes.TEXT_NODE,
            NodeTypes.CDATA_SECTION_NODE
          ],
          "text()"
        );
        NodeTest.commentTest = NodeTest.makeNodeTypeTest(
          NodeTest.COMMENT,
          [NodeTypes.COMMENT_NODE],
          "comment()"
        );
        NodeTest.nodeTest = NodeTest.makeNodeTypeTest(
          NodeTest.NODE,
          [
            NodeTypes.ELEMENT_NODE,
            NodeTypes.ATTRIBUTE_NODE,
            NodeTypes.TEXT_NODE,
            NodeTypes.CDATA_SECTION_NODE,
            NodeTypes.PROCESSING_INSTRUCTION_NODE,
            NodeTypes.COMMENT_NODE,
            NodeTypes.DOCUMENT_NODE
          ],
          "node()"
        );
        NodeTest.anyPiTest = NodeTest.makeNodeTypeTest(
          NodeTest.PI,
          [NodeTypes.PROCESSING_INSTRUCTION_NODE],
          "processing-instruction()"
        );
        VariableReference.prototype = new Expression();
        VariableReference.prototype.constructor = VariableReference;
        VariableReference.superclass = Expression.prototype;
        function VariableReference(v2) {
          if (arguments.length > 0) {
            this.init(v2);
          }
        }
        VariableReference.prototype.init = function(v2) {
          this.variable = v2;
        };
        VariableReference.prototype.toString = function() {
          return "$" + this.variable;
        };
        VariableReference.prototype.evaluate = function(c2) {
          var parts = Utilities.resolveQName(this.variable, c2.namespaceResolver, c2.contextNode, false);
          if (parts[0] == null) {
            throw new Error("Cannot resolve QName " + fn);
          }
          var result = c2.variableResolver.getVariable(parts[1], parts[0]);
          if (!result) {
            throw XPathException.fromMessage("Undeclared variable: " + this.toString());
          }
          return result;
        };
        FunctionCall.prototype = new Expression();
        FunctionCall.prototype.constructor = FunctionCall;
        FunctionCall.superclass = Expression.prototype;
        function FunctionCall(fn2, args) {
          if (arguments.length > 0) {
            this.init(fn2, args);
          }
        }
        FunctionCall.prototype.init = function(fn2, args) {
          this.functionName = fn2;
          this.arguments = args;
        };
        FunctionCall.prototype.toString = function() {
          var s2 = this.functionName + "(";
          for (var i2 = 0; i2 < this.arguments.length; i2++) {
            if (i2 > 0) {
              s2 += ", ";
            }
            s2 += this.arguments[i2].toString();
          }
          return s2 + ")";
        };
        FunctionCall.prototype.evaluate = function(c2) {
          var f2 = FunctionResolver.getFunctionFromContext(this.functionName, c2);
          if (!f2) {
            throw new Error("Unknown function " + this.functionName);
          }
          var a2 = [c2].concat(this.arguments);
          return f2.apply(c2.functionResolver.thisArg, a2);
        };
        var Operators = new Object();
        Operators.equals = function(l2, r2) {
          return l2.equals(r2);
        };
        Operators.notequal = function(l2, r2) {
          return l2.notequal(r2);
        };
        Operators.lessthan = function(l2, r2) {
          return l2.lessthan(r2);
        };
        Operators.greaterthan = function(l2, r2) {
          return l2.greaterthan(r2);
        };
        Operators.lessthanorequal = function(l2, r2) {
          return l2.lessthanorequal(r2);
        };
        Operators.greaterthanorequal = function(l2, r2) {
          return l2.greaterthanorequal(r2);
        };
        XString.prototype = new Expression();
        XString.prototype.constructor = XString;
        XString.superclass = Expression.prototype;
        function XString(s2) {
          if (arguments.length > 0) {
            this.init(s2);
          }
        }
        XString.prototype.init = function(s2) {
          this.str = String(s2);
        };
        XString.prototype.toString = function() {
          return this.str;
        };
        XString.prototype.evaluate = function(c2) {
          return this;
        };
        XString.prototype.string = function() {
          return this;
        };
        XString.prototype.number = function() {
          return new XNumber(this.str);
        };
        XString.prototype.bool = function() {
          return new XBoolean(this.str);
        };
        XString.prototype.nodeset = function() {
          throw new Error("Cannot convert string to nodeset");
        };
        XString.prototype.stringValue = function() {
          return this.str;
        };
        XString.prototype.numberValue = function() {
          return this.number().numberValue();
        };
        XString.prototype.booleanValue = function() {
          return this.bool().booleanValue();
        };
        XString.prototype.equals = function(r2) {
          if (Utilities.instance_of(r2, XBoolean)) {
            return this.bool().equals(r2);
          }
          if (Utilities.instance_of(r2, XNumber)) {
            return this.number().equals(r2);
          }
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithString(this, Operators.equals);
          }
          return new XBoolean(this.str == r2.str);
        };
        XString.prototype.notequal = function(r2) {
          if (Utilities.instance_of(r2, XBoolean)) {
            return this.bool().notequal(r2);
          }
          if (Utilities.instance_of(r2, XNumber)) {
            return this.number().notequal(r2);
          }
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithString(this, Operators.notequal);
          }
          return new XBoolean(this.str != r2.str);
        };
        XString.prototype.lessthan = function(r2) {
          return this.number().lessthan(r2);
        };
        XString.prototype.greaterthan = function(r2) {
          return this.number().greaterthan(r2);
        };
        XString.prototype.lessthanorequal = function(r2) {
          return this.number().lessthanorequal(r2);
        };
        XString.prototype.greaterthanorequal = function(r2) {
          return this.number().greaterthanorequal(r2);
        };
        XNumber.prototype = new Expression();
        XNumber.prototype.constructor = XNumber;
        XNumber.superclass = Expression.prototype;
        function XNumber(n2) {
          if (arguments.length > 0) {
            this.init(n2);
          }
        }
        XNumber.prototype.init = function(n2) {
          this.num = typeof n2 === "string" ? this.parse(n2) : Number(n2);
        };
        XNumber.prototype.numberFormat = /^\s*-?[0-9]*\.?[0-9]+\s*$/;
        XNumber.prototype.parse = function(s2) {
          return this.numberFormat.test(s2) ? parseFloat(s2) : Number.NaN;
        };
        function padSmallNumber(numberStr) {
          var parts = numberStr.split("e-");
          var base = parts[0].replace(".", "");
          var exponent = Number(parts[1]);
          for (var i2 = 0; i2 < exponent - 1; i2 += 1) {
            base = "0" + base;
          }
          return "0." + base;
        }
        function padLargeNumber(numberStr) {
          var parts = numberStr.split("e");
          var base = parts[0].replace(".", "");
          var exponent = Number(parts[1]);
          var zerosToAppend = exponent + 1 - base.length;
          for (var i2 = 0; i2 < zerosToAppend; i2 += 1) {
            base += "0";
          }
          return base;
        }
        XNumber.prototype.toString = function() {
          var strValue = this.num.toString();
          if (strValue.indexOf("e-") !== -1) {
            return padSmallNumber(strValue);
          }
          if (strValue.indexOf("e") !== -1) {
            return padLargeNumber(strValue);
          }
          return strValue;
        };
        XNumber.prototype.evaluate = function(c2) {
          return this;
        };
        XNumber.prototype.string = function() {
          return new XString(this.toString());
        };
        XNumber.prototype.number = function() {
          return this;
        };
        XNumber.prototype.bool = function() {
          return new XBoolean(this.num);
        };
        XNumber.prototype.nodeset = function() {
          throw new Error("Cannot convert number to nodeset");
        };
        XNumber.prototype.stringValue = function() {
          return this.string().stringValue();
        };
        XNumber.prototype.numberValue = function() {
          return this.num;
        };
        XNumber.prototype.booleanValue = function() {
          return this.bool().booleanValue();
        };
        XNumber.prototype.negate = function() {
          return new XNumber(-this.num);
        };
        XNumber.prototype.equals = function(r2) {
          if (Utilities.instance_of(r2, XBoolean)) {
            return this.bool().equals(r2);
          }
          if (Utilities.instance_of(r2, XString)) {
            return this.equals(r2.number());
          }
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithNumber(this, Operators.equals);
          }
          return new XBoolean(this.num == r2.num);
        };
        XNumber.prototype.notequal = function(r2) {
          if (Utilities.instance_of(r2, XBoolean)) {
            return this.bool().notequal(r2);
          }
          if (Utilities.instance_of(r2, XString)) {
            return this.notequal(r2.number());
          }
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithNumber(this, Operators.notequal);
          }
          return new XBoolean(this.num != r2.num);
        };
        XNumber.prototype.lessthan = function(r2) {
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithNumber(this, Operators.greaterthan);
          }
          if (Utilities.instance_of(r2, XBoolean) || Utilities.instance_of(r2, XString)) {
            return this.lessthan(r2.number());
          }
          return new XBoolean(this.num < r2.num);
        };
        XNumber.prototype.greaterthan = function(r2) {
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithNumber(this, Operators.lessthan);
          }
          if (Utilities.instance_of(r2, XBoolean) || Utilities.instance_of(r2, XString)) {
            return this.greaterthan(r2.number());
          }
          return new XBoolean(this.num > r2.num);
        };
        XNumber.prototype.lessthanorequal = function(r2) {
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithNumber(this, Operators.greaterthanorequal);
          }
          if (Utilities.instance_of(r2, XBoolean) || Utilities.instance_of(r2, XString)) {
            return this.lessthanorequal(r2.number());
          }
          return new XBoolean(this.num <= r2.num);
        };
        XNumber.prototype.greaterthanorequal = function(r2) {
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithNumber(this, Operators.lessthanorequal);
          }
          if (Utilities.instance_of(r2, XBoolean) || Utilities.instance_of(r2, XString)) {
            return this.greaterthanorequal(r2.number());
          }
          return new XBoolean(this.num >= r2.num);
        };
        XNumber.prototype.plus = function(r2) {
          return new XNumber(this.num + r2.num);
        };
        XNumber.prototype.minus = function(r2) {
          return new XNumber(this.num - r2.num);
        };
        XNumber.prototype.multiply = function(r2) {
          return new XNumber(this.num * r2.num);
        };
        XNumber.prototype.div = function(r2) {
          return new XNumber(this.num / r2.num);
        };
        XNumber.prototype.mod = function(r2) {
          return new XNumber(this.num % r2.num);
        };
        XBoolean.prototype = new Expression();
        XBoolean.prototype.constructor = XBoolean;
        XBoolean.superclass = Expression.prototype;
        function XBoolean(b2) {
          if (arguments.length > 0) {
            this.init(b2);
          }
        }
        XBoolean.prototype.init = function(b2) {
          this.b = Boolean(b2);
        };
        XBoolean.prototype.toString = function() {
          return this.b.toString();
        };
        XBoolean.prototype.evaluate = function(c2) {
          return this;
        };
        XBoolean.prototype.string = function() {
          return new XString(this.b);
        };
        XBoolean.prototype.number = function() {
          return new XNumber(this.b);
        };
        XBoolean.prototype.bool = function() {
          return this;
        };
        XBoolean.prototype.nodeset = function() {
          throw new Error("Cannot convert boolean to nodeset");
        };
        XBoolean.prototype.stringValue = function() {
          return this.string().stringValue();
        };
        XBoolean.prototype.numberValue = function() {
          return this.number().numberValue();
        };
        XBoolean.prototype.booleanValue = function() {
          return this.b;
        };
        XBoolean.prototype.not = function() {
          return new XBoolean(!this.b);
        };
        XBoolean.prototype.equals = function(r2) {
          if (Utilities.instance_of(r2, XString) || Utilities.instance_of(r2, XNumber)) {
            return this.equals(r2.bool());
          }
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithBoolean(this, Operators.equals);
          }
          return new XBoolean(this.b == r2.b);
        };
        XBoolean.prototype.notequal = function(r2) {
          if (Utilities.instance_of(r2, XString) || Utilities.instance_of(r2, XNumber)) {
            return this.notequal(r2.bool());
          }
          if (Utilities.instance_of(r2, XNodeSet)) {
            return r2.compareWithBoolean(this, Operators.notequal);
          }
          return new XBoolean(this.b != r2.b);
        };
        XBoolean.prototype.lessthan = function(r2) {
          return this.number().lessthan(r2);
        };
        XBoolean.prototype.greaterthan = function(r2) {
          return this.number().greaterthan(r2);
        };
        XBoolean.prototype.lessthanorequal = function(r2) {
          return this.number().lessthanorequal(r2);
        };
        XBoolean.prototype.greaterthanorequal = function(r2) {
          return this.number().greaterthanorequal(r2);
        };
        XBoolean.true_ = new XBoolean(true);
        XBoolean.false_ = new XBoolean(false);
        AVLTree.prototype = new Object();
        AVLTree.prototype.constructor = AVLTree;
        AVLTree.superclass = Object.prototype;
        function AVLTree(n2) {
          this.init(n2);
        }
        AVLTree.prototype.init = function(n2) {
          this.left = null;
          this.right = null;
          this.node = n2;
          this.depth = 1;
        };
        AVLTree.prototype.balance = function() {
          var ldepth = this.left == null ? 0 : this.left.depth;
          var rdepth = this.right == null ? 0 : this.right.depth;
          if (ldepth > rdepth + 1) {
            var lldepth = this.left.left == null ? 0 : this.left.left.depth;
            var lrdepth = this.left.right == null ? 0 : this.left.right.depth;
            if (lldepth < lrdepth) {
              this.left.rotateRR();
            }
            this.rotateLL();
          } else if (ldepth + 1 < rdepth) {
            var rrdepth = this.right.right == null ? 0 : this.right.right.depth;
            var rldepth = this.right.left == null ? 0 : this.right.left.depth;
            if (rldepth > rrdepth) {
              this.right.rotateLL();
            }
            this.rotateRR();
          }
        };
        AVLTree.prototype.rotateLL = function() {
          var nodeBefore = this.node;
          var rightBefore = this.right;
          this.node = this.left.node;
          this.right = this.left;
          this.left = this.left.left;
          this.right.left = this.right.right;
          this.right.right = rightBefore;
          this.right.node = nodeBefore;
          this.right.updateInNewLocation();
          this.updateInNewLocation();
        };
        AVLTree.prototype.rotateRR = function() {
          var nodeBefore = this.node;
          var leftBefore = this.left;
          this.node = this.right.node;
          this.left = this.right;
          this.right = this.right.right;
          this.left.right = this.left.left;
          this.left.left = leftBefore;
          this.left.node = nodeBefore;
          this.left.updateInNewLocation();
          this.updateInNewLocation();
        };
        AVLTree.prototype.updateInNewLocation = function() {
          this.getDepthFromChildren();
        };
        AVLTree.prototype.getDepthFromChildren = function() {
          this.depth = this.node == null ? 0 : 1;
          if (this.left != null) {
            this.depth = this.left.depth + 1;
          }
          if (this.right != null && this.depth <= this.right.depth) {
            this.depth = this.right.depth + 1;
          }
        };
        function nodeOrder(n1, n2) {
          if (n1 === n2) {
            return 0;
          }
          if (n1.compareDocumentPosition) {
            var cpos = n1.compareDocumentPosition(n2);
            if (cpos & 1) {
              return 1;
            }
            if (cpos & 10) {
              return 1;
            }
            if (cpos & 20) {
              return -1;
            }
            return 0;
          }
          var d1 = 0, d2 = 0;
          for (var m1 = n1; m1 != null; m1 = m1.parentNode || m1.ownerElement) {
            d1++;
          }
          for (var m2 = n2; m2 != null; m2 = m2.parentNode || m2.ownerElement) {
            d2++;
          }
          if (d1 > d2) {
            while (d1 > d2) {
              n1 = n1.parentNode || n1.ownerElement;
              d1--;
            }
            if (n1 === n2) {
              return 1;
            }
          } else if (d2 > d1) {
            while (d2 > d1) {
              n2 = n2.parentNode || n2.ownerElement;
              d2--;
            }
            if (n1 === n2) {
              return -1;
            }
          }
          var n1Par = n1.parentNode || n1.ownerElement, n2Par = n2.parentNode || n2.ownerElement;
          while (n1Par !== n2Par) {
            n1 = n1Par;
            n2 = n2Par;
            n1Par = n1.parentNode || n1.ownerElement;
            n2Par = n2.parentNode || n2.ownerElement;
          }
          var n1isAttr = isAttributeLike(n1);
          var n2isAttr = isAttributeLike(n2);
          if (n1isAttr && !n2isAttr) {
            return -1;
          }
          if (!n1isAttr && n2isAttr) {
            return 1;
          }
          if (n1.isXPathNamespace) {
            if (n1.nodeValue === XPath.XML_NAMESPACE_URI) {
              return -1;
            }
            if (!n2.isXPathNamespace) {
              return -1;
            }
            if (n2.nodeValue === XPath.XML_NAMESPACE_URI) {
              return 1;
            }
          } else if (n2.isXPathNamespace) {
            return 1;
          }
          if (n1Par) {
            var cn = n1isAttr ? n1Par.attributes : n1Par.childNodes;
            var len = cn.length;
            var n1Compare = n1.baseNode || n1;
            var n2Compare = n2.baseNode || n2;
            for (var i2 = 0; i2 < len; i2 += 1) {
              var n3 = cn[i2];
              if (n3 === n1Compare) {
                return -1;
              }
              if (n3 === n2Compare) {
                return 1;
              }
            }
          }
          throw new Error("Unexpected: could not determine node order");
        }
        AVLTree.prototype.add = function(n2) {
          if (n2 === this.node) {
            return false;
          }
          var o2 = nodeOrder(n2, this.node);
          var ret = false;
          if (o2 == -1) {
            if (this.left == null) {
              this.left = new AVLTree(n2);
              ret = true;
            } else {
              ret = this.left.add(n2);
              if (ret) {
                this.balance();
              }
            }
          } else if (o2 == 1) {
            if (this.right == null) {
              this.right = new AVLTree(n2);
              ret = true;
            } else {
              ret = this.right.add(n2);
              if (ret) {
                this.balance();
              }
            }
          }
          if (ret) {
            this.getDepthFromChildren();
          }
          return ret;
        };
        XNodeSet.prototype = new Expression();
        XNodeSet.prototype.constructor = XNodeSet;
        XNodeSet.superclass = Expression.prototype;
        function XNodeSet() {
          this.init();
        }
        XNodeSet.prototype.init = function() {
          this.tree = null;
          this.nodes = [];
          this.size = 0;
        };
        XNodeSet.prototype.toString = function() {
          var p2 = this.first();
          if (p2 == null) {
            return "";
          }
          return this.stringForNode(p2);
        };
        XNodeSet.prototype.evaluate = function(c2) {
          return this;
        };
        XNodeSet.prototype.string = function() {
          return new XString(this.toString());
        };
        XNodeSet.prototype.stringValue = function() {
          return this.toString();
        };
        XNodeSet.prototype.number = function() {
          return new XNumber(this.string());
        };
        XNodeSet.prototype.numberValue = function() {
          return Number(this.string());
        };
        XNodeSet.prototype.bool = function() {
          return new XBoolean(this.booleanValue());
        };
        XNodeSet.prototype.booleanValue = function() {
          return !!this.size;
        };
        XNodeSet.prototype.nodeset = function() {
          return this;
        };
        XNodeSet.prototype.stringForNode = function(n2) {
          if (n2.nodeType == NodeTypes.DOCUMENT_NODE || n2.nodeType == NodeTypes.ELEMENT_NODE || n2.nodeType === NodeTypes.DOCUMENT_FRAGMENT_NODE) {
            return this.stringForContainerNode(n2);
          }
          if (n2.nodeType === NodeTypes.ATTRIBUTE_NODE) {
            return n2.value || n2.nodeValue;
          }
          if (n2.isNamespaceNode) {
            return n2.namespace;
          }
          return n2.nodeValue;
        };
        XNodeSet.prototype.stringForContainerNode = function(n2) {
          var s2 = "";
          for (var n22 = n2.firstChild; n22 != null; n22 = n22.nextSibling) {
            var nt2 = n22.nodeType;
            if (nt2 === 1 || nt2 === 3 || nt2 === 4 || nt2 === 9 || nt2 === 11) {
              s2 += this.stringForNode(n22);
            }
          }
          return s2;
        };
        XNodeSet.prototype.buildTree = function() {
          if (!this.tree && this.nodes.length) {
            this.tree = new AVLTree(this.nodes[0]);
            for (var i2 = 1; i2 < this.nodes.length; i2 += 1) {
              this.tree.add(this.nodes[i2]);
            }
          }
          return this.tree;
        };
        XNodeSet.prototype.first = function() {
          var p2 = this.buildTree();
          if (p2 == null) {
            return null;
          }
          while (p2.left != null) {
            p2 = p2.left;
          }
          return p2.node;
        };
        XNodeSet.prototype.add = function(n2) {
          for (var i2 = 0; i2 < this.nodes.length; i2 += 1) {
            if (n2 === this.nodes[i2]) {
              return;
            }
          }
          this.tree = null;
          this.nodes.push(n2);
          this.size += 1;
        };
        XNodeSet.prototype.addArray = function(ns) {
          var self2 = this;
          forEach(function(x2) {
            self2.add(x2);
          }, ns);
        };
        XNodeSet.prototype.toArray = function() {
          var a2 = [];
          this.toArrayRec(this.buildTree(), a2);
          return a2;
        };
        XNodeSet.prototype.toArrayRec = function(t2, a2) {
          if (t2 != null) {
            this.toArrayRec(t2.left, a2);
            a2.push(t2.node);
            this.toArrayRec(t2.right, a2);
          }
        };
        XNodeSet.prototype.toUnsortedArray = function() {
          return this.nodes.slice();
        };
        XNodeSet.prototype.compareWithString = function(r2, o2) {
          var a2 = this.toUnsortedArray();
          for (var i2 = 0; i2 < a2.length; i2++) {
            var n2 = a2[i2];
            var l2 = new XString(this.stringForNode(n2));
            var res = o2(l2, r2);
            if (res.booleanValue()) {
              return res;
            }
          }
          return new XBoolean(false);
        };
        XNodeSet.prototype.compareWithNumber = function(r2, o2) {
          var a2 = this.toUnsortedArray();
          for (var i2 = 0; i2 < a2.length; i2++) {
            var n2 = a2[i2];
            var l2 = new XNumber(this.stringForNode(n2));
            var res = o2(l2, r2);
            if (res.booleanValue()) {
              return res;
            }
          }
          return new XBoolean(false);
        };
        XNodeSet.prototype.compareWithBoolean = function(r2, o2) {
          return o2(this.bool(), r2);
        };
        XNodeSet.prototype.compareWithNodeSet = function(r2, o2) {
          var arr = this.toUnsortedArray();
          var oInvert = function(lop, rop) {
            return o2(rop, lop);
          };
          for (var i2 = 0; i2 < arr.length; i2++) {
            var l2 = new XString(this.stringForNode(arr[i2]));
            var res = r2.compareWithString(l2, oInvert);
            if (res.booleanValue()) {
              return res;
            }
          }
          return new XBoolean(false);
        };
        XNodeSet.compareWith = curry(function(o2, r2) {
          if (Utilities.instance_of(r2, XString)) {
            return this.compareWithString(r2, o2);
          }
          if (Utilities.instance_of(r2, XNumber)) {
            return this.compareWithNumber(r2, o2);
          }
          if (Utilities.instance_of(r2, XBoolean)) {
            return this.compareWithBoolean(r2, o2);
          }
          return this.compareWithNodeSet(r2, o2);
        });
        XNodeSet.prototype.equals = XNodeSet.compareWith(Operators.equals);
        XNodeSet.prototype.notequal = XNodeSet.compareWith(Operators.notequal);
        XNodeSet.prototype.lessthan = XNodeSet.compareWith(Operators.lessthan);
        XNodeSet.prototype.greaterthan = XNodeSet.compareWith(Operators.greaterthan);
        XNodeSet.prototype.lessthanorequal = XNodeSet.compareWith(Operators.lessthanorequal);
        XNodeSet.prototype.greaterthanorequal = XNodeSet.compareWith(Operators.greaterthanorequal);
        XNodeSet.prototype.union = function(r2) {
          var ns = new XNodeSet();
          ns.addArray(this.toUnsortedArray());
          ns.addArray(r2.toUnsortedArray());
          return ns;
        };
        XPathNamespace.prototype = new Object();
        XPathNamespace.prototype.constructor = XPathNamespace;
        XPathNamespace.superclass = Object.prototype;
        function XPathNamespace(pre, node, uri, p2) {
          this.isXPathNamespace = true;
          this.baseNode = node;
          this.ownerDocument = p2.ownerDocument;
          this.nodeName = pre;
          this.prefix = pre;
          this.localName = pre;
          this.namespaceURI = null;
          this.nodeValue = uri;
          this.ownerElement = p2;
          this.nodeType = NodeTypes.NAMESPACE_NODE;
        }
        XPathNamespace.prototype.toString = function() {
          return '{ "' + this.prefix + '", "' + this.namespaceURI + '" }';
        };
        XPathContext.prototype = new Object();
        XPathContext.prototype.constructor = XPathContext;
        XPathContext.superclass = Object.prototype;
        function XPathContext(vr2, nr2, fr2) {
          this.variableResolver = vr2 != null ? vr2 : new VariableResolver();
          this.namespaceResolver = nr2 != null ? nr2 : new NamespaceResolver();
          this.functionResolver = fr2 != null ? fr2 : new FunctionResolver();
        }
        XPathContext.prototype.extend = function(newProps) {
          return assign(new XPathContext(), this, newProps);
        };
        VariableResolver.prototype = new Object();
        VariableResolver.prototype.constructor = VariableResolver;
        VariableResolver.superclass = Object.prototype;
        function VariableResolver() {
        }
        VariableResolver.prototype.getVariable = function(ln, ns) {
          return null;
        };
        FunctionResolver.prototype = new Object();
        FunctionResolver.prototype.constructor = FunctionResolver;
        FunctionResolver.superclass = Object.prototype;
        function FunctionResolver(thisArg) {
          this.thisArg = thisArg != null ? thisArg : Functions;
          this.functions = new Object();
          this.addStandardFunctions();
        }
        FunctionResolver.prototype.addStandardFunctions = function() {
          this.functions["{}last"] = Functions.last;
          this.functions["{}position"] = Functions.position;
          this.functions["{}count"] = Functions.count;
          this.functions["{}id"] = Functions.id;
          this.functions["{}local-name"] = Functions.localName;
          this.functions["{}namespace-uri"] = Functions.namespaceURI;
          this.functions["{}name"] = Functions.name;
          this.functions["{}string"] = Functions.string;
          this.functions["{}concat"] = Functions.concat;
          this.functions["{}starts-with"] = Functions.startsWith;
          this.functions["{}contains"] = Functions.contains;
          this.functions["{}substring-before"] = Functions.substringBefore;
          this.functions["{}substring-after"] = Functions.substringAfter;
          this.functions["{}substring"] = Functions.substring;
          this.functions["{}string-length"] = Functions.stringLength;
          this.functions["{}normalize-space"] = Functions.normalizeSpace;
          this.functions["{}translate"] = Functions.translate;
          this.functions["{}boolean"] = Functions.boolean_;
          this.functions["{}not"] = Functions.not;
          this.functions["{}true"] = Functions.true_;
          this.functions["{}false"] = Functions.false_;
          this.functions["{}lang"] = Functions.lang;
          this.functions["{}number"] = Functions.number;
          this.functions["{}sum"] = Functions.sum;
          this.functions["{}floor"] = Functions.floor;
          this.functions["{}ceiling"] = Functions.ceiling;
          this.functions["{}round"] = Functions.round;
        };
        FunctionResolver.prototype.addFunction = function(ns, ln, f2) {
          this.functions["{" + ns + "}" + ln] = f2;
        };
        FunctionResolver.getFunctionFromContext = function(qName, context) {
          var parts = Utilities.resolveQName(qName, context.namespaceResolver, context.contextNode, false);
          if (parts[0] === null) {
            throw new Error("Cannot resolve QName " + name);
          }
          return context.functionResolver.getFunction(parts[1], parts[0]);
        };
        FunctionResolver.prototype.getFunction = function(localName, namespace) {
          return this.functions["{" + namespace + "}" + localName];
        };
        NamespaceResolver.prototype = new Object();
        NamespaceResolver.prototype.constructor = NamespaceResolver;
        NamespaceResolver.superclass = Object.prototype;
        function NamespaceResolver() {
        }
        NamespaceResolver.prototype.getNamespace = function(prefix, n2) {
          if (prefix == "xml") {
            return XPath.XML_NAMESPACE_URI;
          } else if (prefix == "xmlns") {
            return XPath.XMLNS_NAMESPACE_URI;
          }
          if (n2.nodeType == NodeTypes.DOCUMENT_NODE) {
            n2 = n2.documentElement;
          } else if (n2.nodeType == NodeTypes.ATTRIBUTE_NODE) {
            n2 = PathExpr.getOwnerElement(n2);
          } else if (n2.nodeType != NodeTypes.ELEMENT_NODE) {
            n2 = n2.parentNode;
          }
          while (n2 != null && n2.nodeType == NodeTypes.ELEMENT_NODE) {
            var nnm = n2.attributes;
            for (var i2 = 0; i2 < nnm.length; i2++) {
              var a2 = nnm.item(i2);
              var aname = a2.name || a2.nodeName;
              if (aname === "xmlns" && prefix === "" || aname === "xmlns:" + prefix) {
                return String(a2.value || a2.nodeValue);
              }
            }
            n2 = n2.parentNode;
          }
          return null;
        };
        var Functions = new Object();
        Functions.last = function(c2) {
          if (arguments.length != 1) {
            throw new Error("Function last expects ()");
          }
          return new XNumber(c2.contextSize);
        };
        Functions.position = function(c2) {
          if (arguments.length != 1) {
            throw new Error("Function position expects ()");
          }
          return new XNumber(c2.contextPosition);
        };
        Functions.count = function() {
          var c2 = arguments[0];
          var ns;
          if (arguments.length != 2 || !Utilities.instance_of(ns = arguments[1].evaluate(c2), XNodeSet)) {
            throw new Error("Function count expects (node-set)");
          }
          return new XNumber(ns.size);
        };
        Functions.id = function() {
          var c2 = arguments[0];
          var id;
          if (arguments.length != 2) {
            throw new Error("Function id expects (object)");
          }
          id = arguments[1].evaluate(c2);
          if (Utilities.instance_of(id, XNodeSet)) {
            id = id.toArray().join(" ");
          } else {
            id = id.stringValue();
          }
          var ids = id.split(/[\x0d\x0a\x09\x20]+/);
          var count = 0;
          var ns = new XNodeSet();
          var doc = c2.contextNode.nodeType == NodeTypes.DOCUMENT_NODE ? c2.contextNode : c2.contextNode.ownerDocument;
          for (var i2 = 0; i2 < ids.length; i2++) {
            var n2;
            if (doc.getElementById) {
              n2 = doc.getElementById(ids[i2]);
            } else {
              n2 = Utilities.getElementById(doc, ids[i2]);
            }
            if (n2 != null) {
              ns.add(n2);
              count++;
            }
          }
          return ns;
        };
        Functions.localName = function(c2, eNode) {
          var n2;
          if (arguments.length == 1) {
            n2 = c2.contextNode;
          } else if (arguments.length == 2) {
            n2 = eNode.evaluate(c2).first();
          } else {
            throw new Error("Function local-name expects (node-set?)");
          }
          if (n2 == null) {
            return new XString("");
          }
          return new XString(
            n2.localName || //  standard elements and attributes
            n2.baseName || //  IE
            n2.target || //  processing instructions
            n2.nodeName || //  DOM1 elements
            ""
            //  fallback
          );
        };
        Functions.namespaceURI = function() {
          var c2 = arguments[0];
          var n2;
          if (arguments.length == 1) {
            n2 = c2.contextNode;
          } else if (arguments.length == 2) {
            n2 = arguments[1].evaluate(c2).first();
          } else {
            throw new Error("Function namespace-uri expects (node-set?)");
          }
          if (n2 == null) {
            return new XString("");
          }
          return new XString(n2.namespaceURI || "");
        };
        Functions.name = function() {
          var c2 = arguments[0];
          var n2;
          if (arguments.length == 1) {
            n2 = c2.contextNode;
          } else if (arguments.length == 2) {
            n2 = arguments[1].evaluate(c2).first();
          } else {
            throw new Error("Function name expects (node-set?)");
          }
          if (n2 == null) {
            return new XString("");
          }
          if (n2.nodeType == NodeTypes.ELEMENT_NODE) {
            return new XString(n2.nodeName);
          } else if (n2.nodeType == NodeTypes.ATTRIBUTE_NODE) {
            return new XString(n2.name || n2.nodeName);
          } else if (n2.nodeType === NodeTypes.PROCESSING_INSTRUCTION_NODE) {
            return new XString(n2.target || n2.nodeName);
          } else if (n2.localName == null) {
            return new XString("");
          } else {
            return new XString(n2.localName);
          }
        };
        Functions.string = function() {
          var c2 = arguments[0];
          if (arguments.length == 1) {
            return new XString(XNodeSet.prototype.stringForNode(c2.contextNode));
          } else if (arguments.length == 2) {
            return arguments[1].evaluate(c2).string();
          }
          throw new Error("Function string expects (object?)");
        };
        Functions.concat = function(c2) {
          if (arguments.length < 3) {
            throw new Error("Function concat expects (string, string[, string]*)");
          }
          var s2 = "";
          for (var i2 = 1; i2 < arguments.length; i2++) {
            s2 += arguments[i2].evaluate(c2).stringValue();
          }
          return new XString(s2);
        };
        Functions.startsWith = function() {
          var c2 = arguments[0];
          if (arguments.length != 3) {
            throw new Error("Function startsWith expects (string, string)");
          }
          var s1 = arguments[1].evaluate(c2).stringValue();
          var s2 = arguments[2].evaluate(c2).stringValue();
          return new XBoolean(s1.substring(0, s2.length) == s2);
        };
        Functions.contains = function() {
          var c2 = arguments[0];
          if (arguments.length != 3) {
            throw new Error("Function contains expects (string, string)");
          }
          var s1 = arguments[1].evaluate(c2).stringValue();
          var s2 = arguments[2].evaluate(c2).stringValue();
          return new XBoolean(s1.indexOf(s2) !== -1);
        };
        Functions.substringBefore = function() {
          var c2 = arguments[0];
          if (arguments.length != 3) {
            throw new Error("Function substring-before expects (string, string)");
          }
          var s1 = arguments[1].evaluate(c2).stringValue();
          var s2 = arguments[2].evaluate(c2).stringValue();
          return new XString(s1.substring(0, s1.indexOf(s2)));
        };
        Functions.substringAfter = function() {
          var c2 = arguments[0];
          if (arguments.length != 3) {
            throw new Error("Function substring-after expects (string, string)");
          }
          var s1 = arguments[1].evaluate(c2).stringValue();
          var s2 = arguments[2].evaluate(c2).stringValue();
          if (s2.length == 0) {
            return new XString(s1);
          }
          var i2 = s1.indexOf(s2);
          if (i2 == -1) {
            return new XString("");
          }
          return new XString(s1.substring(i2 + s2.length));
        };
        Functions.substring = function() {
          var c2 = arguments[0];
          if (!(arguments.length == 3 || arguments.length == 4)) {
            throw new Error("Function substring expects (string, number, number?)");
          }
          var s2 = arguments[1].evaluate(c2).stringValue();
          var n1 = Math.round(arguments[2].evaluate(c2).numberValue()) - 1;
          var n2 = arguments.length == 4 ? n1 + Math.round(arguments[3].evaluate(c2).numberValue()) : void 0;
          return new XString(s2.substring(n1, n2));
        };
        Functions.stringLength = function() {
          var c2 = arguments[0];
          var s2;
          if (arguments.length == 1) {
            s2 = XNodeSet.prototype.stringForNode(c2.contextNode);
          } else if (arguments.length == 2) {
            s2 = arguments[1].evaluate(c2).stringValue();
          } else {
            throw new Error("Function string-length expects (string?)");
          }
          return new XNumber(s2.length);
        };
        Functions.normalizeSpace = function() {
          var c2 = arguments[0];
          var s2;
          if (arguments.length == 1) {
            s2 = XNodeSet.prototype.stringForNode(c2.contextNode);
          } else if (arguments.length == 2) {
            s2 = arguments[1].evaluate(c2).stringValue();
          } else {
            throw new Error("Function normalize-space expects (string?)");
          }
          var i2 = 0;
          var j2 = s2.length - 1;
          while (Utilities.isSpace(s2.charCodeAt(j2))) {
            j2--;
          }
          var t2 = "";
          while (i2 <= j2 && Utilities.isSpace(s2.charCodeAt(i2))) {
            i2++;
          }
          while (i2 <= j2) {
            if (Utilities.isSpace(s2.charCodeAt(i2))) {
              t2 += " ";
              while (i2 <= j2 && Utilities.isSpace(s2.charCodeAt(i2))) {
                i2++;
              }
            } else {
              t2 += s2.charAt(i2);
              i2++;
            }
          }
          return new XString(t2);
        };
        Functions.translate = function(c2, eValue, eFrom, eTo) {
          if (arguments.length != 4) {
            throw new Error("Function translate expects (string, string, string)");
          }
          var value = eValue.evaluate(c2).stringValue();
          var from = eFrom.evaluate(c2).stringValue();
          var to2 = eTo.evaluate(c2).stringValue();
          var cMap = reduce(function(acc, ch, i2) {
            if (!(ch in acc)) {
              acc[ch] = i2 > to2.length ? "" : to2[i2];
            }
            return acc;
          }, {}, from);
          var t2 = join(
            "",
            map(function(ch) {
              return ch in cMap ? cMap[ch] : ch;
            }, value)
          );
          return new XString(t2);
        };
        Functions.boolean_ = function() {
          var c2 = arguments[0];
          if (arguments.length != 2) {
            throw new Error("Function boolean expects (object)");
          }
          return arguments[1].evaluate(c2).bool();
        };
        Functions.not = function(c2, eValue) {
          if (arguments.length != 2) {
            throw new Error("Function not expects (object)");
          }
          return eValue.evaluate(c2).bool().not();
        };
        Functions.true_ = function() {
          if (arguments.length != 1) {
            throw new Error("Function true expects ()");
          }
          return XBoolean.true_;
        };
        Functions.false_ = function() {
          if (arguments.length != 1) {
            throw new Error("Function false expects ()");
          }
          return XBoolean.false_;
        };
        Functions.lang = function() {
          var c2 = arguments[0];
          if (arguments.length != 2) {
            throw new Error("Function lang expects (string)");
          }
          var lang;
          for (var n2 = c2.contextNode; n2 != null && n2.nodeType != NodeTypes.DOCUMENT_NODE; n2 = n2.parentNode) {
            var a2 = n2.getAttributeNS(XPath.XML_NAMESPACE_URI, "lang");
            if (a2 != null) {
              lang = String(a2);
              break;
            }
          }
          if (lang == null) {
            return XBoolean.false_;
          }
          var s2 = arguments[1].evaluate(c2).stringValue();
          return new XBoolean(lang.substring(0, s2.length) == s2 && (lang.length == s2.length || lang.charAt(s2.length) == "-"));
        };
        Functions.number = function() {
          var c2 = arguments[0];
          if (!(arguments.length == 1 || arguments.length == 2)) {
            throw new Error("Function number expects (object?)");
          }
          if (arguments.length == 1) {
            return new XNumber(XNodeSet.prototype.stringForNode(c2.contextNode));
          }
          return arguments[1].evaluate(c2).number();
        };
        Functions.sum = function() {
          var c2 = arguments[0];
          var ns;
          if (arguments.length != 2 || !Utilities.instance_of(ns = arguments[1].evaluate(c2), XNodeSet)) {
            throw new Error("Function sum expects (node-set)");
          }
          ns = ns.toUnsortedArray();
          var n2 = 0;
          for (var i2 = 0; i2 < ns.length; i2++) {
            n2 += new XNumber(XNodeSet.prototype.stringForNode(ns[i2])).numberValue();
          }
          return new XNumber(n2);
        };
        Functions.floor = function() {
          var c2 = arguments[0];
          if (arguments.length != 2) {
            throw new Error("Function floor expects (number)");
          }
          return new XNumber(Math.floor(arguments[1].evaluate(c2).numberValue()));
        };
        Functions.ceiling = function() {
          var c2 = arguments[0];
          if (arguments.length != 2) {
            throw new Error("Function ceiling expects (number)");
          }
          return new XNumber(Math.ceil(arguments[1].evaluate(c2).numberValue()));
        };
        Functions.round = function() {
          var c2 = arguments[0];
          if (arguments.length != 2) {
            throw new Error("Function round expects (number)");
          }
          return new XNumber(Math.round(arguments[1].evaluate(c2).numberValue()));
        };
        var Utilities = new Object();
        var isAttributeLike = function(val) {
          return val && (val.nodeType === NodeTypes.ATTRIBUTE_NODE || val.ownerElement || val.isXPathNamespace);
        };
        Utilities.splitQName = function(qn) {
          var i2 = qn.indexOf(":");
          if (i2 == -1) {
            return [null, qn];
          }
          return [qn.substring(0, i2), qn.substring(i2 + 1)];
        };
        Utilities.resolveQName = function(qn, nr2, n2, useDefault) {
          var parts = Utilities.splitQName(qn);
          if (parts[0] != null) {
            parts[0] = nr2.getNamespace(parts[0], n2);
          } else {
            if (useDefault) {
              parts[0] = nr2.getNamespace("", n2);
              if (parts[0] == null) {
                parts[0] = "";
              }
            } else {
              parts[0] = "";
            }
          }
          return parts;
        };
        Utilities.isSpace = function(c2) {
          return c2 == 9 || c2 == 13 || c2 == 10 || c2 == 32;
        };
        Utilities.isLetter = function(c2) {
          return c2 >= 65 && c2 <= 90 || c2 >= 97 && c2 <= 122 || c2 >= 192 && c2 <= 214 || c2 >= 216 && c2 <= 246 || c2 >= 248 && c2 <= 255 || c2 >= 256 && c2 <= 305 || c2 >= 308 && c2 <= 318 || c2 >= 321 && c2 <= 328 || c2 >= 330 && c2 <= 382 || c2 >= 384 && c2 <= 451 || c2 >= 461 && c2 <= 496 || c2 >= 500 && c2 <= 501 || c2 >= 506 && c2 <= 535 || c2 >= 592 && c2 <= 680 || c2 >= 699 && c2 <= 705 || c2 == 902 || c2 >= 904 && c2 <= 906 || c2 == 908 || c2 >= 910 && c2 <= 929 || c2 >= 931 && c2 <= 974 || c2 >= 976 && c2 <= 982 || c2 == 986 || c2 == 988 || c2 == 990 || c2 == 992 || c2 >= 994 && c2 <= 1011 || c2 >= 1025 && c2 <= 1036 || c2 >= 1038 && c2 <= 1103 || c2 >= 1105 && c2 <= 1116 || c2 >= 1118 && c2 <= 1153 || c2 >= 1168 && c2 <= 1220 || c2 >= 1223 && c2 <= 1224 || c2 >= 1227 && c2 <= 1228 || c2 >= 1232 && c2 <= 1259 || c2 >= 1262 && c2 <= 1269 || c2 >= 1272 && c2 <= 1273 || c2 >= 1329 && c2 <= 1366 || c2 == 1369 || c2 >= 1377 && c2 <= 1414 || c2 >= 1488 && c2 <= 1514 || c2 >= 1520 && c2 <= 1522 || c2 >= 1569 && c2 <= 1594 || c2 >= 1601 && c2 <= 1610 || c2 >= 1649 && c2 <= 1719 || c2 >= 1722 && c2 <= 1726 || c2 >= 1728 && c2 <= 1742 || c2 >= 1744 && c2 <= 1747 || c2 == 1749 || c2 >= 1765 && c2 <= 1766 || c2 >= 2309 && c2 <= 2361 || c2 == 2365 || c2 >= 2392 && c2 <= 2401 || c2 >= 2437 && c2 <= 2444 || c2 >= 2447 && c2 <= 2448 || c2 >= 2451 && c2 <= 2472 || c2 >= 2474 && c2 <= 2480 || c2 == 2482 || c2 >= 2486 && c2 <= 2489 || c2 >= 2524 && c2 <= 2525 || c2 >= 2527 && c2 <= 2529 || c2 >= 2544 && c2 <= 2545 || c2 >= 2565 && c2 <= 2570 || c2 >= 2575 && c2 <= 2576 || c2 >= 2579 && c2 <= 2600 || c2 >= 2602 && c2 <= 2608 || c2 >= 2610 && c2 <= 2611 || c2 >= 2613 && c2 <= 2614 || c2 >= 2616 && c2 <= 2617 || c2 >= 2649 && c2 <= 2652 || c2 == 2654 || c2 >= 2674 && c2 <= 2676 || c2 >= 2693 && c2 <= 2699 || c2 == 2701 || c2 >= 2703 && c2 <= 2705 || c2 >= 2707 && c2 <= 2728 || c2 >= 2730 && c2 <= 2736 || c2 >= 2738 && c2 <= 2739 || c2 >= 2741 && c2 <= 2745 || c2 == 2749 || c2 == 2784 || c2 >= 2821 && c2 <= 2828 || c2 >= 2831 && c2 <= 2832 || c2 >= 2835 && c2 <= 2856 || c2 >= 2858 && c2 <= 2864 || c2 >= 2866 && c2 <= 2867 || c2 >= 2870 && c2 <= 2873 || c2 == 2877 || c2 >= 2908 && c2 <= 2909 || c2 >= 2911 && c2 <= 2913 || c2 >= 2949 && c2 <= 2954 || c2 >= 2958 && c2 <= 2960 || c2 >= 2962 && c2 <= 2965 || c2 >= 2969 && c2 <= 2970 || c2 == 2972 || c2 >= 2974 && c2 <= 2975 || c2 >= 2979 && c2 <= 2980 || c2 >= 2984 && c2 <= 2986 || c2 >= 2990 && c2 <= 2997 || c2 >= 2999 && c2 <= 3001 || c2 >= 3077 && c2 <= 3084 || c2 >= 3086 && c2 <= 3088 || c2 >= 3090 && c2 <= 3112 || c2 >= 3114 && c2 <= 3123 || c2 >= 3125 && c2 <= 3129 || c2 >= 3168 && c2 <= 3169 || c2 >= 3205 && c2 <= 3212 || c2 >= 3214 && c2 <= 3216 || c2 >= 3218 && c2 <= 3240 || c2 >= 3242 && c2 <= 3251 || c2 >= 3253 && c2 <= 3257 || c2 == 3294 || c2 >= 3296 && c2 <= 3297 || c2 >= 3333 && c2 <= 3340 || c2 >= 3342 && c2 <= 3344 || c2 >= 3346 && c2 <= 3368 || c2 >= 3370 && c2 <= 3385 || c2 >= 3424 && c2 <= 3425 || c2 >= 3585 && c2 <= 3630 || c2 == 3632 || c2 >= 3634 && c2 <= 3635 || c2 >= 3648 && c2 <= 3653 || c2 >= 3713 && c2 <= 3714 || c2 == 3716 || c2 >= 3719 && c2 <= 3720 || c2 == 3722 || c2 == 3725 || c2 >= 3732 && c2 <= 3735 || c2 >= 3737 && c2 <= 3743 || c2 >= 3745 && c2 <= 3747 || c2 == 3749 || c2 == 3751 || c2 >= 3754 && c2 <= 3755 || c2 >= 3757 && c2 <= 3758 || c2 == 3760 || c2 >= 3762 && c2 <= 3763 || c2 == 3773 || c2 >= 3776 && c2 <= 3780 || c2 >= 3904 && c2 <= 3911 || c2 >= 3913 && c2 <= 3945 || c2 >= 4256 && c2 <= 4293 || c2 >= 4304 && c2 <= 4342 || c2 == 4352 || c2 >= 4354 && c2 <= 4355 || c2 >= 4357 && c2 <= 4359 || c2 == 4361 || c2 >= 4363 && c2 <= 4364 || c2 >= 4366 && c2 <= 4370 || c2 == 4412 || c2 == 4414 || c2 == 4416 || c2 == 4428 || c2 == 4430 || c2 == 4432 || c2 >= 4436 && c2 <= 4437 || c2 == 4441 || c2 >= 4447 && c2 <= 4449 || c2 == 4451 || c2 == 4453 || c2 == 4455 || c2 == 4457 || c2 >= 4461 && c2 <= 4462 || c2 >= 4466 && c2 <= 4467 || c2 == 4469 || c2 == 4510 || c2 == 4520 || c2 == 4523 || c2 >= 4526 && c2 <= 4527 || c2 >= 4535 && c2 <= 4536 || c2 == 4538 || c2 >= 4540 && c2 <= 4546 || c2 == 4587 || c2 == 4592 || c2 == 4601 || c2 >= 7680 && c2 <= 7835 || c2 >= 7840 && c2 <= 7929 || c2 >= 7936 && c2 <= 7957 || c2 >= 7960 && c2 <= 7965 || c2 >= 7968 && c2 <= 8005 || c2 >= 8008 && c2 <= 8013 || c2 >= 8016 && c2 <= 8023 || c2 == 8025 || c2 == 8027 || c2 == 8029 || c2 >= 8031 && c2 <= 8061 || c2 >= 8064 && c2 <= 8116 || c2 >= 8118 && c2 <= 8124 || c2 == 8126 || c2 >= 8130 && c2 <= 8132 || c2 >= 8134 && c2 <= 8140 || c2 >= 8144 && c2 <= 8147 || c2 >= 8150 && c2 <= 8155 || c2 >= 8160 && c2 <= 8172 || c2 >= 8178 && c2 <= 8180 || c2 >= 8182 && c2 <= 8188 || c2 == 8486 || c2 >= 8490 && c2 <= 8491 || c2 == 8494 || c2 >= 8576 && c2 <= 8578 || c2 >= 12353 && c2 <= 12436 || c2 >= 12449 && c2 <= 12538 || c2 >= 12549 && c2 <= 12588 || c2 >= 44032 && c2 <= 55203 || c2 >= 19968 && c2 <= 40869 || c2 == 12295 || c2 >= 12321 && c2 <= 12329;
        };
        Utilities.isNCNameChar = function(c2) {
          return c2 >= 48 && c2 <= 57 || c2 >= 1632 && c2 <= 1641 || c2 >= 1776 && c2 <= 1785 || c2 >= 2406 && c2 <= 2415 || c2 >= 2534 && c2 <= 2543 || c2 >= 2662 && c2 <= 2671 || c2 >= 2790 && c2 <= 2799 || c2 >= 2918 && c2 <= 2927 || c2 >= 3047 && c2 <= 3055 || c2 >= 3174 && c2 <= 3183 || c2 >= 3302 && c2 <= 3311 || c2 >= 3430 && c2 <= 3439 || c2 >= 3664 && c2 <= 3673 || c2 >= 3792 && c2 <= 3801 || c2 >= 3872 && c2 <= 3881 || c2 == 46 || c2 == 45 || c2 == 95 || Utilities.isLetter(c2) || c2 >= 768 && c2 <= 837 || c2 >= 864 && c2 <= 865 || c2 >= 1155 && c2 <= 1158 || c2 >= 1425 && c2 <= 1441 || c2 >= 1443 && c2 <= 1465 || c2 >= 1467 && c2 <= 1469 || c2 == 1471 || c2 >= 1473 && c2 <= 1474 || c2 == 1476 || c2 >= 1611 && c2 <= 1618 || c2 == 1648 || c2 >= 1750 && c2 <= 1756 || c2 >= 1757 && c2 <= 1759 || c2 >= 1760 && c2 <= 1764 || c2 >= 1767 && c2 <= 1768 || c2 >= 1770 && c2 <= 1773 || c2 >= 2305 && c2 <= 2307 || c2 == 2364 || c2 >= 2366 && c2 <= 2380 || c2 == 2381 || c2 >= 2385 && c2 <= 2388 || c2 >= 2402 && c2 <= 2403 || c2 >= 2433 && c2 <= 2435 || c2 == 2492 || c2 == 2494 || c2 == 2495 || c2 >= 2496 && c2 <= 2500 || c2 >= 2503 && c2 <= 2504 || c2 >= 2507 && c2 <= 2509 || c2 == 2519 || c2 >= 2530 && c2 <= 2531 || c2 == 2562 || c2 == 2620 || c2 == 2622 || c2 == 2623 || c2 >= 2624 && c2 <= 2626 || c2 >= 2631 && c2 <= 2632 || c2 >= 2635 && c2 <= 2637 || c2 >= 2672 && c2 <= 2673 || c2 >= 2689 && c2 <= 2691 || c2 == 2748 || c2 >= 2750 && c2 <= 2757 || c2 >= 2759 && c2 <= 2761 || c2 >= 2763 && c2 <= 2765 || c2 >= 2817 && c2 <= 2819 || c2 == 2876 || c2 >= 2878 && c2 <= 2883 || c2 >= 2887 && c2 <= 2888 || c2 >= 2891 && c2 <= 2893 || c2 >= 2902 && c2 <= 2903 || c2 >= 2946 && c2 <= 2947 || c2 >= 3006 && c2 <= 3010 || c2 >= 3014 && c2 <= 3016 || c2 >= 3018 && c2 <= 3021 || c2 == 3031 || c2 >= 3073 && c2 <= 3075 || c2 >= 3134 && c2 <= 3140 || c2 >= 3142 && c2 <= 3144 || c2 >= 3146 && c2 <= 3149 || c2 >= 3157 && c2 <= 3158 || c2 >= 3202 && c2 <= 3203 || c2 >= 3262 && c2 <= 3268 || c2 >= 3270 && c2 <= 3272 || c2 >= 3274 && c2 <= 3277 || c2 >= 3285 && c2 <= 3286 || c2 >= 3330 && c2 <= 3331 || c2 >= 3390 && c2 <= 3395 || c2 >= 3398 && c2 <= 3400 || c2 >= 3402 && c2 <= 3405 || c2 == 3415 || c2 == 3633 || c2 >= 3636 && c2 <= 3642 || c2 >= 3655 && c2 <= 3662 || c2 == 3761 || c2 >= 3764 && c2 <= 3769 || c2 >= 3771 && c2 <= 3772 || c2 >= 3784 && c2 <= 3789 || c2 >= 3864 && c2 <= 3865 || c2 == 3893 || c2 == 3895 || c2 == 3897 || c2 == 3902 || c2 == 3903 || c2 >= 3953 && c2 <= 3972 || c2 >= 3974 && c2 <= 3979 || c2 >= 3984 && c2 <= 3989 || c2 == 3991 || c2 >= 3993 && c2 <= 4013 || c2 >= 4017 && c2 <= 4023 || c2 == 4025 || c2 >= 8400 && c2 <= 8412 || c2 == 8417 || c2 >= 12330 && c2 <= 12335 || c2 == 12441 || c2 == 12442 || c2 == 183 || c2 == 720 || c2 == 721 || c2 == 903 || c2 == 1600 || c2 == 3654 || c2 == 3782 || c2 == 12293 || c2 >= 12337 && c2 <= 12341 || c2 >= 12445 && c2 <= 12446 || c2 >= 12540 && c2 <= 12542;
        };
        Utilities.coalesceText = function(n2) {
          for (var m2 = n2.firstChild; m2 != null; m2 = m2.nextSibling) {
            if (m2.nodeType == NodeTypes.TEXT_NODE || m2.nodeType == NodeTypes.CDATA_SECTION_NODE) {
              var s2 = m2.nodeValue;
              var first = m2;
              m2 = m2.nextSibling;
              while (m2 != null && (m2.nodeType == NodeTypes.TEXT_NODE || m2.nodeType == NodeTypes.CDATA_SECTION_NODE)) {
                s2 += m2.nodeValue;
                var del = m2;
                m2 = m2.nextSibling;
                del.parentNode.removeChild(del);
              }
              if (first.nodeType == NodeTypes.CDATA_SECTION_NODE) {
                var p2 = first.parentNode;
                if (first.nextSibling == null) {
                  p2.removeChild(first);
                  p2.appendChild(p2.ownerDocument.createTextNode(s2));
                } else {
                  var next = first.nextSibling;
                  p2.removeChild(first);
                  p2.insertBefore(p2.ownerDocument.createTextNode(s2), next);
                }
              } else {
                first.nodeValue = s2;
              }
              if (m2 == null) {
                break;
              }
            } else if (m2.nodeType == NodeTypes.ELEMENT_NODE) {
              Utilities.coalesceText(m2);
            }
          }
        };
        Utilities.instance_of = function(o2, c2) {
          while (o2 != null) {
            if (o2.constructor === c2) {
              return true;
            }
            if (o2 === Object) {
              return false;
            }
            o2 = o2.constructor.superclass;
          }
          return false;
        };
        Utilities.getElementById = function(n2, id) {
          if (n2.nodeType == NodeTypes.ELEMENT_NODE) {
            if (n2.getAttribute("id") == id || n2.getAttributeNS(null, "id") == id) {
              return n2;
            }
          }
          for (var m2 = n2.firstChild; m2 != null; m2 = m2.nextSibling) {
            var res = Utilities.getElementById(m2, id);
            if (res != null) {
              return res;
            }
          }
          return null;
        };
        var XPathException = (function() {
          function getMessage(code, exception) {
            var msg = exception ? ": " + exception.toString() : "";
            switch (code) {
              case XPathException2.INVALID_EXPRESSION_ERR:
                return "Invalid expression" + msg;
              case XPathException2.TYPE_ERR:
                return "Type error" + msg;
            }
            return null;
          }
          function XPathException2(code, error, message) {
            var err = Error.call(this, getMessage(code, error) || message);
            err.code = code;
            err.exception = error;
            return err;
          }
          XPathException2.prototype = Object.create(Error.prototype);
          XPathException2.prototype.constructor = XPathException2;
          XPathException2.superclass = Error;
          XPathException2.prototype.toString = function() {
            return this.message;
          };
          XPathException2.fromMessage = function(message, error) {
            return new XPathException2(null, error, message);
          };
          XPathException2.INVALID_EXPRESSION_ERR = 51;
          XPathException2.TYPE_ERR = 52;
          return XPathException2;
        })();
        XPathExpression.prototype = {};
        XPathExpression.prototype.constructor = XPathExpression;
        XPathExpression.superclass = Object.prototype;
        function XPathExpression(e2, r2, p2) {
          this.xpath = p2.parse(e2);
          this.context = new XPathContext();
          this.context.namespaceResolver = new XPathNSResolverWrapper(r2);
        }
        XPathExpression.getOwnerDocument = function(n2) {
          return n2.nodeType === NodeTypes.DOCUMENT_NODE ? n2 : n2.ownerDocument;
        };
        XPathExpression.detectHtmlDom = function(n2) {
          if (!n2) {
            return false;
          }
          var doc = XPathExpression.getOwnerDocument(n2);
          try {
            return doc.implementation.hasFeature("HTML", "2.0");
          } catch (e2) {
            return true;
          }
        };
        XPathExpression.prototype.evaluate = function(n2, t2, res) {
          this.context.expressionContextNode = n2;
          this.context.caseInsensitive = XPathExpression.detectHtmlDom(n2);
          var result = this.xpath.evaluate(this.context);
          return new XPathResult(result, t2);
        };
        XPathNSResolverWrapper.prototype = {};
        XPathNSResolverWrapper.prototype.constructor = XPathNSResolverWrapper;
        XPathNSResolverWrapper.superclass = Object.prototype;
        function XPathNSResolverWrapper(r2) {
          this.xpathNSResolver = r2;
        }
        XPathNSResolverWrapper.prototype.getNamespace = function(prefix, n2) {
          if (this.xpathNSResolver == null) {
            return null;
          }
          return this.xpathNSResolver.lookupNamespaceURI(prefix);
        };
        NodeXPathNSResolver.prototype = {};
        NodeXPathNSResolver.prototype.constructor = NodeXPathNSResolver;
        NodeXPathNSResolver.superclass = Object.prototype;
        function NodeXPathNSResolver(n2) {
          this.node = n2;
          this.namespaceResolver = new NamespaceResolver();
        }
        NodeXPathNSResolver.prototype.lookupNamespaceURI = function(prefix) {
          return this.namespaceResolver.getNamespace(prefix, this.node);
        };
        XPathResult.prototype = {};
        XPathResult.prototype.constructor = XPathResult;
        XPathResult.superclass = Object.prototype;
        function XPathResult(v2, t2) {
          if (t2 == XPathResult.ANY_TYPE) {
            if (v2.constructor === XString) {
              t2 = XPathResult.STRING_TYPE;
            } else if (v2.constructor === XNumber) {
              t2 = XPathResult.NUMBER_TYPE;
            } else if (v2.constructor === XBoolean) {
              t2 = XPathResult.BOOLEAN_TYPE;
            } else if (v2.constructor === XNodeSet) {
              t2 = XPathResult.UNORDERED_NODE_ITERATOR_TYPE;
            }
          }
          this.resultType = t2;
          switch (t2) {
            case XPathResult.NUMBER_TYPE:
              this.numberValue = v2.numberValue();
              return;
            case XPathResult.STRING_TYPE:
              this.stringValue = v2.stringValue();
              return;
            case XPathResult.BOOLEAN_TYPE:
              this.booleanValue = v2.booleanValue();
              return;
            case XPathResult.ANY_UNORDERED_NODE_TYPE:
            case XPathResult.FIRST_ORDERED_NODE_TYPE:
              if (v2.constructor === XNodeSet) {
                this.singleNodeValue = v2.first();
                return;
              }
              break;
            case XPathResult.UNORDERED_NODE_ITERATOR_TYPE:
            case XPathResult.ORDERED_NODE_ITERATOR_TYPE:
              if (v2.constructor === XNodeSet) {
                this.invalidIteratorState = false;
                this.nodes = v2.toArray();
                this.iteratorIndex = 0;
                return;
              }
              break;
            case XPathResult.UNORDERED_NODE_SNAPSHOT_TYPE:
            case XPathResult.ORDERED_NODE_SNAPSHOT_TYPE:
              if (v2.constructor === XNodeSet) {
                this.nodes = v2.toArray();
                this.snapshotLength = this.nodes.length;
                return;
              }
              break;
          }
          throw new XPathException(XPathException.TYPE_ERR);
        }
        ;
        XPathResult.prototype.iterateNext = function() {
          if (this.resultType != XPathResult.UNORDERED_NODE_ITERATOR_TYPE && this.resultType != XPathResult.ORDERED_NODE_ITERATOR_TYPE) {
            throw new XPathException(XPathException.TYPE_ERR);
          }
          return this.nodes[this.iteratorIndex++];
        };
        XPathResult.prototype.snapshotItem = function(i2) {
          if (this.resultType != XPathResult.UNORDERED_NODE_SNAPSHOT_TYPE && this.resultType != XPathResult.ORDERED_NODE_SNAPSHOT_TYPE) {
            throw new XPathException(XPathException.TYPE_ERR);
          }
          return this.nodes[i2];
        };
        XPathResult.ANY_TYPE = 0;
        XPathResult.NUMBER_TYPE = 1;
        XPathResult.STRING_TYPE = 2;
        XPathResult.BOOLEAN_TYPE = 3;
        XPathResult.UNORDERED_NODE_ITERATOR_TYPE = 4;
        XPathResult.ORDERED_NODE_ITERATOR_TYPE = 5;
        XPathResult.UNORDERED_NODE_SNAPSHOT_TYPE = 6;
        XPathResult.ORDERED_NODE_SNAPSHOT_TYPE = 7;
        XPathResult.ANY_UNORDERED_NODE_TYPE = 8;
        XPathResult.FIRST_ORDERED_NODE_TYPE = 9;
        function installDOM3XPathSupport(doc, p2) {
          doc.createExpression = function(e2, r2) {
            try {
              return new XPathExpression(e2, r2, p2);
            } catch (e3) {
              throw new XPathException(XPathException.INVALID_EXPRESSION_ERR, e3);
            }
          };
          doc.createNSResolver = function(n2) {
            return new NodeXPathNSResolver(n2);
          };
          doc.evaluate = function(e2, cn, r2, t2, res) {
            if (t2 < 0 || t2 > 9) {
              throw { code: 0, toString: function() {
                return "Request type not supported";
              } };
            }
            return doc.createExpression(e2, r2, p2).evaluate(cn, t2, res);
          };
        }
        ;
        try {
          var shouldInstall = true;
          try {
            if (document.implementation && document.implementation.hasFeature && document.implementation.hasFeature("XPath", null)) {
              shouldInstall = false;
            }
          } catch (e2) {
          }
          if (shouldInstall) {
            installDOM3XPathSupport(document, new XPathParser());
          }
        } catch (e2) {
        }
        installDOM3XPathSupport(exports2, new XPathParser());
        (function() {
          var parser = new XPathParser();
          var defaultNSResolver = new NamespaceResolver();
          var defaultFunctionResolver = new FunctionResolver();
          var defaultVariableResolver = new VariableResolver();
          function makeNSResolverFromFunction(func) {
            return {
              getNamespace: function(prefix, node) {
                var ns = func(prefix, node);
                return ns || defaultNSResolver.getNamespace(prefix, node);
              }
            };
          }
          function makeNSResolverFromObject(obj) {
            return makeNSResolverFromFunction(obj.getNamespace.bind(obj));
          }
          function makeNSResolverFromMap(map2) {
            return makeNSResolverFromFunction(function(prefix) {
              return map2[prefix];
            });
          }
          function makeNSResolver(resolver) {
            if (resolver && typeof resolver.getNamespace === "function") {
              return makeNSResolverFromObject(resolver);
            }
            if (typeof resolver === "function") {
              return makeNSResolverFromFunction(resolver);
            }
            if (typeof resolver === "object") {
              return makeNSResolverFromMap(resolver);
            }
            return defaultNSResolver;
          }
          function convertValue(value) {
            if (value === null || typeof value === "undefined" || value instanceof XString || value instanceof XBoolean || value instanceof XNumber || value instanceof XNodeSet) {
              return value;
            }
            switch (typeof value) {
              case "string":
                return new XString(value);
              case "boolean":
                return new XBoolean(value);
              case "number":
                return new XNumber(value);
            }
            var ns = new XNodeSet();
            ns.addArray([].concat(value));
            return ns;
          }
          function makeEvaluator(func) {
            return function(context) {
              var args = Array.prototype.slice.call(arguments, 1).map(function(arg) {
                return arg.evaluate(context);
              });
              var result = func.apply(this, [].concat(context, args));
              return convertValue(result);
            };
          }
          function makeFunctionResolverFromFunction(func) {
            return {
              getFunction: function(name2, namespace) {
                var found = func(name2, namespace);
                if (found) {
                  return makeEvaluator(found);
                }
                return defaultFunctionResolver.getFunction(name2, namespace);
              }
            };
          }
          function makeFunctionResolverFromObject(obj) {
            return makeFunctionResolverFromFunction(obj.getFunction.bind(obj));
          }
          function makeFunctionResolverFromMap(map2) {
            return makeFunctionResolverFromFunction(function(name2) {
              return map2[name2];
            });
          }
          function makeFunctionResolver(resolver) {
            if (resolver && typeof resolver.getFunction === "function") {
              return makeFunctionResolverFromObject(resolver);
            }
            if (typeof resolver === "function") {
              return makeFunctionResolverFromFunction(resolver);
            }
            if (typeof resolver === "object") {
              return makeFunctionResolverFromMap(resolver);
            }
            return defaultFunctionResolver;
          }
          function makeVariableResolverFromFunction(func) {
            return {
              getVariable: function(name2, namespace) {
                var value = func(name2, namespace);
                return convertValue(value);
              }
            };
          }
          function makeVariableResolver(resolver) {
            if (resolver) {
              if (typeof resolver.getVariable === "function") {
                return makeVariableResolverFromFunction(resolver.getVariable.bind(resolver));
              }
              if (typeof resolver === "function") {
                return makeVariableResolverFromFunction(resolver);
              }
              if (typeof resolver === "object") {
                return makeVariableResolverFromFunction(function(name2) {
                  return resolver[name2];
                });
              }
            }
            return defaultVariableResolver;
          }
          function copyIfPresent(prop, dest, source) {
            if (prop in source) {
              dest[prop] = source[prop];
            }
          }
          function makeContext(options) {
            var context = new XPathContext();
            if (options) {
              context.namespaceResolver = makeNSResolver(options.namespaces);
              context.functionResolver = makeFunctionResolver(options.functions);
              context.variableResolver = makeVariableResolver(options.variables);
              context.expressionContextNode = options.node;
              context.expressionContextPosition = options.position;
              context.expressionContextSize = options.size;
              copyIfPresent("allowAnyNamespaceForNoPrefix", context, options);
              copyIfPresent("isHtml", context, options);
            } else {
              context.namespaceResolver = defaultNSResolver;
            }
            return context;
          }
          function evaluate(parsedExpression, options) {
            var context = makeContext(options);
            return parsedExpression.evaluate(context);
          }
          var evaluatorPrototype = {
            evaluate: function(options) {
              return evaluate(this.expression, options);
            },
            evaluateNumber: function(options) {
              return this.evaluate(options).numberValue();
            },
            evaluateString: function(options) {
              return this.evaluate(options).stringValue();
            },
            evaluateBoolean: function(options) {
              return this.evaluate(options).booleanValue();
            },
            evaluateNodeSet: function(options) {
              return this.evaluate(options).nodeset();
            },
            select: function(options) {
              return this.evaluateNodeSet(options).toArray();
            },
            select1: function(options) {
              return this.select(options)[0];
            }
          };
          function parse(xpath3) {
            var parsed = parser.parse(xpath3);
            return Object.create(evaluatorPrototype, {
              expression: {
                value: parsed
              }
            });
          }
          exports2.parse = parse;
        })();
        assign(
          exports2,
          {
            XPath,
            XPathParser,
            XPathResult,
            Step,
            PathExpr,
            NodeTest,
            LocationPath,
            OrOperation,
            AndOperation,
            BarOperation,
            EqualsOperation,
            NotEqualOperation,
            LessThanOperation,
            GreaterThanOperation,
            LessThanOrEqualOperation,
            GreaterThanOrEqualOperation,
            PlusOperation,
            MinusOperation,
            MultiplyOperation,
            DivOperation,
            ModOperation,
            UnaryMinusOperation,
            FunctionCall,
            VariableReference,
            XPathContext,
            XNodeSet,
            XBoolean,
            XString,
            XNumber,
            NamespaceResolver,
            FunctionResolver,
            VariableResolver,
            Utilities
          }
        );
        exports2.select = function(e2, doc, single) {
          return exports2.selectWithResolver(e2, doc, null, single);
        };
        exports2.useNamespaces = function(mappings) {
          var resolver = {
            mappings: mappings || {},
            lookupNamespaceURI: function(prefix) {
              return this.mappings[prefix];
            }
          };
          return function(e2, doc, single) {
            return exports2.selectWithResolver(e2, doc, resolver, single);
          };
        };
        exports2.selectWithResolver = function(e2, doc, resolver, single) {
          var expression = new XPathExpression(e2, resolver, new XPathParser());
          var type = XPathResult.ANY_TYPE;
          var result = expression.evaluate(doc, type, null);
          if (result.resultType == XPathResult.STRING_TYPE) {
            result = result.stringValue;
          } else if (result.resultType == XPathResult.NUMBER_TYPE) {
            result = result.numberValue;
          } else if (result.resultType == XPathResult.BOOLEAN_TYPE) {
            result = result.booleanValue;
          } else {
            result = result.nodes;
            if (single) {
              result = result[0];
            }
          }
          return result;
        };
        exports2.select1 = function(e2, doc) {
          return exports2.select(e2, doc, true);
        };
        var isArrayOfNodes = function(value) {
          return Array.isArray(value) && value.every(isNodeLike);
        };
        var isNodeOfType = function(type) {
          return function(value) {
            return isNodeLike(value) && value.nodeType === type;
          };
        };
        assign(
          exports2,
          {
            isNodeLike,
            isArrayOfNodes,
            isElement: isNodeOfType(NodeTypes.ELEMENT_NODE),
            isAttribute: isNodeOfType(NodeTypes.ATTRIBUTE_NODE),
            isTextNode: isNodeOfType(NodeTypes.TEXT_NODE),
            isCDATASection: isNodeOfType(NodeTypes.CDATA_SECTION_NODE),
            isProcessingInstruction: isNodeOfType(NodeTypes.PROCESSING_INSTRUCTION_NODE),
            isComment: isNodeOfType(NodeTypes.COMMENT_NODE),
            isDocumentNode: isNodeOfType(NodeTypes.DOCUMENT_NODE),
            isDocumentTypeNode: isNodeOfType(NodeTypes.DOCUMENT_TYPE_NODE),
            isDocumentFragment: isNodeOfType(NodeTypes.DOCUMENT_FRAGMENT_NODE)
          }
        );
      })(xpath2);
    }
  });

  // node_modules/web-streams-polyfill/dist/ponyfill.mjs
  var ponyfill_exports = {};
  __export(ponyfill_exports, {
    ByteLengthQueuingStrategy: () => ByteLengthQueuingStrategy,
    CountQueuingStrategy: () => CountQueuingStrategy,
    ReadableByteStreamController: () => ReadableByteStreamController,
    ReadableStream: () => ReadableStream,
    ReadableStreamBYOBReader: () => ReadableStreamBYOBReader,
    ReadableStreamBYOBRequest: () => ReadableStreamBYOBRequest,
    ReadableStreamDefaultController: () => ReadableStreamDefaultController,
    ReadableStreamDefaultReader: () => ReadableStreamDefaultReader,
    TransformStream: () => TransformStream,
    TransformStreamDefaultController: () => TransformStreamDefaultController,
    WritableStream: () => WritableStream,
    WritableStreamDefaultController: () => WritableStreamDefaultController,
    WritableStreamDefaultWriter: () => WritableStreamDefaultWriter
  });
  function e() {
  }
  function t(e2) {
    return "object" == typeof e2 && null !== e2 || "function" == typeof e2;
  }
  var r = e;
  function o(e2, t2) {
    try {
      Object.defineProperty(e2, "name", { value: t2, configurable: true });
    } catch (e3) {
    }
  }
  var n = Promise;
  var i = Promise.resolve.bind(n);
  var a = Promise.prototype.then;
  var s = Promise.reject.bind(n);
  var l = i;
  function u(e2) {
    return new n(e2);
  }
  function c(e2) {
    return u((t2) => t2(e2));
  }
  function d(e2) {
    return s(e2);
  }
  function f(e2, t2, r2) {
    return a.call(e2, t2, r2);
  }
  function h(e2, t2, o2) {
    f(f(e2, t2, o2), void 0, r);
  }
  function b(e2, t2) {
    h(e2, t2);
  }
  function _(e2, t2) {
    h(e2, void 0, t2);
  }
  function m(e2, t2, r2) {
    return f(e2, t2, r2);
  }
  function p(e2) {
    f(e2, void 0, r);
  }
  var y = (e2) => {
    if ("function" == typeof queueMicrotask) y = queueMicrotask;
    else {
      const e3 = c(void 0);
      y = (t2) => f(e3, t2);
    }
    return y(e2);
  };
  function S(e2, t2, r2) {
    if ("function" != typeof e2) throw new TypeError("Argument is not a function");
    return Function.prototype.apply.call(e2, t2, r2);
  }
  function g(e2, t2, r2) {
    try {
      return c(S(e2, t2, r2));
    } catch (e3) {
      return d(e3);
    }
  }
  var v = class {
    constructor() {
      this._cursor = 0, this._size = 0, this._front = { _elements: [], _next: void 0 }, this._back = this._front, this._cursor = 0, this._size = 0;
    }
    get length() {
      return this._size;
    }
    push(e2) {
      const t2 = this._back;
      let r2 = t2;
      16383 === t2._elements.length && (r2 = { _elements: [], _next: void 0 }), t2._elements.push(e2), r2 !== t2 && (this._back = r2, t2._next = r2), ++this._size;
    }
    shift() {
      const e2 = this._front;
      let t2 = e2;
      const r2 = this._cursor;
      let o2 = r2 + 1;
      const n2 = e2._elements, i2 = n2[r2];
      return 16384 === o2 && (t2 = e2._next, o2 = 0), --this._size, this._cursor = o2, e2 !== t2 && (this._front = t2), n2[r2] = void 0, i2;
    }
    forEach(e2) {
      let t2 = this._cursor, r2 = this._front, o2 = r2._elements;
      for (; !(t2 === o2.length && void 0 === r2._next || t2 === o2.length && (r2 = r2._next, o2 = r2._elements, t2 = 0, 0 === o2.length)); ) e2(o2[t2]), ++t2;
    }
    peek() {
      const e2 = this._front, t2 = this._cursor;
      return e2._elements[t2];
    }
  };
  var w = /* @__PURE__ */ Symbol("[[AbortSteps]]");
  var R = /* @__PURE__ */ Symbol("[[ErrorSteps]]");
  var T = /* @__PURE__ */ Symbol("[[CancelSteps]]");
  var P = /* @__PURE__ */ Symbol("[[PullSteps]]");
  var C = /* @__PURE__ */ Symbol("[[CanPullSyncSteps]]");
  var q = /* @__PURE__ */ Symbol("[[ReleaseSteps]]");
  function E(e2, t2) {
    e2._ownerReadableStream = t2, t2._reader = e2, "readable" === t2._state ? j(e2) : "closed" === t2._state ? (function(e3) {
      j(e3), z(e3);
    })(e2) : k(e2, t2._storedError);
  }
  function W(e2, t2) {
    return Mr(e2._ownerReadableStream, t2);
  }
  function O(e2) {
    const t2 = e2._ownerReadableStream;
    "readable" === t2._state ? A(e2, new TypeError("Reader was released and can no longer be used to monitor the stream's closedness")) : (function(e3, t3) {
      k(e3, t3);
    })(e2, new TypeError("Reader was released and can no longer be used to monitor the stream's closedness")), t2._readableStreamController[q](), t2._reader = void 0, e2._ownerReadableStream = void 0;
  }
  function B(e2) {
    return new TypeError("Cannot " + e2 + " a stream using a released reader");
  }
  function j(e2) {
    e2._closedPromise = u((t2, r2) => {
      e2._closedPromise_resolve = t2, e2._closedPromise_reject = r2;
    });
  }
  function k(e2, t2) {
    j(e2), A(e2, t2);
  }
  function A(e2, t2) {
    void 0 !== e2._closedPromise_reject && (p(e2._closedPromise), e2._closedPromise_reject(t2), e2._closedPromise_resolve = void 0, e2._closedPromise_reject = void 0);
  }
  function z(e2) {
    void 0 !== e2._closedPromise_resolve && (e2._closedPromise_resolve(void 0), e2._closedPromise_resolve = void 0, e2._closedPromise_reject = void 0);
  }
  var D = Number.isFinite || function(e2) {
    return "number" == typeof e2 && isFinite(e2);
  };
  var F = Math.trunc || function(e2) {
    return e2 < 0 ? Math.ceil(e2) : Math.floor(e2);
  };
  function L(e2, t2) {
    if (void 0 !== e2 && ("object" != typeof (r2 = e2) && "function" != typeof r2)) throw new TypeError(`${t2} is not an object.`);
    var r2;
  }
  function I(e2, t2) {
    if ("function" != typeof e2) throw new TypeError(`${t2} is not a function.`);
  }
  function $(e2, t2) {
    if (!/* @__PURE__ */ (function(e3) {
      return "object" == typeof e3 && null !== e3 || "function" == typeof e3;
    })(e2)) throw new TypeError(`${t2} is not an object.`);
  }
  function M(e2, t2, r2) {
    if (void 0 === e2) throw new TypeError(`Parameter ${t2} is required in '${r2}'.`);
  }
  function Y(e2, t2, r2) {
    if (void 0 === e2) throw new TypeError(`${t2} is required in '${r2}'.`);
  }
  function x(e2) {
    return Number(e2);
  }
  function Q(e2) {
    return 0 === e2 ? 0 : e2;
  }
  function N(e2, t2) {
    const r2 = Number.MAX_SAFE_INTEGER;
    let o2 = Number(e2);
    if (o2 = Q(o2), !D(o2)) throw new TypeError(`${t2} is not a finite number`);
    if (o2 = (function(e3) {
      return Q(F(e3));
    })(o2), o2 < 0 || o2 > r2) throw new TypeError(`${t2} is outside the accepted range of 0 to ${r2}, inclusive`);
    return D(o2) && 0 !== o2 ? o2 : 0;
  }
  function H(e2, t2) {
    if (!Ir(e2)) throw new TypeError(`${t2} is not a ReadableStream.`);
  }
  function V(e2) {
    return new ReadableStreamDefaultReader(e2);
  }
  function U(e2, t2) {
    e2._reader._readRequests.push(t2);
  }
  function G(e2, t2, r2) {
    const o2 = e2._reader._readRequests.shift();
    r2 ? o2._closeSteps() : o2._chunkSteps(t2);
  }
  function X(e2) {
    return e2._reader._readRequests.length;
  }
  function J(e2) {
    const t2 = e2._reader;
    return void 0 !== t2 && !!ee(t2);
  }
  var ReadableStreamDefaultReader = class {
    constructor(e2) {
      if (M(e2, 1, "ReadableStreamDefaultReader"), H(e2, "First parameter"), $r(e2)) throw new TypeError("This stream has already been locked for exclusive reading by another reader");
      E(this, e2), this._readRequests = new v();
    }
    get closed() {
      return ee(this) ? this._closedPromise : d(ne("closed"));
    }
    cancel(e2 = void 0) {
      return ee(this) ? void 0 === this._ownerReadableStream ? d(B("cancel")) : W(this, e2) : d(ne("cancel"));
    }
    read() {
      if (!ee(this)) return d(ne("read"));
      if (void 0 === this._ownerReadableStream) return d(B("read from"));
      const e2 = re(this) ? new Z() : new K();
      return te(this, e2), e2._promise;
    }
    releaseLock() {
      if (!ee(this)) throw ne("releaseLock");
      void 0 !== this._ownerReadableStream && (function(e2) {
        O(e2);
        const t2 = new TypeError("Reader was released");
        oe(e2, t2);
      })(this);
    }
  };
  Object.defineProperties(ReadableStreamDefaultReader.prototype, { cancel: { enumerable: true }, read: { enumerable: true }, releaseLock: { enumerable: true }, closed: { enumerable: true } }), o(ReadableStreamDefaultReader.prototype.cancel, "cancel"), o(ReadableStreamDefaultReader.prototype.read, "read"), o(ReadableStreamDefaultReader.prototype.releaseLock, "releaseLock"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamDefaultReader.prototype, Symbol.toStringTag, { value: "ReadableStreamDefaultReader", configurable: true });
  var K = class {
    constructor() {
      this._promise = u((e2, t2) => {
        this._resolvePromise = e2, this._rejectPromise = t2;
      });
    }
    _chunkSteps(e2) {
      this._resolvePromise({ value: e2, done: false });
    }
    _closeSteps() {
      this._resolvePromise({ value: void 0, done: true });
    }
    _errorSteps(e2) {
      this._rejectPromise(e2);
    }
  };
  var Z = class {
    constructor() {
      this._promise = void 0;
    }
    _chunkSteps(e2) {
      this._promise = l({ value: e2, done: false });
    }
    _closeSteps() {
      this._promise = l({ value: void 0, done: true });
    }
    _errorSteps(e2) {
      this._promise = d(e2);
    }
  };
  function ee(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_readRequests") && e2 instanceof ReadableStreamDefaultReader);
  }
  function te(e2, t2) {
    const r2 = e2._ownerReadableStream;
    r2._disturbed = true, "closed" === r2._state ? t2._closeSteps() : "errored" === r2._state ? t2._errorSteps(r2._storedError) : r2._readableStreamController[P](t2);
  }
  function re(e2) {
    const t2 = e2._ownerReadableStream;
    return "closed" === t2._state || ("errored" === t2._state || t2._readableStreamController[C]());
  }
  function oe(e2, t2) {
    const r2 = e2._readRequests;
    e2._readRequests = new v(), r2.forEach((e3) => {
      e3._errorSteps(t2);
    });
  }
  function ne(e2) {
    return new TypeError(`ReadableStreamDefaultReader.prototype.${e2} can only be used on a ReadableStreamDefaultReader`);
  }
  var ie;
  var ae;
  var se;
  function le(e2) {
    return e2.slice();
  }
  function ue(e2, t2, r2, o2, n2) {
    new Uint8Array(e2).set(new Uint8Array(r2, o2, n2), t2);
  }
  var ce = (e2) => (ce = "function" == typeof e2.transfer ? (e3) => e3.transfer() : "function" == typeof structuredClone ? (e3) => structuredClone(e3, { transfer: [e3] }) : (e3) => e3, ce(e2));
  var de = (e2) => (de = "boolean" == typeof e2.detached ? (e3) => e3.detached : (e3) => 0 === e3.byteLength, de(e2));
  function fe(e2, t2, r2) {
    if (e2.slice) return e2.slice(t2, r2);
    const o2 = r2 - t2, n2 = new ArrayBuffer(o2);
    return ue(n2, 0, e2, t2, o2), n2;
  }
  function he(e2, t2) {
    const r2 = e2[t2];
    if (null != r2) {
      if ("function" != typeof r2) throw new TypeError(`${String(t2)} is not a function`);
      return r2;
    }
  }
  function be(e2) {
    try {
      const t2 = e2.done, r2 = e2.value;
      return f(l(r2), (e3) => ({ done: t2, value: e3 }));
    } catch (e3) {
      return d(e3);
    }
  }
  var _e = null !== (se = null !== (ie = Symbol.asyncIterator) && void 0 !== ie ? ie : null === (ae = Symbol.for) || void 0 === ae ? void 0 : ae.call(Symbol, "Symbol.asyncIterator")) && void 0 !== se ? se : "@@asyncIterator";
  function me(e2, r2 = "sync", o2) {
    if (void 0 === o2) if ("async" === r2) {
      if (void 0 === (o2 = he(e2, _e))) {
        return (function(e3) {
          const r3 = { next() {
            let t2;
            try {
              t2 = pe(e3);
            } catch (e4) {
              return d(e4);
            }
            return be(t2);
          }, return(r4) {
            let o3;
            try {
              const t2 = he(e3.iterator, "return");
              if (void 0 === t2) return c({ done: true, value: r4 });
              o3 = S(t2, e3.iterator, [r4]);
            } catch (e4) {
              return d(e4);
            }
            return t(o3) ? be(o3) : d(new TypeError("The iterator.return() method must return an object"));
          } };
          return { iterator: r3, nextMethod: r3.next, done: false };
        })(me(e2, "sync", he(e2, Symbol.iterator)));
      }
    } else o2 = he(e2, Symbol.iterator);
    if (void 0 === o2) throw new TypeError("The object is not iterable");
    const n2 = S(o2, e2, []);
    if (!t(n2)) throw new TypeError("The iterator method must return an object");
    return { iterator: n2, nextMethod: n2.next, done: false };
  }
  function pe(e2) {
    const r2 = S(e2.nextMethod, e2.iterator, []);
    if (!t(r2)) throw new TypeError("The iterator.next() method must return an object");
    return r2;
  }
  var ye = class {
    constructor(e2, t2) {
      this._ongoingPromise = void 0, this._isFinished = false, this._reader = e2, this._preventCancel = t2;
    }
    next() {
      const e2 = () => this._nextSteps();
      return this._ongoingPromise = this._ongoingPromise ? m(this._ongoingPromise, e2, e2) : e2(), this._ongoingPromise;
    }
    return(e2) {
      const t2 = () => this._returnSteps(e2);
      return this._ongoingPromise = this._ongoingPromise ? m(this._ongoingPromise, t2, t2) : t2(), this._ongoingPromise;
    }
    _nextSteps() {
      if (this._isFinished) return Promise.resolve({ value: void 0, done: true });
      const e2 = this._reader, t2 = new Se(this);
      return te(e2, t2), t2._promise;
    }
    _returnSteps(e2) {
      if (this._isFinished) return Promise.resolve({ value: e2, done: true });
      this._isFinished = true;
      const t2 = this._reader;
      if (!this._preventCancel) {
        const r2 = W(t2, e2);
        return O(t2), m(r2, () => ({ value: e2, done: true }));
      }
      return O(t2), c({ value: e2, done: true });
    }
  };
  var Se = class {
    constructor(e2) {
      this._iterator = e2, this._promise = u((e3, t2) => {
        this._resolvePromise = e3, this._rejectPromise = t2;
      });
    }
    _chunkSteps(e2) {
      this._iterator._ongoingPromise = void 0, y(() => this._resolvePromise({ value: e2, done: false }));
    }
    _closeSteps() {
      const e2 = this._iterator;
      e2._ongoingPromise = void 0, e2._isFinished = true, O(e2._reader), this._resolvePromise({ value: void 0, done: true });
    }
    _errorSteps(e2) {
      const t2 = this._iterator;
      t2._ongoingPromise = void 0, t2._isFinished = true, O(t2._reader), this._rejectPromise(e2);
    }
  };
  var ge = { next() {
    return ve(this) ? this._asyncIteratorImpl.next() : d(we("next"));
  }, return(e2) {
    return ve(this) ? this._asyncIteratorImpl.return(e2) : d(we("return"));
  }, [_e]() {
    return this;
  } };
  function ve(e2) {
    if (!t(e2)) return false;
    if (!Object.prototype.hasOwnProperty.call(e2, "_asyncIteratorImpl")) return false;
    try {
      return e2._asyncIteratorImpl instanceof ye;
    } catch (e3) {
      return false;
    }
  }
  function we(e2) {
    return new TypeError(`ReadableStreamAsyncIterator.${e2} can only be used on a ReadableSteamAsyncIterator`);
  }
  Object.defineProperty(ge, _e, { enumerable: false });
  var Re = Number.isNaN || function(e2) {
    return e2 != e2;
  };
  function Te(e2) {
    const t2 = fe(e2.buffer, e2.byteOffset, e2.byteOffset + e2.byteLength);
    return new Uint8Array(t2);
  }
  function Pe(e2) {
    const t2 = e2._queue.shift();
    return e2._queueTotalSize -= t2.size, e2._queueTotalSize < 0 && (e2._queueTotalSize = 0), t2.value;
  }
  function Ce(e2, t2, r2) {
    if ("number" != typeof (o2 = r2) || Re(o2) || o2 < 0 || r2 === 1 / 0) throw new RangeError("Size must be a finite, non-NaN, non-negative number.");
    var o2;
    e2._queue.push({ value: t2, size: r2 }), e2._queueTotalSize += r2;
  }
  function qe(e2) {
    e2._queue = new v(), e2._queueTotalSize = 0;
  }
  function Ee(e2) {
    return e2 === DataView;
  }
  function We(e2) {
    return Ee(e2) ? 1 : e2.BYTES_PER_ELEMENT;
  }
  var ReadableStreamBYOBRequest = class {
    constructor() {
      throw new TypeError("Illegal constructor");
    }
    get view() {
      if (!Be(this)) throw ot("view");
      return this._view;
    }
    respond(e2) {
      if (!Be(this)) throw ot("respond");
      if (M(e2, 1, "respond"), e2 = N(e2, "First parameter"), void 0 === this._associatedReadableByteStreamController) throw new TypeError("This BYOB request has been invalidated");
      if (de(this._view.buffer)) throw new TypeError("The BYOB request's buffer has been detached and so cannot be used as a response");
      et(this._associatedReadableByteStreamController, e2);
    }
    respondWithNewView(e2) {
      if (!Be(this)) throw ot("respondWithNewView");
      if (M(e2, 1, "respondWithNewView"), !ArrayBuffer.isView(e2)) throw new TypeError("You can only respond with array buffer views");
      if (void 0 === this._associatedReadableByteStreamController) throw new TypeError("This BYOB request has been invalidated");
      if (de(e2.buffer)) throw new TypeError("The given view's buffer has been detached and so cannot be used as a response");
      tt(this._associatedReadableByteStreamController, e2);
    }
  };
  Object.defineProperties(ReadableStreamBYOBRequest.prototype, { respond: { enumerable: true }, respondWithNewView: { enumerable: true }, view: { enumerable: true } }), o(ReadableStreamBYOBRequest.prototype.respond, "respond"), o(ReadableStreamBYOBRequest.prototype.respondWithNewView, "respondWithNewView"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamBYOBRequest.prototype, Symbol.toStringTag, { value: "ReadableStreamBYOBRequest", configurable: true });
  var ReadableByteStreamController = class {
    constructor() {
      throw new TypeError("Illegal constructor");
    }
    get byobRequest() {
      if (!Oe(this)) throw nt("byobRequest");
      return Ke(this);
    }
    get desiredSize() {
      if (!Oe(this)) throw nt("desiredSize");
      return Ze(this);
    }
    close() {
      if (!Oe(this)) throw nt("close");
      if (this._closeRequested) throw new TypeError("The stream has already been closed; do not close it again!");
      const e2 = this._controlledReadableByteStream._state;
      if ("readable" !== e2) throw new TypeError(`The stream (in ${e2} state) is not in the readable state and cannot be closed`);
      Ue(this);
    }
    enqueue(e2) {
      if (!Oe(this)) throw nt("enqueue");
      if (M(e2, 1, "enqueue"), !ArrayBuffer.isView(e2)) throw new TypeError("chunk must be an array buffer view");
      if (0 === e2.byteLength) throw new TypeError("chunk must have non-zero byteLength");
      if (0 === e2.buffer.byteLength) throw new TypeError("chunk's buffer must have non-zero byteLength");
      if (this._closeRequested) throw new TypeError("stream is closed or draining");
      const t2 = this._controlledReadableByteStream._state;
      if ("readable" !== t2) throw new TypeError(`The stream (in ${t2} state) is not in the readable state and cannot be enqueued to`);
      Ge(this, e2);
    }
    error(e2 = void 0) {
      if (!Oe(this)) throw nt("error");
      Xe(this, e2);
    }
    [T](e2) {
      ke(this), qe(this);
      const t2 = this._cancelAlgorithm(e2);
      return Ve(this), t2;
    }
    [P](e2) {
      const t2 = this._controlledReadableByteStream;
      if (this._queueTotalSize > 0) return void Je(this, e2);
      const r2 = this._autoAllocateChunkSize;
      if (void 0 !== r2) {
        let t3;
        try {
          t3 = new ArrayBuffer(r2);
        } catch (t4) {
          return void e2._errorSteps(t4);
        }
        const o2 = { buffer: t3, bufferByteLength: r2, byteOffset: 0, byteLength: r2, bytesFilled: 0, minimumFill: 1, elementSize: 1, viewConstructor: Uint8Array, readerType: "default" };
        this._pendingPullIntos.push(o2);
      }
      U(t2, e2), je(this);
    }
    [C]() {
      return this._queueTotalSize > 0;
    }
    [q]() {
      if (this._pendingPullIntos.length > 0) {
        const e2 = this._pendingPullIntos.peek();
        e2.readerType = "none", this._pendingPullIntos = new v(), this._pendingPullIntos.push(e2);
      }
    }
  };
  function Oe(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_controlledReadableByteStream") && e2 instanceof ReadableByteStreamController);
  }
  function Be(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_associatedReadableByteStreamController") && e2 instanceof ReadableStreamBYOBRequest);
  }
  function je(e2) {
    const t2 = (function(e3) {
      const t3 = e3._controlledReadableByteStream;
      if ("readable" !== t3._state) return false;
      if (e3._closeRequested) return false;
      if (!e3._started) return false;
      if (J(t3) && X(t3) > 0) return true;
      if (ut(t3) && lt(t3) > 0) return true;
      const r2 = Ze(e3);
      if (r2 > 0) return true;
      return false;
    })(e2);
    if (!t2) return;
    if (e2._pulling) return void (e2._pullAgain = true);
    e2._pulling = true;
    h(e2._pullAlgorithm(), () => (e2._pulling = false, e2._pullAgain && (e2._pullAgain = false, je(e2)), null), (t3) => (Xe(e2, t3), null));
  }
  function ke(e2) {
    xe(e2), e2._pendingPullIntos = new v();
  }
  function Ae(e2, t2) {
    let r2 = false;
    "closed" === e2._state && (r2 = true);
    const o2 = De(t2);
    "default" === t2.readerType ? G(e2, o2, r2) : (function(e3, t3, r3) {
      const o3 = e3._reader, n2 = o3._readIntoRequests.shift();
      r3 ? n2._closeSteps(t3) : n2._chunkSteps(t3);
    })(e2, o2, r2);
  }
  function ze(e2, t2) {
    for (let r2 = 0; r2 < t2.length; ++r2) Ae(e2, t2[r2]);
  }
  function De(e2) {
    const t2 = e2.bytesFilled, r2 = e2.elementSize;
    return new e2.viewConstructor(e2.buffer, e2.byteOffset, t2 / r2);
  }
  function Fe(e2, t2, r2, o2) {
    e2._queue.push({ buffer: t2, byteOffset: r2, byteLength: o2 }), e2._queueTotalSize += o2;
  }
  function Le(e2, t2, r2, o2) {
    let n2;
    try {
      n2 = fe(t2, r2, r2 + o2);
    } catch (t3) {
      throw Xe(e2, t3), t3;
    }
    Fe(e2, n2, 0, o2);
  }
  function Ie(e2, t2) {
    t2.bytesFilled > 0 && Le(e2, t2.buffer, t2.byteOffset, t2.bytesFilled), He(e2);
  }
  function $e(e2, t2) {
    const r2 = Math.min(e2._queueTotalSize, t2.byteLength - t2.bytesFilled), o2 = t2.bytesFilled + r2;
    let n2 = r2, i2 = false;
    const a2 = o2 - o2 % t2.elementSize;
    a2 >= t2.minimumFill && (n2 = a2 - t2.bytesFilled, i2 = true);
    const s2 = e2._queue;
    for (; n2 > 0; ) {
      const r3 = s2.peek(), o3 = Math.min(n2, r3.byteLength), i3 = t2.byteOffset + t2.bytesFilled;
      ue(t2.buffer, i3, r3.buffer, r3.byteOffset, o3), r3.byteLength === o3 ? s2.shift() : (r3.byteOffset += o3, r3.byteLength -= o3), e2._queueTotalSize -= o3, Me(e2, o3, t2), n2 -= o3;
    }
    return i2;
  }
  function Me(e2, t2, r2) {
    r2.bytesFilled += t2;
  }
  function Ye(e2) {
    0 === e2._queueTotalSize && e2._closeRequested ? (Ve(e2), Yr(e2._controlledReadableByteStream)) : je(e2);
  }
  function xe(e2) {
    null !== e2._byobRequest && (e2._byobRequest._associatedReadableByteStreamController = void 0, e2._byobRequest._view = null, e2._byobRequest = null);
  }
  function Qe(e2) {
    const t2 = [];
    for (; e2._pendingPullIntos.length > 0 && 0 !== e2._queueTotalSize; ) {
      const r2 = e2._pendingPullIntos.peek();
      $e(e2, r2) && (He(e2), t2.push(r2));
    }
    return t2;
  }
  function Ne(e2, t2) {
    const r2 = e2._pendingPullIntos.peek();
    xe(e2);
    "closed" === e2._controlledReadableByteStream._state ? (function(e3, t3) {
      "none" === t3.readerType && He(e3);
      const r3 = e3._controlledReadableByteStream;
      if (ut(r3)) {
        const t4 = [];
        for (; t4.length < lt(r3); ) t4.push(He(e3));
        ze(r3, t4);
      }
    })(e2, r2) : (function(e3, t3, r3) {
      if (Me(0, t3, r3), "none" === r3.readerType) {
        Ie(e3, r3);
        const t4 = Qe(e3);
        return void ze(e3._controlledReadableByteStream, t4);
      }
      if (r3.bytesFilled < r3.minimumFill) return;
      He(e3);
      const o2 = r3.bytesFilled % r3.elementSize;
      if (o2 > 0) {
        const t4 = r3.byteOffset + r3.bytesFilled;
        Le(e3, r3.buffer, t4 - o2, o2);
      }
      r3.bytesFilled -= o2;
      const n2 = Qe(e3);
      Ae(e3._controlledReadableByteStream, r3), ze(e3._controlledReadableByteStream, n2);
    })(e2, t2, r2), je(e2);
  }
  function He(e2) {
    return e2._pendingPullIntos.shift();
  }
  function Ve(e2) {
    e2._pullAlgorithm = void 0, e2._cancelAlgorithm = void 0;
  }
  function Ue(e2) {
    const t2 = e2._controlledReadableByteStream;
    if (!e2._closeRequested && "readable" === t2._state) if (e2._queueTotalSize > 0) e2._closeRequested = true;
    else {
      if (e2._pendingPullIntos.length > 0) {
        const t3 = e2._pendingPullIntos.peek();
        if (t3.bytesFilled % t3.elementSize !== 0) {
          const t4 = new TypeError("Insufficient bytes to fill elements in the given buffer");
          throw Xe(e2, t4), t4;
        }
      }
      Ve(e2), Yr(t2);
    }
  }
  function Ge(e2, t2) {
    const r2 = e2._controlledReadableByteStream;
    if (e2._closeRequested || "readable" !== r2._state) return;
    const { buffer: o2, byteOffset: n2, byteLength: i2 } = t2;
    if (de(o2)) throw new TypeError("chunk's buffer is detached and so cannot be enqueued");
    const a2 = ce(o2);
    if (e2._pendingPullIntos.length > 0) {
      const t3 = e2._pendingPullIntos.peek();
      if (de(t3.buffer)) throw new TypeError("The BYOB request's buffer has been detached and so cannot be filled with an enqueued chunk");
      xe(e2), t3.buffer = ce(t3.buffer), "none" === t3.readerType && Ie(e2, t3);
    }
    if (J(r2)) if ((function(e3) {
      const t3 = e3._controlledReadableByteStream._reader;
      for (; t3._readRequests.length > 0; ) {
        if (0 === e3._queueTotalSize) return;
        Je(e3, t3._readRequests.shift());
      }
    })(e2), 0 === X(r2)) Fe(e2, a2, n2, i2);
    else {
      e2._pendingPullIntos.length > 0 && He(e2);
      G(r2, new Uint8Array(a2, n2, i2), false);
    }
    else if (ut(r2)) {
      Fe(e2, a2, n2, i2);
      ze(r2, Qe(e2));
    } else Fe(e2, a2, n2, i2);
    je(e2);
  }
  function Xe(e2, t2) {
    const r2 = e2._controlledReadableByteStream;
    "readable" === r2._state && (ke(e2), qe(e2), Ve(e2), xr(r2, t2));
  }
  function Je(e2, t2) {
    const r2 = e2._queue.shift();
    e2._queueTotalSize -= r2.byteLength, Ye(e2);
    const o2 = new Uint8Array(r2.buffer, r2.byteOffset, r2.byteLength);
    t2._chunkSteps(o2);
  }
  function Ke(e2) {
    if (null === e2._byobRequest && e2._pendingPullIntos.length > 0) {
      const t2 = e2._pendingPullIntos.peek(), r2 = new Uint8Array(t2.buffer, t2.byteOffset + t2.bytesFilled, t2.byteLength - t2.bytesFilled), o2 = Object.create(ReadableStreamBYOBRequest.prototype);
      !(function(e3, t3, r3) {
        e3._associatedReadableByteStreamController = t3, e3._view = r3;
      })(o2, e2, r2), e2._byobRequest = o2;
    }
    return e2._byobRequest;
  }
  function Ze(e2) {
    const t2 = e2._controlledReadableByteStream._state;
    return "errored" === t2 ? null : "closed" === t2 ? 0 : e2._strategyHWM - e2._queueTotalSize;
  }
  function et(e2, t2) {
    const r2 = e2._pendingPullIntos.peek();
    if ("closed" === e2._controlledReadableByteStream._state) {
      if (0 !== t2) throw new TypeError("bytesWritten must be 0 when calling respond() on a closed stream");
    } else {
      if (0 === t2) throw new TypeError("bytesWritten must be greater than 0 when calling respond() on a readable stream");
      if (r2.bytesFilled + t2 > r2.byteLength) throw new RangeError("bytesWritten out of range");
    }
    r2.buffer = ce(r2.buffer), Ne(e2, t2);
  }
  function tt(e2, t2) {
    const r2 = e2._pendingPullIntos.peek();
    if ("closed" === e2._controlledReadableByteStream._state) {
      if (0 !== t2.byteLength) throw new TypeError("The view's length must be 0 when calling respondWithNewView() on a closed stream");
    } else if (0 === t2.byteLength) throw new TypeError("The view's length must be greater than 0 when calling respondWithNewView() on a readable stream");
    if (r2.byteOffset + r2.bytesFilled !== t2.byteOffset) throw new RangeError("The region specified by view does not match byobRequest");
    if (r2.bufferByteLength !== t2.buffer.byteLength) throw new RangeError("The buffer of view has different capacity than byobRequest");
    if (r2.bytesFilled + t2.byteLength > r2.byteLength) throw new RangeError("The region specified by view is larger than byobRequest");
    const o2 = t2.byteLength;
    r2.buffer = ce(t2.buffer), Ne(e2, o2);
  }
  function rt(e2, t2, r2, o2, n2, i2, a2) {
    t2._controlledReadableByteStream = e2, t2._pullAgain = false, t2._pulling = false, t2._byobRequest = null, t2._queue = t2._queueTotalSize = void 0, qe(t2), t2._closeRequested = false, t2._started = false, t2._strategyHWM = i2, t2._pullAlgorithm = o2, t2._cancelAlgorithm = n2, t2._autoAllocateChunkSize = a2, t2._pendingPullIntos = new v(), e2._readableStreamController = t2;
    h(c(r2()), () => (t2._started = true, je(t2), null), (e3) => (Xe(t2, e3), null));
  }
  function ot(e2) {
    return new TypeError(`ReadableStreamBYOBRequest.prototype.${e2} can only be used on a ReadableStreamBYOBRequest`);
  }
  function nt(e2) {
    return new TypeError(`ReadableByteStreamController.prototype.${e2} can only be used on a ReadableByteStreamController`);
  }
  function it(e2, t2) {
    if ("byob" !== (e2 = `${e2}`)) throw new TypeError(`${t2} '${e2}' is not a valid enumeration value for ReadableStreamReaderMode`);
    return e2;
  }
  function at(e2) {
    return new ReadableStreamBYOBReader(e2);
  }
  function st(e2, t2) {
    e2._reader._readIntoRequests.push(t2);
  }
  function lt(e2) {
    return e2._reader._readIntoRequests.length;
  }
  function ut(e2) {
    const t2 = e2._reader;
    return void 0 !== t2 && !!ft(t2);
  }
  Object.defineProperties(ReadableByteStreamController.prototype, { close: { enumerable: true }, enqueue: { enumerable: true }, error: { enumerable: true }, byobRequest: { enumerable: true }, desiredSize: { enumerable: true } }), o(ReadableByteStreamController.prototype.close, "close"), o(ReadableByteStreamController.prototype.enqueue, "enqueue"), o(ReadableByteStreamController.prototype.error, "error"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableByteStreamController.prototype, Symbol.toStringTag, { value: "ReadableByteStreamController", configurable: true });
  var ReadableStreamBYOBReader = class {
    constructor(e2) {
      if (M(e2, 1, "ReadableStreamBYOBReader"), H(e2, "First parameter"), $r(e2)) throw new TypeError("This stream has already been locked for exclusive reading by another reader");
      if (!Oe(e2._readableStreamController)) throw new TypeError("Cannot construct a ReadableStreamBYOBReader for a stream not constructed with a byte source");
      E(this, e2), this._readIntoRequests = new v();
    }
    get closed() {
      return ft(this) ? this._closedPromise : d(_t("closed"));
    }
    cancel(e2 = void 0) {
      return ft(this) ? void 0 === this._ownerReadableStream ? d(B("cancel")) : W(this, e2) : d(_t("cancel"));
    }
    read(e2, t2 = {}) {
      if (!ft(this)) return d(_t("read"));
      if (!ArrayBuffer.isView(e2)) return d(new TypeError("view must be an array buffer view"));
      if (0 === e2.byteLength) return d(new TypeError("view must have non-zero byteLength"));
      if (0 === e2.buffer.byteLength) return d(new TypeError("view's buffer must have non-zero byteLength"));
      if (de(e2.buffer)) return d(new TypeError("view's buffer has been detached"));
      let r2;
      try {
        r2 = (function(e3, t3) {
          var r3;
          return L(e3, t3), { min: N(null !== (r3 = null == e3 ? void 0 : e3.min) && void 0 !== r3 ? r3 : 1, `${t3} has member 'min' that`) };
        })(t2, "options");
      } catch (e3) {
        return d(e3);
      }
      const o2 = r2.min;
      if (0 === o2) return d(new TypeError("options.min must be greater than 0"));
      if ((function(e3) {
        return Ee(e3.constructor);
      })(e2)) {
        if (o2 > e2.byteLength) return d(new RangeError("options.min must be less than or equal to view's byteLength"));
      } else if (o2 > e2.length) return d(new RangeError("options.min must be less than or equal to view's length"));
      if (void 0 === this._ownerReadableStream) return d(B("read from"));
      const n2 = (function(e3, t3, r3) {
        const o3 = e3._ownerReadableStream;
        return "errored" === o3._state || (function(e4, t4, r4) {
          const o4 = e4._controlledReadableByteStream, n3 = We(t4.constructor), { byteLength: i2 } = t4, a2 = r4 * n3;
          return !(e4._pendingPullIntos.length > 0) && ("closed" === o4._state || e4._queueTotalSize >= a2);
        })(o3._readableStreamController, t3, r3);
      })(this, e2, o2) ? new dt() : new ct();
      return ht(this, e2, o2, n2), n2._promise;
    }
    releaseLock() {
      if (!ft(this)) throw _t("releaseLock");
      void 0 !== this._ownerReadableStream && (function(e2) {
        O(e2);
        const t2 = new TypeError("Reader was released");
        bt(e2, t2);
      })(this);
    }
  };
  Object.defineProperties(ReadableStreamBYOBReader.prototype, { cancel: { enumerable: true }, read: { enumerable: true }, releaseLock: { enumerable: true }, closed: { enumerable: true } }), o(ReadableStreamBYOBReader.prototype.cancel, "cancel"), o(ReadableStreamBYOBReader.prototype.read, "read"), o(ReadableStreamBYOBReader.prototype.releaseLock, "releaseLock"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamBYOBReader.prototype, Symbol.toStringTag, { value: "ReadableStreamBYOBReader", configurable: true });
  var ct = class {
    constructor() {
      this._promise = u((e2, t2) => {
        this._resolvePromise = e2, this._rejectPromise = t2;
      });
    }
    _chunkSteps(e2) {
      this._resolvePromise({ value: e2, done: false });
    }
    _closeSteps(e2) {
      this._resolvePromise({ value: e2, done: true });
    }
    _errorSteps(e2) {
      this._rejectPromise(e2);
    }
  };
  var dt = class {
    constructor() {
      this._promise = void 0;
    }
    _chunkSteps(e2) {
      this._promise = l({ value: e2, done: false });
    }
    _closeSteps(e2) {
      this._promise = l({ value: e2, done: true });
    }
    _errorSteps(e2) {
      this._promise = d(e2);
    }
  };
  function ft(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_readIntoRequests") && e2 instanceof ReadableStreamBYOBReader);
  }
  function ht(e2, t2, r2, o2) {
    const n2 = e2._ownerReadableStream;
    n2._disturbed = true, "errored" === n2._state ? o2._errorSteps(n2._storedError) : (function(e3, t3, r3, o3) {
      const n3 = e3._controlledReadableByteStream, i2 = t3.constructor, a2 = We(i2), { byteOffset: s2, byteLength: l2 } = t3, u2 = r3 * a2;
      let c2;
      try {
        c2 = ce(t3.buffer);
      } catch (e4) {
        return void o3._errorSteps(e4);
      }
      const d2 = { buffer: c2, bufferByteLength: c2.byteLength, byteOffset: s2, byteLength: l2, bytesFilled: 0, minimumFill: u2, elementSize: a2, viewConstructor: i2, readerType: "byob" };
      if (e3._pendingPullIntos.length > 0) return e3._pendingPullIntos.push(d2), void st(n3, o3);
      if ("closed" === n3._state) {
        const e4 = new i2(d2.buffer, d2.byteOffset, 0);
        return void o3._closeSteps(e4);
      }
      if (e3._queueTotalSize > 0) {
        if ($e(e3, d2)) {
          const t4 = De(d2);
          return Ye(e3), void o3._chunkSteps(t4);
        }
        if (e3._closeRequested) {
          const t4 = new TypeError("Insufficient bytes to fill elements in the given buffer");
          return Xe(e3, t4), void o3._errorSteps(t4);
        }
      }
      e3._pendingPullIntos.push(d2), st(n3, o3), je(e3);
    })(n2._readableStreamController, t2, r2, o2);
  }
  function bt(e2, t2) {
    const r2 = e2._readIntoRequests;
    e2._readIntoRequests = new v(), r2.forEach((e3) => {
      e3._errorSteps(t2);
    });
  }
  function _t(e2) {
    return new TypeError(`ReadableStreamBYOBReader.prototype.${e2} can only be used on a ReadableStreamBYOBReader`);
  }
  function mt(e2, t2) {
    const { highWaterMark: r2 } = e2;
    if (void 0 === r2) return t2;
    if (Re(r2) || r2 < 0) throw new RangeError("Invalid highWaterMark");
    return r2;
  }
  function pt(e2) {
    const { size: t2 } = e2;
    return t2 || (() => 1);
  }
  function yt(e2, t2) {
    L(e2, t2);
    const r2 = null == e2 ? void 0 : e2.highWaterMark, o2 = null == e2 ? void 0 : e2.size;
    return { highWaterMark: void 0 === r2 ? void 0 : x(r2), size: void 0 === o2 ? void 0 : St(o2, `${t2} has member 'size' that`) };
  }
  function St(e2, t2) {
    return I(e2, t2), (t3) => x(e2(t3));
  }
  function gt(e2, t2, r2) {
    return I(e2, r2), (r3) => g(e2, t2, [r3]);
  }
  function vt(e2, t2, r2) {
    return I(e2, r2), () => g(e2, t2, []);
  }
  function wt(e2, t2, r2) {
    return I(e2, r2), (r3) => S(e2, t2, [r3]);
  }
  function Rt(e2, t2, r2) {
    return I(e2, r2), (r3, o2) => g(e2, t2, [r3, o2]);
  }
  function Tt(e2, t2) {
    if (!qt(e2)) throw new TypeError(`${t2} is not a WritableStream.`);
  }
  var WritableStream = class {
    constructor(e2 = {}, t2 = {}) {
      void 0 === e2 ? e2 = null : $(e2, "First parameter");
      const r2 = yt(t2, "Second parameter"), o2 = (function(e3, t3) {
        L(e3, t3);
        const r3 = null == e3 ? void 0 : e3.abort, o3 = null == e3 ? void 0 : e3.close, n3 = null == e3 ? void 0 : e3.start, i2 = null == e3 ? void 0 : e3.type, a2 = null == e3 ? void 0 : e3.write;
        return { abort: void 0 === r3 ? void 0 : gt(r3, e3, `${t3} has member 'abort' that`), close: void 0 === o3 ? void 0 : vt(o3, e3, `${t3} has member 'close' that`), start: void 0 === n3 ? void 0 : wt(n3, e3, `${t3} has member 'start' that`), write: void 0 === a2 ? void 0 : Rt(a2, e3, `${t3} has member 'write' that`), type: i2 };
      })(e2, "First parameter");
      Ct(this);
      if (void 0 !== o2.type) throw new RangeError("Invalid type is specified");
      const n2 = pt(r2);
      !(function(e3, t3, r3, o3) {
        const n3 = Object.create(WritableStreamDefaultController.prototype);
        let i2, a2, s2, l2;
        i2 = void 0 !== t3.start ? () => t3.start(n3) : () => {
        };
        a2 = void 0 !== t3.write ? (e4) => t3.write(e4, n3) : () => c(void 0);
        s2 = void 0 !== t3.close ? () => t3.close() : () => c(void 0);
        l2 = void 0 !== t3.abort ? (e4) => t3.abort(e4) : () => c(void 0);
        Nt(e3, n3, i2, a2, s2, l2, r3, o3);
      })(this, o2, mt(r2, 1), n2);
    }
    get locked() {
      if (!qt(this)) throw Kt("locked");
      return Et(this);
    }
    abort(e2 = void 0) {
      return qt(this) ? Et(this) ? d(new TypeError("Cannot abort a stream that already has a writer")) : Wt(this, e2) : d(Kt("abort"));
    }
    close() {
      return qt(this) ? Et(this) ? d(new TypeError("Cannot close a stream that already has a writer")) : At(this) ? d(new TypeError("Cannot close an already-closing stream")) : Ot(this) : d(Kt("close"));
    }
    getWriter() {
      if (!qt(this)) throw Kt("getWriter");
      return Pt(this);
    }
  };
  function Pt(e2) {
    return new WritableStreamDefaultWriter(e2);
  }
  function Ct(e2) {
    e2._state = "writable", e2._storedError = void 0, e2._writer = void 0, e2._writableStreamController = void 0, e2._writeRequests = new v(), e2._inFlightWriteRequest = void 0, e2._closeRequest = void 0, e2._inFlightCloseRequest = void 0, e2._pendingAbortRequest = void 0, e2._backpressure = false;
  }
  function qt(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_writableStreamController") && e2 instanceof WritableStream);
  }
  function Et(e2) {
    return void 0 !== e2._writer;
  }
  function Wt(e2, t2) {
    var r2;
    if ("closed" === e2._state || "errored" === e2._state) return c(void 0);
    e2._writableStreamController._abortReason = t2, null === (r2 = e2._writableStreamController._abortController) || void 0 === r2 || r2.abort(t2);
    const o2 = e2._state;
    if ("closed" === o2 || "errored" === o2) return c(void 0);
    if (void 0 !== e2._pendingAbortRequest) return e2._pendingAbortRequest._promise;
    let n2 = false;
    "erroring" === o2 && (n2 = true, t2 = void 0);
    const i2 = u((r3, o3) => {
      e2._pendingAbortRequest = { _promise: void 0, _resolve: r3, _reject: o3, _reason: t2, _wasAlreadyErroring: n2 };
    });
    return e2._pendingAbortRequest._promise = i2, n2 || jt(e2, t2), i2;
  }
  function Ot(e2) {
    const t2 = e2._state;
    if ("closed" === t2 || "errored" === t2) return d(new TypeError(`The stream (in ${t2} state) is not in the writable state and cannot be closed`));
    const r2 = u((t3, r3) => {
      const o3 = { _resolve: t3, _reject: r3 };
      e2._closeRequest = o3;
    }), o2 = e2._writer;
    var n2;
    return void 0 !== o2 && e2._backpressure && "writable" === t2 && cr(o2), Ce(n2 = e2._writableStreamController, xt, 0), Ut(n2), r2;
  }
  function Bt(e2, t2) {
    "writable" !== e2._state ? kt(e2) : jt(e2, t2);
  }
  function jt(e2, t2) {
    const r2 = e2._writableStreamController;
    e2._state = "erroring", e2._storedError = t2;
    const o2 = e2._writer;
    void 0 !== o2 && $t(o2, t2), !(function(e3) {
      if (void 0 === e3._inFlightWriteRequest && void 0 === e3._inFlightCloseRequest) return false;
      return true;
    })(e2) && r2._started && kt(e2);
  }
  function kt(e2) {
    e2._state = "errored", e2._writableStreamController[R]();
    const t2 = e2._storedError;
    if (e2._writeRequests.forEach((e3) => {
      e3._reject(t2);
    }), e2._writeRequests = new v(), void 0 === e2._pendingAbortRequest) return void zt(e2);
    const r2 = e2._pendingAbortRequest;
    if (e2._pendingAbortRequest = void 0, r2._wasAlreadyErroring) return r2._reject(t2), void zt(e2);
    h(e2._writableStreamController[w](r2._reason), () => (r2._resolve(), zt(e2), null), (t3) => (r2._reject(t3), zt(e2), null));
  }
  function At(e2) {
    return void 0 !== e2._closeRequest || void 0 !== e2._inFlightCloseRequest;
  }
  function zt(e2) {
    void 0 !== e2._closeRequest && (e2._closeRequest._reject(e2._storedError), e2._closeRequest = void 0);
    const t2 = e2._writer;
    void 0 !== t2 && nr(t2, e2._storedError);
  }
  function Dt(e2, t2) {
    const r2 = e2._writer;
    void 0 !== r2 && t2 !== e2._backpressure && (t2 ? (function(e3) {
      ar(e3);
    })(r2) : cr(r2)), e2._backpressure = t2;
  }
  Object.defineProperties(WritableStream.prototype, { abort: { enumerable: true }, close: { enumerable: true }, getWriter: { enumerable: true }, locked: { enumerable: true } }), o(WritableStream.prototype.abort, "abort"), o(WritableStream.prototype.close, "close"), o(WritableStream.prototype.getWriter, "getWriter"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(WritableStream.prototype, Symbol.toStringTag, { value: "WritableStream", configurable: true });
  var WritableStreamDefaultWriter = class {
    constructor(e2) {
      if (M(e2, 1, "WritableStreamDefaultWriter"), Tt(e2, "First parameter"), Et(e2)) throw new TypeError("This stream has already been locked for exclusive writing by another writer");
      this._ownerWritableStream = e2, e2._writer = this;
      const t2 = e2._state;
      if ("writable" === t2) !At(e2) && e2._backpressure ? ar(this) : lr(this), rr(this);
      else if ("erroring" === t2) sr(this, e2._storedError), rr(this);
      else if ("closed" === t2) lr(this), rr(r2 = this), ir(r2);
      else {
        const t3 = e2._storedError;
        sr(this, t3), or(this, t3);
      }
      var r2;
    }
    get closed() {
      return Ft(this) ? this._closedPromise : d(er("closed"));
    }
    get desiredSize() {
      if (!Ft(this)) throw er("desiredSize");
      if (void 0 === this._ownerWritableStream) throw tr("desiredSize");
      return (function(e2) {
        const t2 = e2._ownerWritableStream, r2 = t2._state;
        if ("errored" === r2 || "erroring" === r2) return null;
        if ("closed" === r2) return 0;
        return Vt(t2._writableStreamController);
      })(this);
    }
    get ready() {
      return Ft(this) ? this._readyPromise : d(er("ready"));
    }
    abort(e2 = void 0) {
      return Ft(this) ? void 0 === this._ownerWritableStream ? d(tr("abort")) : (function(e3, t2) {
        return Wt(e3._ownerWritableStream, t2);
      })(this, e2) : d(er("abort"));
    }
    close() {
      if (!Ft(this)) return d(er("close"));
      const e2 = this._ownerWritableStream;
      return void 0 === e2 ? d(tr("close")) : At(e2) ? d(new TypeError("Cannot close an already-closing stream")) : Lt(this);
    }
    releaseLock() {
      if (!Ft(this)) throw er("releaseLock");
      void 0 !== this._ownerWritableStream && Mt(this);
    }
    write(e2 = void 0) {
      return Ft(this) ? void 0 === this._ownerWritableStream ? d(tr("write to")) : Yt(this, e2) : d(er("write"));
    }
  };
  function Ft(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_ownerWritableStream") && e2 instanceof WritableStreamDefaultWriter);
  }
  function Lt(e2) {
    return Ot(e2._ownerWritableStream);
  }
  function It(e2, t2) {
    "pending" === e2._closedPromiseState ? nr(e2, t2) : (function(e3, t3) {
      or(e3, t3);
    })(e2, t2);
  }
  function $t(e2, t2) {
    "pending" === e2._readyPromiseState ? ur(e2, t2) : (function(e3, t3) {
      sr(e3, t3);
    })(e2, t2);
  }
  function Mt(e2) {
    const t2 = e2._ownerWritableStream, r2 = new TypeError("Writer was released and can no longer be used to monitor the stream's closedness");
    $t(e2, r2), It(e2, r2), t2._writer = void 0, e2._ownerWritableStream = void 0;
  }
  function Yt(e2, t2) {
    const r2 = e2._ownerWritableStream, o2 = r2._writableStreamController, n2 = (function(e3, t3) {
      if (void 0 === e3._strategySizeAlgorithm) return 1;
      try {
        return e3._strategySizeAlgorithm(t3);
      } catch (t4) {
        return Gt(e3, t4), 1;
      }
    })(o2, t2);
    if (r2 !== e2._ownerWritableStream) return d(tr("write to"));
    const i2 = r2._state;
    if ("errored" === i2) return d(r2._storedError);
    if (At(r2) || "closed" === i2) return d(new TypeError("The stream is closing or closed and cannot be written to"));
    if ("erroring" === i2) return d(r2._storedError);
    const a2 = (function(e3) {
      return u((t3, r3) => {
        const o3 = { _resolve: t3, _reject: r3 };
        e3._writeRequests.push(o3);
      });
    })(r2);
    return (function(e3, t3, r3) {
      try {
        Ce(e3, t3, r3);
      } catch (t4) {
        return void Gt(e3, t4);
      }
      const o3 = e3._controlledWritableStream;
      if (!At(o3) && "writable" === o3._state) {
        Dt(o3, Xt(e3));
      }
      Ut(e3);
    })(o2, t2, n2), a2;
  }
  Object.defineProperties(WritableStreamDefaultWriter.prototype, { abort: { enumerable: true }, close: { enumerable: true }, releaseLock: { enumerable: true }, write: { enumerable: true }, closed: { enumerable: true }, desiredSize: { enumerable: true }, ready: { enumerable: true } }), o(WritableStreamDefaultWriter.prototype.abort, "abort"), o(WritableStreamDefaultWriter.prototype.close, "close"), o(WritableStreamDefaultWriter.prototype.releaseLock, "releaseLock"), o(WritableStreamDefaultWriter.prototype.write, "write"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(WritableStreamDefaultWriter.prototype, Symbol.toStringTag, { value: "WritableStreamDefaultWriter", configurable: true });
  var xt = {};
  var WritableStreamDefaultController = class {
    constructor() {
      throw new TypeError("Illegal constructor");
    }
    get abortReason() {
      if (!Qt(this)) throw Zt("abortReason");
      return this._abortReason;
    }
    get signal() {
      if (!Qt(this)) throw Zt("signal");
      if (void 0 === this._abortController) throw new TypeError("WritableStreamDefaultController.prototype.signal is not supported");
      return this._abortController.signal;
    }
    error(e2 = void 0) {
      if (!Qt(this)) throw Zt("error");
      "writable" === this._controlledWritableStream._state && Jt(this, e2);
    }
    [w](e2) {
      const t2 = this._abortAlgorithm(e2);
      return Ht(this), t2;
    }
    [R]() {
      qe(this);
    }
  };
  function Qt(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_controlledWritableStream") && e2 instanceof WritableStreamDefaultController);
  }
  function Nt(e2, t2, r2, o2, n2, i2, a2, s2) {
    t2._controlledWritableStream = e2, e2._writableStreamController = t2, t2._queue = void 0, t2._queueTotalSize = void 0, qe(t2), t2._abortReason = void 0, t2._abortController = (function() {
      if ("function" == typeof AbortController) return new AbortController();
    })(), t2._started = false, t2._strategySizeAlgorithm = s2, t2._strategyHWM = a2, t2._writeAlgorithm = o2, t2._closeAlgorithm = n2, t2._abortAlgorithm = i2;
    const l2 = Xt(t2);
    Dt(e2, l2);
    h(c(r2()), () => (t2._started = true, Ut(t2), null), (r3) => (t2._started = true, Bt(e2, r3), null));
  }
  function Ht(e2) {
    e2._writeAlgorithm = void 0, e2._closeAlgorithm = void 0, e2._abortAlgorithm = void 0, e2._strategySizeAlgorithm = void 0;
  }
  function Vt(e2) {
    return e2._strategyHWM - e2._queueTotalSize;
  }
  function Ut(e2) {
    const t2 = e2._controlledWritableStream;
    if (!e2._started) return;
    if (void 0 !== t2._inFlightWriteRequest) return;
    if ("erroring" === t2._state) return void kt(t2);
    if (0 === e2._queue.length) return;
    const r2 = e2._queue.peek().value;
    r2 === xt ? (function(e3) {
      const t3 = e3._controlledWritableStream;
      (function(e4) {
        e4._inFlightCloseRequest = e4._closeRequest, e4._closeRequest = void 0;
      })(t3), Pe(e3);
      const r3 = e3._closeAlgorithm();
      Ht(e3), h(r3, () => ((function(e4) {
        e4._inFlightCloseRequest._resolve(void 0), e4._inFlightCloseRequest = void 0, "erroring" === e4._state && (e4._storedError = void 0, void 0 !== e4._pendingAbortRequest && (e4._pendingAbortRequest._resolve(), e4._pendingAbortRequest = void 0)), e4._state = "closed";
        const t4 = e4._writer;
        void 0 !== t4 && ir(t4);
      })(t3), null), (e4) => ((function(e5, t4) {
        e5._inFlightCloseRequest._reject(t4), e5._inFlightCloseRequest = void 0, void 0 !== e5._pendingAbortRequest && (e5._pendingAbortRequest._reject(t4), e5._pendingAbortRequest = void 0), Bt(e5, t4);
      })(t3, e4), null));
    })(e2) : (function(e3, t3) {
      const r3 = e3._controlledWritableStream;
      !(function(e4) {
        e4._inFlightWriteRequest = e4._writeRequests.shift();
      })(r3);
      const o2 = e3._writeAlgorithm(t3);
      h(o2, () => {
        !(function(e4) {
          e4._inFlightWriteRequest._resolve(void 0), e4._inFlightWriteRequest = void 0;
        })(r3);
        const t4 = r3._state;
        if (Pe(e3), !At(r3) && "writable" === t4) {
          const t5 = Xt(e3);
          Dt(r3, t5);
        }
        return Ut(e3), null;
      }, (t4) => ("writable" === r3._state && Ht(e3), (function(e4, t5) {
        e4._inFlightWriteRequest._reject(t5), e4._inFlightWriteRequest = void 0, Bt(e4, t5);
      })(r3, t4), null));
    })(e2, r2);
  }
  function Gt(e2, t2) {
    "writable" === e2._controlledWritableStream._state && Jt(e2, t2);
  }
  function Xt(e2) {
    return Vt(e2) <= 0;
  }
  function Jt(e2, t2) {
    const r2 = e2._controlledWritableStream;
    Ht(e2), jt(r2, t2);
  }
  function Kt(e2) {
    return new TypeError(`WritableStream.prototype.${e2} can only be used on a WritableStream`);
  }
  function Zt(e2) {
    return new TypeError(`WritableStreamDefaultController.prototype.${e2} can only be used on a WritableStreamDefaultController`);
  }
  function er(e2) {
    return new TypeError(`WritableStreamDefaultWriter.prototype.${e2} can only be used on a WritableStreamDefaultWriter`);
  }
  function tr(e2) {
    return new TypeError("Cannot " + e2 + " a stream using a released writer");
  }
  function rr(e2) {
    e2._closedPromise = u((t2, r2) => {
      e2._closedPromise_resolve = t2, e2._closedPromise_reject = r2, e2._closedPromiseState = "pending";
    });
  }
  function or(e2, t2) {
    rr(e2), nr(e2, t2);
  }
  function nr(e2, t2) {
    void 0 !== e2._closedPromise_reject && (p(e2._closedPromise), e2._closedPromise_reject(t2), e2._closedPromise_resolve = void 0, e2._closedPromise_reject = void 0, e2._closedPromiseState = "rejected");
  }
  function ir(e2) {
    void 0 !== e2._closedPromise_resolve && (e2._closedPromise_resolve(void 0), e2._closedPromise_resolve = void 0, e2._closedPromise_reject = void 0, e2._closedPromiseState = "resolved");
  }
  function ar(e2) {
    e2._readyPromise = u((t2, r2) => {
      e2._readyPromise_resolve = t2, e2._readyPromise_reject = r2;
    }), e2._readyPromiseState = "pending";
  }
  function sr(e2, t2) {
    ar(e2), ur(e2, t2);
  }
  function lr(e2) {
    ar(e2), cr(e2);
  }
  function ur(e2, t2) {
    void 0 !== e2._readyPromise_reject && (p(e2._readyPromise), e2._readyPromise_reject(t2), e2._readyPromise_resolve = void 0, e2._readyPromise_reject = void 0, e2._readyPromiseState = "rejected");
  }
  function cr(e2) {
    void 0 !== e2._readyPromise_resolve && (e2._readyPromise_resolve(void 0), e2._readyPromise_resolve = void 0, e2._readyPromise_reject = void 0, e2._readyPromiseState = "fulfilled");
  }
  Object.defineProperties(WritableStreamDefaultController.prototype, { abortReason: { enumerable: true }, signal: { enumerable: true }, error: { enumerable: true } }), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(WritableStreamDefaultController.prototype, Symbol.toStringTag, { value: "WritableStreamDefaultController", configurable: true });
  var dr = "undefined" != typeof globalThis ? globalThis : "undefined" != typeof self ? self : "undefined" != typeof global ? global : void 0;
  var fr = (function() {
    const e2 = null == dr ? void 0 : dr.DOMException;
    return (function(e3) {
      if ("function" != typeof e3 && "object" != typeof e3) return false;
      if ("DOMException" !== e3.name) return false;
      try {
        return new e3(), true;
      } catch (e4) {
        return false;
      }
    })(e2) ? e2 : void 0;
  })() || (function() {
    const e2 = function(e3, t2) {
      this.message = e3 || "", this.name = t2 || "Error", Error.captureStackTrace && Error.captureStackTrace(this, this.constructor);
    };
    return o(e2, "DOMException"), e2.prototype = Object.create(Error.prototype), Object.defineProperty(e2.prototype, "constructor", { value: e2, writable: true, configurable: true }), e2;
  })();
  function hr(e2, t2, r2, o2, n2, i2) {
    const a2 = V(e2), s2 = Pt(t2);
    e2._disturbed = true;
    const l2 = new br(s2), _2 = new mr(l2);
    return u((m2, y2) => {
      let S2;
      if (void 0 !== i2) {
        if (S2 = () => {
          const r3 = void 0 !== i2.reason ? i2.reason : new fr("Aborted", "AbortError"), a3 = [];
          o2 || a3.push(() => "writable" === t2._state ? Wt(t2, r3) : c(void 0)), n2 || a3.push(() => "readable" === e2._state ? Mr(e2, r3) : c(void 0)), T2(() => Promise.all(a3.map((e3) => e3())), true, r3);
        }, i2.aborted) return void S2();
        i2.addEventListener("abort", S2);
      }
      function g2() {
        for (; !l2._shuttingDown && !t2._backpressure && "writable" === t2._state && !At(t2) && "readable" === e2._state && re(a2); ) te(a2, _2);
        if (l2._shuttingDown) return c(true);
        if (t2._backpressure) return f(s2._readyPromise, g2);
        const r3 = new _r(l2);
        return te(a2, r3), r3._promise;
      }
      var v2, w2, R2;
      if (pr(e2, a2._closedPromise, (e3) => (o2 ? P2(true, e3) : T2(() => Wt(t2, e3), true, e3), null)), pr(t2, s2._closedPromise, (t3) => (n2 ? P2(true, t3) : T2(() => Mr(e2, t3), true, t3), null)), v2 = e2, w2 = a2._closedPromise, R2 = () => (r2 ? P2() : T2(() => (function(e3) {
        const t3 = e3._ownerWritableStream, r3 = t3._state;
        return At(t3) || "closed" === r3 ? c(void 0) : "errored" === r3 ? d(t3._storedError) : Lt(e3);
      })(s2)), null), "closed" === v2._state ? R2() : b(w2, R2), At(t2) || "closed" === t2._state) {
        const t3 = new TypeError("the destination writable stream closed before all data could be piped to it");
        n2 ? P2(true, t3) : T2(() => Mr(e2, t3), true, t3);
      }
      function T2(e3, r3, o3) {
        function n3() {
          return h(e3(), () => C2(r3, o3), (e4) => C2(true, e4)), null;
        }
        l2._shuttingDown || (l2._shuttingDown = true, "writable" !== t2._state || At(t2) ? n3() : b(l2._waitForWritesToFinish(), n3));
      }
      function P2(e3, r3) {
        l2._shuttingDown || (l2._shuttingDown = true, "writable" !== t2._state || At(t2) ? C2(e3, r3) : b(l2._waitForWritesToFinish(), () => C2(e3, r3)));
      }
      function C2(e3, t3) {
        return Mt(s2), O(a2), void 0 !== i2 && i2.removeEventListener("abort", S2), e3 ? y2(t3) : m2(void 0), null;
      }
      p(u((e3, t3) => {
        !(function r3(o3) {
          o3 ? e3() : f(g2(), r3, t3);
        })(false);
      }));
    });
  }
  var br = class {
    constructor(e2) {
      this._writer = e2, this._shuttingDown = false, this._currentWrite = c(void 0);
    }
    _waitForWritesToFinish() {
      const e2 = this._currentWrite;
      return f(this._currentWrite, () => e2 !== this._currentWrite ? this._waitForWritesToFinish() : void 0);
    }
  };
  var _r = class {
    constructor(e2) {
      this._state = e2, this._promise = u((e3, t2) => {
        this._resolvePromise = e3, this._rejectPromise = t2;
      });
    }
    _chunkSteps(t2) {
      this._state._currentWrite = f(Yt(this._state._writer, t2), void 0, e), this._resolvePromise(false);
    }
    _closeSteps() {
      this._resolvePromise(true);
    }
    _errorSteps(e2) {
      this._rejectPromise(e2);
    }
  };
  var mr = class {
    constructor(e2) {
      this._state = e2;
    }
    _chunkSteps(t2) {
      this._state._currentWrite = f(Yt(this._state._writer, t2), void 0, e);
    }
    _closeSteps() {
    }
    _errorSteps(e2) {
    }
  };
  function pr(e2, t2, r2) {
    "errored" === e2._state ? r2(e2._storedError) : _(t2, r2);
  }
  var ReadableStreamDefaultController = class {
    constructor() {
      throw new TypeError("Illegal constructor");
    }
    get desiredSize() {
      if (!yr(this)) throw Er("desiredSize");
      return Pr(this);
    }
    close() {
      if (!yr(this)) throw Er("close");
      if (!Cr(this)) throw new TypeError("The stream is not in a state that permits close");
      wr(this);
    }
    enqueue(e2 = void 0) {
      if (!yr(this)) throw Er("enqueue");
      if (!Cr(this)) throw new TypeError("The stream is not in a state that permits enqueue");
      return Rr(this, e2);
    }
    error(e2 = void 0) {
      if (!yr(this)) throw Er("error");
      Tr(this, e2);
    }
    [T](e2) {
      qe(this);
      const t2 = this._cancelAlgorithm(e2);
      return vr(this), t2;
    }
    [P](e2) {
      const t2 = this._controlledReadableStream;
      if (this._queue.length > 0) {
        const r2 = Pe(this);
        this._closeRequested && 0 === this._queue.length ? (vr(this), Yr(t2)) : Sr(this), e2._chunkSteps(r2);
      } else U(t2, e2), Sr(this);
    }
    [C]() {
      return this._queue.length > 0;
    }
    [q]() {
    }
  };
  function yr(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_controlledReadableStream") && e2 instanceof ReadableStreamDefaultController);
  }
  function Sr(e2) {
    if (!gr(e2)) return;
    if (e2._pulling) return void (e2._pullAgain = true);
    e2._pulling = true;
    h(e2._pullAlgorithm(), () => (e2._pulling = false, e2._pullAgain && (e2._pullAgain = false, Sr(e2)), null), (t2) => (Tr(e2, t2), null));
  }
  function gr(e2) {
    const t2 = e2._controlledReadableStream;
    if (!Cr(e2)) return false;
    if (!e2._started) return false;
    if ($r(t2) && X(t2) > 0) return true;
    return Pr(e2) > 0;
  }
  function vr(e2) {
    e2._pullAlgorithm = void 0, e2._cancelAlgorithm = void 0, e2._strategySizeAlgorithm = void 0;
  }
  function wr(e2) {
    if (!Cr(e2)) return;
    const t2 = e2._controlledReadableStream;
    e2._closeRequested = true, 0 === e2._queue.length && (vr(e2), Yr(t2));
  }
  function Rr(e2, t2) {
    if (!Cr(e2)) return;
    const r2 = e2._controlledReadableStream;
    if ($r(r2) && X(r2) > 0) G(r2, t2, false);
    else {
      let r3;
      try {
        r3 = e2._strategySizeAlgorithm(t2);
      } catch (t3) {
        throw Tr(e2, t3), t3;
      }
      try {
        Ce(e2, t2, r3);
      } catch (t3) {
        throw Tr(e2, t3), t3;
      }
    }
    Sr(e2);
  }
  function Tr(e2, t2) {
    const r2 = e2._controlledReadableStream;
    "readable" === r2._state && (qe(e2), vr(e2), xr(r2, t2));
  }
  function Pr(e2) {
    const t2 = e2._controlledReadableStream._state;
    return "errored" === t2 ? null : "closed" === t2 ? 0 : e2._strategyHWM - e2._queueTotalSize;
  }
  function Cr(e2) {
    const t2 = e2._controlledReadableStream._state;
    return !e2._closeRequested && "readable" === t2;
  }
  function qr(e2, t2, r2, o2, n2, i2, a2) {
    t2._controlledReadableStream = e2, t2._queue = void 0, t2._queueTotalSize = void 0, qe(t2), t2._started = false, t2._closeRequested = false, t2._pullAgain = false, t2._pulling = false, t2._strategySizeAlgorithm = a2, t2._strategyHWM = i2, t2._pullAlgorithm = o2, t2._cancelAlgorithm = n2, e2._readableStreamController = t2;
    h(c(r2()), () => (t2._started = true, Sr(t2), null), (e3) => (Tr(t2, e3), null));
  }
  function Er(e2) {
    return new TypeError(`ReadableStreamDefaultController.prototype.${e2} can only be used on a ReadableStreamDefaultController`);
  }
  function Wr(e2, t2) {
    return Oe(e2._readableStreamController) ? (function(e3) {
      let t3, r2, o2, n2, i2, a2 = V(e3), s2 = false, l2 = false, d2 = false, f2 = false, h2 = false;
      const b2 = u((e4) => {
        i2 = e4;
      });
      function m2(e4) {
        _(e4._closedPromise, (t4) => (e4 !== a2 || (Xe(o2._readableStreamController, t4), Xe(n2._readableStreamController, t4), f2 && h2 || i2(void 0)), null));
      }
      function p2() {
        ft(a2) && (O(a2), a2 = V(e3), m2(a2));
        te(a2, { _chunkSteps: (t4) => {
          y(() => {
            l2 = false, d2 = false;
            const r3 = t4;
            let a3 = t4;
            if (!f2 && !h2) try {
              a3 = Te(t4);
            } catch (t5) {
              return Xe(o2._readableStreamController, t5), Xe(n2._readableStreamController, t5), void i2(Mr(e3, t5));
            }
            f2 || Ge(o2._readableStreamController, r3), h2 || Ge(n2._readableStreamController, a3), s2 = false, l2 ? g2() : d2 && v2();
          });
        }, _closeSteps: () => {
          s2 = false, f2 || Ue(o2._readableStreamController), h2 || Ue(n2._readableStreamController), o2._readableStreamController._pendingPullIntos.length > 0 && et(o2._readableStreamController, 0), n2._readableStreamController._pendingPullIntos.length > 0 && et(n2._readableStreamController, 0), f2 && h2 || i2(void 0);
        }, _errorSteps: () => {
          s2 = false;
        } });
      }
      function S2(t4, r3) {
        ee(a2) && (O(a2), a2 = at(e3), m2(a2));
        const u2 = r3 ? n2 : o2, c2 = r3 ? o2 : n2;
        ht(a2, t4, 1, { _chunkSteps: (t5) => {
          y(() => {
            l2 = false, d2 = false;
            const o3 = r3 ? h2 : f2;
            if (r3 ? f2 : h2) o3 || tt(u2._readableStreamController, t5);
            else {
              let r4;
              try {
                r4 = Te(t5);
              } catch (t6) {
                return Xe(u2._readableStreamController, t6), Xe(c2._readableStreamController, t6), void i2(Mr(e3, t6));
              }
              o3 || tt(u2._readableStreamController, t5), Ge(c2._readableStreamController, r4);
            }
            s2 = false, l2 ? g2() : d2 && v2();
          });
        }, _closeSteps: (e4) => {
          s2 = false;
          const t5 = r3 ? h2 : f2, o3 = r3 ? f2 : h2;
          t5 || Ue(u2._readableStreamController), o3 || Ue(c2._readableStreamController), void 0 !== e4 && (t5 || tt(u2._readableStreamController, e4), !o3 && c2._readableStreamController._pendingPullIntos.length > 0 && et(c2._readableStreamController, 0)), t5 && o3 || i2(void 0);
        }, _errorSteps: () => {
          s2 = false;
        } });
      }
      function g2() {
        if (s2) return l2 = true, c(void 0);
        s2 = true;
        const e4 = Ke(o2._readableStreamController);
        return null === e4 ? p2() : S2(e4._view, false), c(void 0);
      }
      function v2() {
        if (s2) return d2 = true, c(void 0);
        s2 = true;
        const e4 = Ke(n2._readableStreamController);
        return null === e4 ? p2() : S2(e4._view, true), c(void 0);
      }
      function w2(o3) {
        if (f2 = true, t3 = o3, h2) {
          const o4 = le([t3, r2]), n3 = Mr(e3, o4);
          i2(n3);
        }
        return b2;
      }
      function R2(o3) {
        if (h2 = true, r2 = o3, f2) {
          const o4 = le([t3, r2]), n3 = Mr(e3, o4);
          i2(n3);
        }
        return b2;
      }
      function T2() {
      }
      return o2 = Fr(T2, g2, w2), n2 = Fr(T2, v2, R2), m2(a2), [o2, n2];
    })(e2) : (function(e3) {
      const t3 = V(e3);
      let r2, o2, n2, i2, a2, s2 = false, l2 = false, d2 = false, f2 = false;
      const h2 = u((e4) => {
        a2 = e4;
      });
      function b2() {
        if (s2) return l2 = true, c(void 0);
        s2 = true;
        return te(t3, { _chunkSteps: (e4) => {
          y(() => {
            l2 = false;
            const t4 = e4, r3 = e4;
            d2 || Rr(n2._readableStreamController, t4), f2 || Rr(i2._readableStreamController, r3), s2 = false, l2 && b2();
          });
        }, _closeSteps: () => {
          s2 = false, d2 || wr(n2._readableStreamController), f2 || wr(i2._readableStreamController), d2 && f2 || a2(void 0);
        }, _errorSteps: () => {
          s2 = false;
        } }), c(void 0);
      }
      function m2(t4) {
        if (d2 = true, r2 = t4, f2) {
          const t5 = le([r2, o2]), n3 = Mr(e3, t5);
          a2(n3);
        }
        return h2;
      }
      function p2(t4) {
        if (f2 = true, o2 = t4, d2) {
          const t5 = le([r2, o2]), n3 = Mr(e3, t5);
          a2(n3);
        }
        return h2;
      }
      function S2() {
      }
      return n2 = Dr(S2, b2, m2), i2 = Dr(S2, b2, p2), _(t3._closedPromise, (e4) => (Tr(n2._readableStreamController, e4), Tr(i2._readableStreamController, e4), d2 && f2 || a2(void 0), null)), [n2, i2];
    })(e2);
  }
  function Or(r2) {
    return t(o2 = r2) && void 0 !== o2.getReader ? (function(r3) {
      let o3;
      function n2() {
        let e2;
        try {
          e2 = r3.read();
        } catch (e3) {
          return d(e3);
        }
        return m(e2, (e3) => {
          if (!t(e3)) throw new TypeError("The promise returned by the reader.read() method must fulfill with an object");
          if (e3.done) wr(o3._readableStreamController);
          else {
            const t2 = e3.value;
            Rr(o3._readableStreamController, t2);
          }
        });
      }
      function i2(e2) {
        try {
          return c(r3.cancel(e2));
        } catch (e3) {
          return d(e3);
        }
      }
      return o3 = Dr(e, n2, i2, 0), o3;
    })(r2.getReader()) : (function(r3) {
      let o3;
      const n2 = me(r3, "async");
      function i2() {
        let e2;
        try {
          e2 = pe(n2);
        } catch (e3) {
          return d(e3);
        }
        return m(c(e2), (e3) => {
          if (!t(e3)) throw new TypeError("The promise returned by the iterator.next() method must fulfill with an object");
          if (e3.done) wr(o3._readableStreamController);
          else {
            const t2 = e3.value;
            Rr(o3._readableStreamController, t2);
          }
        });
      }
      function a2(e2) {
        const r4 = n2.iterator;
        let o4;
        try {
          o4 = he(r4, "return");
        } catch (e3) {
          return d(e3);
        }
        if (void 0 === o4) return c(void 0);
        return m(g(o4, r4, [e2]), (e3) => {
          if (!t(e3)) throw new TypeError("The promise returned by the iterator.return() method must fulfill with an object");
        });
      }
      return o3 = Dr(e, i2, a2, 0), o3;
    })(r2);
    var o2;
  }
  function Br(e2, t2, r2) {
    return I(e2, r2), (r3) => g(e2, t2, [r3]);
  }
  function jr(e2, t2, r2) {
    return I(e2, r2), (r3) => g(e2, t2, [r3]);
  }
  function kr(e2, t2, r2) {
    return I(e2, r2), (r3) => S(e2, t2, [r3]);
  }
  function Ar(e2, t2) {
    if ("bytes" !== (e2 = `${e2}`)) throw new TypeError(`${t2} '${e2}' is not a valid enumeration value for ReadableStreamType`);
    return e2;
  }
  function zr(e2, t2) {
    L(e2, t2);
    const r2 = null == e2 ? void 0 : e2.preventAbort, o2 = null == e2 ? void 0 : e2.preventCancel, n2 = null == e2 ? void 0 : e2.preventClose, i2 = null == e2 ? void 0 : e2.signal;
    return void 0 !== i2 && (function(e3, t3) {
      if (!(function(e4) {
        if ("object" != typeof e4 || null === e4) return false;
        try {
          return "boolean" == typeof e4.aborted;
        } catch (e5) {
          return false;
        }
      })(e3)) throw new TypeError(`${t3} is not an AbortSignal.`);
    })(i2, `${t2} has member 'signal' that`), { preventAbort: Boolean(r2), preventCancel: Boolean(o2), preventClose: Boolean(n2), signal: i2 };
  }
  Object.defineProperties(ReadableStreamDefaultController.prototype, { close: { enumerable: true }, enqueue: { enumerable: true }, error: { enumerable: true }, desiredSize: { enumerable: true } }), o(ReadableStreamDefaultController.prototype.close, "close"), o(ReadableStreamDefaultController.prototype.enqueue, "enqueue"), o(ReadableStreamDefaultController.prototype.error, "error"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamDefaultController.prototype, Symbol.toStringTag, { value: "ReadableStreamDefaultController", configurable: true });
  var ReadableStream = class {
    constructor(e2 = {}, t2 = {}) {
      void 0 === e2 ? e2 = null : $(e2, "First parameter");
      const r2 = yt(t2, "Second parameter"), o2 = (function(e3, t3) {
        L(e3, t3);
        const r3 = e3, o3 = null == r3 ? void 0 : r3.autoAllocateChunkSize, n2 = null == r3 ? void 0 : r3.cancel, i2 = null == r3 ? void 0 : r3.pull, a2 = null == r3 ? void 0 : r3.start, s2 = null == r3 ? void 0 : r3.type;
        return { autoAllocateChunkSize: void 0 === o3 ? void 0 : N(o3, `${t3} has member 'autoAllocateChunkSize' that`), cancel: void 0 === n2 ? void 0 : Br(n2, r3, `${t3} has member 'cancel' that`), pull: void 0 === i2 ? void 0 : jr(i2, r3, `${t3} has member 'pull' that`), start: void 0 === a2 ? void 0 : kr(a2, r3, `${t3} has member 'start' that`), type: void 0 === s2 ? void 0 : Ar(s2, `${t3} has member 'type' that`) };
      })(e2, "First parameter");
      if (Lr(this), "bytes" === o2.type) {
        if (void 0 !== r2.size) throw new RangeError("The strategy for a byte stream cannot have a size function");
        !(function(e3, t3, r3) {
          const o3 = Object.create(ReadableByteStreamController.prototype);
          let n2, i2, a2;
          n2 = void 0 !== t3.start ? () => t3.start(o3) : () => {
          }, i2 = void 0 !== t3.pull ? () => t3.pull(o3) : () => c(void 0), a2 = void 0 !== t3.cancel ? (e4) => t3.cancel(e4) : () => c(void 0);
          const s2 = t3.autoAllocateChunkSize;
          if (0 === s2) throw new TypeError("autoAllocateChunkSize must be greater than 0");
          rt(e3, o3, n2, i2, a2, r3, s2);
        })(this, o2, mt(r2, 0));
      } else {
        const e3 = pt(r2);
        !(function(e4, t3, r3, o3) {
          const n2 = Object.create(ReadableStreamDefaultController.prototype);
          let i2, a2, s2;
          i2 = void 0 !== t3.start ? () => t3.start(n2) : () => {
          }, a2 = void 0 !== t3.pull ? () => t3.pull(n2) : () => c(void 0), s2 = void 0 !== t3.cancel ? (e5) => t3.cancel(e5) : () => c(void 0), qr(e4, n2, i2, a2, s2, r3, o3);
        })(this, o2, mt(r2, 1), e3);
      }
    }
    get locked() {
      if (!Ir(this)) throw Qr("locked");
      return $r(this);
    }
    cancel(e2 = void 0) {
      return Ir(this) ? $r(this) ? d(new TypeError("Cannot cancel a stream that already has a reader")) : Mr(this, e2) : d(Qr("cancel"));
    }
    getReader(e2 = void 0) {
      if (!Ir(this)) throw Qr("getReader");
      return void 0 === (function(e3, t2) {
        L(e3, t2);
        const r2 = null == e3 ? void 0 : e3.mode;
        return { mode: void 0 === r2 ? void 0 : it(r2, `${t2} has member 'mode' that`) };
      })(e2, "First parameter").mode ? V(this) : at(this);
    }
    pipeThrough(e2, t2 = {}) {
      if (!Ir(this)) throw Qr("pipeThrough");
      M(e2, 1, "pipeThrough");
      const r2 = (function(e3, t3) {
        L(e3, t3);
        const r3 = null == e3 ? void 0 : e3.readable;
        Y(r3, "readable", "ReadableWritablePair"), H(r3, `${t3} has member 'readable' that`);
        const o3 = null == e3 ? void 0 : e3.writable;
        return Y(o3, "writable", "ReadableWritablePair"), Tt(o3, `${t3} has member 'writable' that`), { readable: r3, writable: o3 };
      })(e2, "First parameter"), o2 = zr(t2, "Second parameter");
      if ($r(this)) throw new TypeError("ReadableStream.prototype.pipeThrough cannot be used on a locked ReadableStream");
      if (Et(r2.writable)) throw new TypeError("ReadableStream.prototype.pipeThrough cannot be used on a locked WritableStream");
      return p(hr(this, r2.writable, o2.preventClose, o2.preventAbort, o2.preventCancel, o2.signal)), r2.readable;
    }
    pipeTo(e2, t2 = {}) {
      if (!Ir(this)) return d(Qr("pipeTo"));
      if (void 0 === e2) return d("Parameter 1 is required in 'pipeTo'.");
      if (!qt(e2)) return d(new TypeError("ReadableStream.prototype.pipeTo's first argument must be a WritableStream"));
      let r2;
      try {
        r2 = zr(t2, "Second parameter");
      } catch (e3) {
        return d(e3);
      }
      return $r(this) ? d(new TypeError("ReadableStream.prototype.pipeTo cannot be used on a locked ReadableStream")) : Et(e2) ? d(new TypeError("ReadableStream.prototype.pipeTo cannot be used on a locked WritableStream")) : hr(this, e2, r2.preventClose, r2.preventAbort, r2.preventCancel, r2.signal);
    }
    tee() {
      if (!Ir(this)) throw Qr("tee");
      return le(Wr(this));
    }
    values(e2 = void 0) {
      if (!Ir(this)) throw Qr("values");
      return (function(e3, t2) {
        const r2 = V(e3), o2 = new ye(r2, t2), n2 = Object.create(ge);
        return n2._asyncIteratorImpl = o2, n2;
      })(this, (function(e3, t2) {
        L(e3, t2);
        const r2 = null == e3 ? void 0 : e3.preventCancel;
        return { preventCancel: Boolean(r2) };
      })(e2, "First parameter").preventCancel);
    }
    [_e](e2) {
      return this.values(e2);
    }
    static from(e2) {
      return Or(e2);
    }
  };
  function Dr(e2, t2, r2, o2 = 1, n2 = () => 1) {
    const i2 = Object.create(ReadableStream.prototype);
    Lr(i2);
    return qr(i2, Object.create(ReadableStreamDefaultController.prototype), e2, t2, r2, o2, n2), i2;
  }
  function Fr(e2, t2, r2) {
    const o2 = Object.create(ReadableStream.prototype);
    Lr(o2);
    return rt(o2, Object.create(ReadableByteStreamController.prototype), e2, t2, r2, 0, void 0), o2;
  }
  function Lr(e2) {
    e2._state = "readable", e2._reader = void 0, e2._storedError = void 0, e2._disturbed = false;
  }
  function Ir(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_readableStreamController") && e2 instanceof ReadableStream);
  }
  function $r(e2) {
    return void 0 !== e2._reader;
  }
  function Mr(t2, r2) {
    if (t2._disturbed = true, "closed" === t2._state) return c(void 0);
    if ("errored" === t2._state) return d(t2._storedError);
    Yr(t2);
    const o2 = t2._reader;
    if (void 0 !== o2 && ft(o2)) {
      const e2 = o2._readIntoRequests;
      o2._readIntoRequests = new v(), e2.forEach((e3) => {
        e3._closeSteps(void 0);
      });
    }
    return m(t2._readableStreamController[T](r2), e);
  }
  function Yr(e2) {
    e2._state = "closed";
    const t2 = e2._reader;
    if (void 0 !== t2 && (z(t2), ee(t2))) {
      const e3 = t2._readRequests;
      t2._readRequests = new v(), e3.forEach((e4) => {
        e4._closeSteps();
      });
    }
  }
  function xr(e2, t2) {
    e2._state = "errored", e2._storedError = t2;
    const r2 = e2._reader;
    void 0 !== r2 && (A(r2, t2), ee(r2) ? oe(r2, t2) : bt(r2, t2));
  }
  function Qr(e2) {
    return new TypeError(`ReadableStream.prototype.${e2} can only be used on a ReadableStream`);
  }
  function Nr(e2, t2) {
    L(e2, t2);
    const r2 = null == e2 ? void 0 : e2.highWaterMark;
    return Y(r2, "highWaterMark", "QueuingStrategyInit"), { highWaterMark: x(r2) };
  }
  Object.defineProperties(ReadableStream, { from: { enumerable: true } }), Object.defineProperties(ReadableStream.prototype, { cancel: { enumerable: true }, getReader: { enumerable: true }, pipeThrough: { enumerable: true }, pipeTo: { enumerable: true }, tee: { enumerable: true }, values: { enumerable: true }, locked: { enumerable: true } }), o(ReadableStream.from, "from"), o(ReadableStream.prototype.cancel, "cancel"), o(ReadableStream.prototype.getReader, "getReader"), o(ReadableStream.prototype.pipeThrough, "pipeThrough"), o(ReadableStream.prototype.pipeTo, "pipeTo"), o(ReadableStream.prototype.tee, "tee"), o(ReadableStream.prototype.values, "values"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStream.prototype, Symbol.toStringTag, { value: "ReadableStream", configurable: true }), Object.defineProperty(ReadableStream.prototype, _e, { value: ReadableStream.prototype.values, writable: true, configurable: true });
  var Hr = (e2) => e2.byteLength;
  o(Hr, "size");
  var ByteLengthQueuingStrategy = class {
    constructor(e2) {
      M(e2, 1, "ByteLengthQueuingStrategy"), e2 = Nr(e2, "First parameter"), this._byteLengthQueuingStrategyHighWaterMark = e2.highWaterMark;
    }
    get highWaterMark() {
      if (!Ur(this)) throw Vr("highWaterMark");
      return this._byteLengthQueuingStrategyHighWaterMark;
    }
    get size() {
      if (!Ur(this)) throw Vr("size");
      return Hr;
    }
  };
  function Vr(e2) {
    return new TypeError(`ByteLengthQueuingStrategy.prototype.${e2} can only be used on a ByteLengthQueuingStrategy`);
  }
  function Ur(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_byteLengthQueuingStrategyHighWaterMark") && e2 instanceof ByteLengthQueuingStrategy);
  }
  Object.defineProperties(ByteLengthQueuingStrategy.prototype, { highWaterMark: { enumerable: true }, size: { enumerable: true } }), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ByteLengthQueuingStrategy.prototype, Symbol.toStringTag, { value: "ByteLengthQueuingStrategy", configurable: true });
  var Gr = () => 1;
  o(Gr, "size");
  var CountQueuingStrategy = class {
    constructor(e2) {
      M(e2, 1, "CountQueuingStrategy"), e2 = Nr(e2, "First parameter"), this._countQueuingStrategyHighWaterMark = e2.highWaterMark;
    }
    get highWaterMark() {
      if (!Jr(this)) throw Xr("highWaterMark");
      return this._countQueuingStrategyHighWaterMark;
    }
    get size() {
      if (!Jr(this)) throw Xr("size");
      return Gr;
    }
  };
  function Xr(e2) {
    return new TypeError(`CountQueuingStrategy.prototype.${e2} can only be used on a CountQueuingStrategy`);
  }
  function Jr(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_countQueuingStrategyHighWaterMark") && e2 instanceof CountQueuingStrategy);
  }
  function Kr(e2, t2, r2) {
    return I(e2, r2), (r3) => g(e2, t2, [r3]);
  }
  function Zr(e2, t2, r2) {
    return I(e2, r2), (r3) => S(e2, t2, [r3]);
  }
  function eo(e2, t2, r2) {
    return I(e2, r2), (r3, o2) => g(e2, t2, [r3, o2]);
  }
  function to(e2, t2, r2) {
    return I(e2, r2), (r3) => g(e2, t2, [r3]);
  }
  Object.defineProperties(CountQueuingStrategy.prototype, { highWaterMark: { enumerable: true }, size: { enumerable: true } }), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(CountQueuingStrategy.prototype, Symbol.toStringTag, { value: "CountQueuingStrategy", configurable: true });
  var TransformStream = class {
    constructor(e2 = {}, t2 = {}, r2 = {}) {
      void 0 === e2 && (e2 = null);
      const o2 = yt(t2, "Second parameter"), n2 = yt(r2, "Third parameter"), i2 = (function(e3, t3) {
        L(e3, t3);
        const r3 = null == e3 ? void 0 : e3.cancel, o3 = null == e3 ? void 0 : e3.flush, n3 = null == e3 ? void 0 : e3.readableType, i3 = null == e3 ? void 0 : e3.start, a3 = null == e3 ? void 0 : e3.transform, s3 = null == e3 ? void 0 : e3.writableType;
        return { cancel: void 0 === r3 ? void 0 : to(r3, e3, `${t3} has member 'cancel' that`), flush: void 0 === o3 ? void 0 : Kr(o3, e3, `${t3} has member 'flush' that`), readableType: n3, start: void 0 === i3 ? void 0 : Zr(i3, e3, `${t3} has member 'start' that`), transform: void 0 === a3 ? void 0 : eo(a3, e3, `${t3} has member 'transform' that`), writableType: s3 };
      })(e2, "First parameter");
      if (void 0 !== i2.readableType) throw new RangeError("Invalid readableType specified");
      if (void 0 !== i2.writableType) throw new RangeError("Invalid writableType specified");
      const a2 = mt(n2, 0), s2 = pt(n2), l2 = mt(o2, 1), f2 = pt(o2);
      let b2;
      !(function(e3, t3, r3, o3, n3, i3) {
        function a3() {
          return t3;
        }
        function s3(t4) {
          return (function(e4, t5) {
            const r4 = e4._transformStreamController;
            if (e4._backpressure) {
              return m(e4._backpressureChangePromise, () => {
                const o4 = e4._writable;
                if ("erroring" === o4._state) throw o4._storedError;
                return co(r4, t5);
              });
            }
            return co(r4, t5);
          })(e3, t4);
        }
        function l3(t4) {
          return (function(e4, t5) {
            const r4 = e4._transformStreamController;
            if (void 0 !== r4._finishPromise) return r4._finishPromise;
            const o4 = e4._readable;
            r4._finishPromise = u((e5, t6) => {
              r4._finishPromise_resolve = e5, r4._finishPromise_reject = t6;
            });
            const n4 = r4._cancelAlgorithm(t5);
            return lo(r4), h(n4, () => ("errored" === o4._state ? bo(r4, o4._storedError) : (Tr(o4._readableStreamController, t5), ho(r4)), null), (e5) => (Tr(o4._readableStreamController, e5), bo(r4, e5), null)), r4._finishPromise;
          })(e3, t4);
        }
        function c2() {
          return (function(e4) {
            const t4 = e4._transformStreamController;
            if (void 0 !== t4._finishPromise) return t4._finishPromise;
            const r4 = e4._readable;
            t4._finishPromise = u((e5, r5) => {
              t4._finishPromise_resolve = e5, t4._finishPromise_reject = r5;
            });
            const o4 = t4._flushAlgorithm();
            return lo(t4), h(o4, () => ("errored" === r4._state ? bo(t4, r4._storedError) : (wr(r4._readableStreamController), ho(t4)), null), (e5) => (Tr(r4._readableStreamController, e5), bo(t4, e5), null)), t4._finishPromise;
          })(e3);
        }
        function d2() {
          return (function(e4) {
            return ao(e4, false), e4._backpressureChangePromise;
          })(e3);
        }
        function f3(t4) {
          return (function(e4, t5) {
            const r4 = e4._transformStreamController;
            if (void 0 !== r4._finishPromise) return r4._finishPromise;
            const o4 = e4._writable;
            r4._finishPromise = u((e5, t6) => {
              r4._finishPromise_resolve = e5, r4._finishPromise_reject = t6;
            });
            const n4 = r4._cancelAlgorithm(t5);
            return lo(r4), h(n4, () => ("errored" === o4._state ? bo(r4, o4._storedError) : (Gt(o4._writableStreamController, t5), io(e4), ho(r4)), null), (t6) => (Gt(o4._writableStreamController, t6), io(e4), bo(r4, t6), null)), r4._finishPromise;
          })(e3, t4);
        }
        e3._writable = (function(e4, t4, r4, o4, n4 = 1, i4 = () => 1) {
          const a4 = Object.create(WritableStream.prototype);
          return Ct(a4), Nt(a4, Object.create(WritableStreamDefaultController.prototype), e4, t4, r4, o4, n4, i4), a4;
        })(a3, s3, c2, l3, r3, o3), e3._readable = Dr(a3, d2, f3, n3, i3), e3._backpressure = void 0, e3._backpressureChangePromise = void 0, e3._backpressureChangePromise_resolve = void 0, ao(e3, true), e3._transformStreamController = void 0;
      })(this, u((e3) => {
        b2 = e3;
      }), l2, f2, a2, s2), (function(e3, t3) {
        const r3 = Object.create(TransformStreamDefaultController.prototype);
        let o3, n3, i3;
        o3 = void 0 !== t3.transform ? (e4) => t3.transform(e4, r3) : (e4) => {
          try {
            return uo(r3, e4), c(void 0);
          } catch (e5) {
            return d(e5);
          }
        };
        n3 = void 0 !== t3.flush ? () => t3.flush(r3) : () => c(void 0);
        i3 = void 0 !== t3.cancel ? (e4) => t3.cancel(e4) : () => c(void 0);
        !(function(e4, t4, r4, o4, n4) {
          t4._controlledTransformStream = e4, e4._transformStreamController = t4, t4._transformAlgorithm = r4, t4._flushAlgorithm = o4, t4._cancelAlgorithm = n4, t4._finishPromise = void 0, t4._finishPromise_resolve = void 0, t4._finishPromise_reject = void 0;
        })(e3, r3, o3, n3, i3);
      })(this, i2), void 0 !== i2.start ? b2(i2.start(this._transformStreamController)) : b2(void 0);
    }
    get readable() {
      if (!ro(this)) throw _o("readable");
      return this._readable;
    }
    get writable() {
      if (!ro(this)) throw _o("writable");
      return this._writable;
    }
  };
  function ro(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_transformStreamController") && e2 instanceof TransformStream);
  }
  function oo(e2, t2) {
    Tr(e2._readable._readableStreamController, t2), no(e2, t2);
  }
  function no(e2, t2) {
    lo(e2._transformStreamController), Gt(e2._writable._writableStreamController, t2), io(e2);
  }
  function io(e2) {
    e2._backpressure && ao(e2, false);
  }
  function ao(e2, t2) {
    void 0 !== e2._backpressureChangePromise && e2._backpressureChangePromise_resolve(), e2._backpressureChangePromise = u((t3) => {
      e2._backpressureChangePromise_resolve = t3;
    }), e2._backpressure = t2;
  }
  Object.defineProperties(TransformStream.prototype, { readable: { enumerable: true }, writable: { enumerable: true } }), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(TransformStream.prototype, Symbol.toStringTag, { value: "TransformStream", configurable: true });
  var TransformStreamDefaultController = class {
    constructor() {
      throw new TypeError("Illegal constructor");
    }
    get desiredSize() {
      if (!so(this)) throw fo("desiredSize");
      return Pr(this._controlledTransformStream._readable._readableStreamController);
    }
    enqueue(e2 = void 0) {
      if (!so(this)) throw fo("enqueue");
      uo(this, e2);
    }
    error(e2 = void 0) {
      if (!so(this)) throw fo("error");
      var t2;
      t2 = e2, oo(this._controlledTransformStream, t2);
    }
    terminate() {
      if (!so(this)) throw fo("terminate");
      !(function(e2) {
        const t2 = e2._controlledTransformStream;
        wr(t2._readable._readableStreamController);
        const r2 = new TypeError("TransformStream terminated");
        no(t2, r2);
      })(this);
    }
  };
  function so(e2) {
    return !!t(e2) && (!!Object.prototype.hasOwnProperty.call(e2, "_controlledTransformStream") && e2 instanceof TransformStreamDefaultController);
  }
  function lo(e2) {
    e2._transformAlgorithm = void 0, e2._flushAlgorithm = void 0, e2._cancelAlgorithm = void 0;
  }
  function uo(e2, t2) {
    const r2 = e2._controlledTransformStream, o2 = r2._readable._readableStreamController;
    if (!Cr(o2)) throw new TypeError("Readable side is not in a state that permits enqueue");
    try {
      Rr(o2, t2);
    } catch (e3) {
      throw no(r2, e3), r2._readable._storedError;
    }
    const n2 = (function(e3) {
      return !gr(e3);
    })(o2);
    n2 !== r2._backpressure && ao(r2, true);
  }
  function co(e2, t2) {
    return m(e2._transformAlgorithm(t2), void 0, (t3) => {
      throw oo(e2._controlledTransformStream, t3), t3;
    });
  }
  function fo(e2) {
    return new TypeError(`TransformStreamDefaultController.prototype.${e2} can only be used on a TransformStreamDefaultController`);
  }
  function ho(e2) {
    void 0 !== e2._finishPromise_resolve && (e2._finishPromise_resolve(), e2._finishPromise_resolve = void 0, e2._finishPromise_reject = void 0);
  }
  function bo(e2, t2) {
    void 0 !== e2._finishPromise_reject && (p(e2._finishPromise), e2._finishPromise_reject(t2), e2._finishPromise_resolve = void 0, e2._finishPromise_reject = void 0);
  }
  function _o(e2) {
    return new TypeError(`TransformStream.prototype.${e2} can only be used on a TransformStream`);
  }
  Object.defineProperties(TransformStreamDefaultController.prototype, { enqueue: { enumerable: true }, error: { enumerable: true }, terminate: { enumerable: true }, desiredSize: { enumerable: true } }), o(TransformStreamDefaultController.prototype.enqueue, "enqueue"), o(TransformStreamDefaultController.prototype.error, "error"), o(TransformStreamDefaultController.prototype.terminate, "terminate"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(TransformStreamDefaultController.prototype, Symbol.toStringTag, { value: "TransformStreamDefaultController", configurable: true });

  // core.js
  var import_structured_clone = __toESM(require_structured_clone2(), 1);
  var import_xpath = __toESM(require_xpath(), 1);
  for (const [name2, value] of Object.entries(ponyfill_exports)) Object.defineProperty(globalThis, name2, { value, writable: true, configurable: true });
  Object.defineProperty(globalThis, "structuredClone", { value: import_structured_clone.default, writable: true, configurable: true });
  var XPathEvaluator = class {
    createExpression(expression, resolver = null) {
      const compiled = import_xpath.default.createExpression(String(expression), typeof resolver === "function" ? { lookupNamespaceURI: resolver } : resolver);
      const evaluate = compiled.evaluate;
      compiled.evaluate = function(node, type = 0, result = null) {
        const doc = node.nodeType === 9 ? node : node.ownerDocument;
        this.context.allowAnyNamespaceForNoPrefix = doc?.contentType === "text/html";
        return evaluate.call(this, node, type, result);
      };
      return compiled;
    }
    createNSResolver(node) {
      return import_xpath.default.createNSResolver(node);
    }
    evaluate(expression, node, resolver = null, type = 0, result = null) {
      return this.createExpression(expression, resolver).evaluate(node, type, result);
    }
  };
  Object.assign(globalThis, { XPathEvaluator, XPathResult: import_xpath.default.XPathResult, XPathExpression: import_xpath.default.XPathExpression });
  for (const name2 of ["createExpression", "createNSResolver", "evaluate"]) {
    Object.defineProperty(Document.prototype, name2, { value: XPathEvaluator.prototype[name2], writable: true, configurable: true });
  }
})();
/*! Bundled license information:

web-streams-polyfill/dist/ponyfill.mjs:
  (**
   * @license
   * web-streams-polyfill v4.3.0
   * Copyright 2026 Mattias Buelens, Diwank Singh Tomer and other contributors.
   * This code is released under the MIT license.
   * SPDX-License-Identifier: MIT
   *)
*/
